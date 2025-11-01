use crate::config::DnsServiceConfig;
use crate::plugin::ServicePlugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct KnotConfig {
    server: KnotServerConfig,
    zone: Vec<KnotZoneConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KnotServerConfig {
    rundir: String,
    #[serde(default)]
    listen: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KnotZoneConfig {
    domain: String,
    file: String,
    #[serde(default)]
    dnssec_signing: Option<String>,
}

/// Knot DNS service integration
pub struct KnotService {
    config: DnsServiceConfig,
    zones: Arc<RwLock<HashMap<String, KnotZoneData>>>,
    config_path: PathBuf,
}

#[derive(Debug, Clone)]
struct KnotZoneData {
    domain: String,
    zone_file_path: PathBuf,
    records: Vec<DnsRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DnsRecord {
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    value: String,
    ttl: Option<u32>,
}

impl KnotService {
    pub fn new(config: DnsServiceConfig) -> Self {
        let config_path = PathBuf::from(&config.config_path);
        Self {
            config,
            zones: Arc::new(RwLock::new(HashMap::new())),
            config_path,
        }
    }

    async fn generate_config(&self) -> Result<()> {
        let zones = self.zones.read().await;
        
        let knot_config = KnotConfig {
            server: KnotServerConfig {
                rundir: "/var/lib/knot".to_string(),
                listen: vec!["0.0.0.0@53".to_string(), "::@53".to_string()],
            },
            zone: zones
                .values()
                .map(|zone_data| KnotZoneConfig {
                    domain: zone_data.domain.clone(),
                    file: zone_data.zone_file_path.to_string_lossy().to_string(),
                    dnssec_signing: Some("on".to_string()),
                })
                .collect(),
        };

        // Write Knot config file
        let config_json = serde_json::to_string_pretty(&knot_config)?;
        std::fs::write(&self.config_path, config_json)
            .context(format!("Failed to write Knot config to {:?}", self.config_path))?;

        info!("Generated Knot config with {} zones", zones.len());
        Ok(())
    }

    async fn generate_zone_file(&self, zone_data: &KnotZoneData) -> Result<()> {
        // Create zone directory if it doesn't exist
        if let Some(parent) = zone_data.zone_file_path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create zone directory: {:?}", parent))?;
        }

        // Generate zone file content
        let mut zone_content = format!("$ORIGIN {}\n", zone_data.domain);
        zone_content.push_str(&format!("$TTL 3600\n"));
        zone_content.push_str("\n");
        zone_content.push_str(&format!("@\tIN\tSOA\tns1.{}. admin.{}. (\n", zone_data.domain, zone_data.domain));
        zone_content.push_str("\t\t1\t; Serial\n");
        zone_content.push_str("\t\t3600\t; Refresh\n");
        zone_content.push_str("\t\t1800\t; Retry\n");
        zone_content.push_str("\t\t604800\t; Expire\n");
        zone_content.push_str("\t\t86400\t; Minimum TTL\n");
        zone_content.push_str(")\n\n");

        for record in &zone_data.records {
            let ttl = record.ttl.unwrap_or(3600);
            zone_content.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                record.name, ttl, record.record_type, record.value
            ));
        }

        std::fs::write(&zone_data.zone_file_path, zone_content)
            .context(format!("Failed to write zone file: {:?}", zone_data.zone_file_path))?;

        info!("Generated zone file for {}", zone_data.domain);
        Ok(())
    }

    async fn reload_knot(&self) -> Result<()> {
        info!("Reloading Knot DNS");
        
        // Try to reload using knotc (Knot control utility)
        let output = Command::new("knotc")
            .arg("reload")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Knot DNS reloaded successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Knot reload command failed: {}", stderr);
                    // Fallback to systemctl restart
                    return self.restart_knot().await;
                }
            }
            Err(e) => {
                warn!("knotc not available, using systemctl: {}", e);
                return self.restart_knot().await;
            }
        }

        Ok(())
    }

    async fn restart_knot(&self) -> Result<()> {
        info!("Restarting Knot DNS service");
        
        let output = Command::new("systemctl")
            .arg("restart")
            .arg("knot")
            .output()
            .context("Failed to restart Knot service")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to restart Knot: {}", stderr));
        }

        info!("Knot DNS service restarted");
        Ok(())
    }

    async fn parse_zone_from_etcd(&self, zone_name: &str, zone_data: &[u8]) -> Result<KnotZoneData> {
        #[derive(Deserialize)]
        struct ZoneJson {
            domain: String,
            ttl: Option<u32>,
            records: Vec<DnsRecord>,
        }

        let zone_json: ZoneJson = serde_json::from_slice(zone_data)
            .context(format!("Failed to parse zone JSON for {}", zone_name))?;

        let zone_file_path = PathBuf::from(&self.config.zone_dir)
            .join(format!("{}.zone", zone_name));

        Ok(KnotZoneData {
            domain: zone_json.domain,
            zone_file_path,
            records: zone_json.records,
        })
    }
}

#[async_trait]
impl ServicePlugin for KnotService {
    fn name(&self) -> &str {
        "knot-dns"
    }

    async fn init(&mut self, _config: &[u8]) -> Result<()> {
        info!("Initializing Knot DNS service");
        info!("Config path: {:?}", self.config_path);
        info!("Zone directory: {}", self.config.zone_dir);

        // Ensure directories exist
        std::fs::create_dir_all(&self.config.zone_dir)
            .context(format!("Failed to create zone directory: {}", self.config.zone_dir))?;

        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create config directory: {:?}", parent))?;
        }

        // Generate initial config
        self.generate_config().await?;

        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
        if key.contains("/dns/zones/") && !key.ends_with("/zonefile") {
            // Extract zone name from key (format: /nnoe/dns/zones/<zone-name>)
            let parts: Vec<&str> = key.split('/').collect();
            if let Some(zone_name) = parts.last() {
                match self.parse_zone_from_etcd(zone_name, value).await {
                    Ok(zone_data) => {
                        // Generate zone file
                        self.generate_zone_file(&zone_data).await?;
                        
                        // Update zones map
                        let mut zones = self.zones.write().await;
                        zones.insert(zone_name.to_string(), zone_data);
                        
                        // Regenerate Knot config
                        drop(zones);
                        self.generate_config().await?;
                        
                        // Reload Knot
                        self.reload_knot().await?;
                        
                        info!("Zone updated: {}", zone_name);
                    }
                    Err(e) => {
                        error!("Failed to parse zone {}: {}", zone_name, e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        info!("Reloading Knot DNS service");
        self.generate_config().await?;
        self.reload_knot().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Knot DNS service");
        let mut zones = self.zones.write().await;
        zones.clear();
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if Knot is running
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg("knot")
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => {
                // Fallback: try to check if knot process exists
                let output = Command::new("pgrep")
                    .arg("-f")
                    .arg("knotd")
                    .output();
                Ok(output.map(|o| o.status.success()).unwrap_or(false))
            }
        }
    }
}

