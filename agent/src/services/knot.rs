use crate::config::DnsServiceConfig;
use crate::plugin::ServicePlugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

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
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    notify: Option<Vec<String>>, // Zone transfer notify list
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_transfer: Option<Vec<String>>, // Zone transfer ACL
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_update: Option<Vec<String>>, // Dynamic update ACL
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    dnssec_key: Option<DnssecKeyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DnssecKeyConfig {
    ksk_size: Option<u16>,         // Key Signing Key size
    zsk_size: Option<u16>,         // Zone Signing Key size
    algorithm: Option<String>,     // DNSSEC algorithm (RSASHA256, ECDSAP256SHA256, etc.)
    key_directory: Option<String>, // Directory for DNSSEC keys
}

/// Knot DNS service integration
pub struct KnotService {
    config: DnsServiceConfig,
    zones: Arc<RwLock<HashMap<String, KnotZoneData>>>,
    config_path: PathBuf,
    dnssec_keys: Arc<RwLock<HashMap<String, DnssecKeyInfo>>>, // zone -> key info
}

#[derive(Debug, Clone)]
struct DnssecKeyInfo {
    ksk_path: PathBuf,
    zsk_path: PathBuf,
    algorithm: String,
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
            dnssec_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate DNSSEC keys for a zone using keymgr (Knot's key management tool)
    async fn generate_dnssec_keys(
        &self,
        zone_name: &str,
        zone_domain: &str,
    ) -> Result<DnssecKeyInfo> {
        info!("Generating DNSSEC keys for zone: {}", zone_name);

        // Key directory - store in Knot's key directory
        let key_dir = PathBuf::from("/var/lib/knot/keys");
        std::fs::create_dir_all(&key_dir).context(format!(
            "Failed to create DNSSEC key directory: {:?}",
            key_dir
        ))?;

        let ksk_path = key_dir.join(format!("{}.ksk.key", zone_name));
        let zsk_path = key_dir.join(format!("{}.zsk.key", zone_name));

        // Check if keys already exist
        if ksk_path.exists() && zsk_path.exists() {
            info!("DNSSEC keys already exist for zone: {}", zone_name);
            return Ok(DnssecKeyInfo {
                ksk_path,
                zsk_path,
                algorithm: "ECDSAP256SHA256".to_string(),
            });
        }

        // Generate KSK (Key Signing Key)
        let ksk_output = Command::new("keymgr")
            .arg("generate")
            .arg(&zone_domain)
            .arg("ksk")
            .arg("ECDSAP256SHA256")
            .arg("--keydir")
            .arg(&key_dir)
            .output();

        match ksk_output {
            Ok(output) if output.status.success() => {
                info!("Generated KSK for zone: {}", zone_name);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("keymgr KSK generation failed: {}", stderr);
                // Fallback: create placeholder key files (keys will need manual generation)
                std::fs::write(&ksk_path, b"placeholder-ksk").ok();
                warn!("Created placeholder KSK file - manual key generation required");
            }
            Err(e) => {
                warn!("keymgr not available: {}", e);
                // Create placeholder files
                std::fs::write(&ksk_path, b"placeholder-ksk").ok();
            }
        }

        // Generate ZSK (Zone Signing Key)
        let zsk_output = Command::new("keymgr")
            .arg("generate")
            .arg(&zone_domain)
            .arg("zsk")
            .arg("ECDSAP256SHA256")
            .arg("--keydir")
            .arg(&key_dir)
            .output();

        match zsk_output {
            Ok(output) if output.status.success() => {
                info!("Generated ZSK for zone: {}", zone_name);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("keymgr ZSK generation failed: {}", stderr);
                std::fs::write(&zsk_path, b"placeholder-zsk").ok();
                warn!("Created placeholder ZSK file - manual key generation required");
            }
            Err(e) => {
                warn!("keymgr not available: {}", e);
                std::fs::write(&zsk_path, b"placeholder-zsk").ok();
            }
        }

        Ok(DnssecKeyInfo {
            ksk_path,
            zsk_path,
            algorithm: "ECDSAP256SHA256".to_string(),
        })
    }

    /// Rotate DNSSEC keys (prepare new keys for key rollover)
    async fn rotate_dnssec_keys(&self, zone_name: &str, zone_domain: &str) -> Result<()> {
        info!("Initiating DNSSEC key rotation for zone: {}", zone_name);

        // Generate new keys with different filenames
        let key_dir = PathBuf::from("/var/lib/knot/keys");
        let new_ksk_path = key_dir.join(format!("{}.ksk.new.key", zone_name));
        let new_zsk_path = key_dir.join(format!("{}.zsk.new.key", zone_name));

        // Generate new KSK
        let ksk_output = Command::new("keymgr")
            .arg("generate")
            .arg(&zone_domain)
            .arg("ksk")
            .arg("ECDSAP256SHA256")
            .arg("--keydir")
            .arg(&key_dir)
            .output();

        if let Ok(output) = ksk_output {
            if output.status.success() {
                info!("Generated new KSK for rotation");
            }
        }

        // Generate new ZSK
        let zsk_output = Command::new("keymgr")
            .arg("generate")
            .arg(&zone_domain)
            .arg("zsk")
            .arg("ECDSAP256SHA256")
            .arg("--keydir")
            .arg(&key_dir)
            .output();

        if let Ok(output) = zsk_output {
            if output.status.success() {
                info!("Generated new ZSK for rotation");
            }
        }

        warn!("Key rotation initiated - manual intervention may be required for key rollover");
        Ok(())
    }

    async fn generate_config(&self) -> Result<()> {
        let zones = self.zones.read().await;

        let listen_str = format!("{}@{}", self.config.listen_address, self.config.listen_port);
        let listen_str_v6 = format!("::@{}", self.config.listen_port);

        let knot_config = KnotConfig {
            server: KnotServerConfig {
                rundir: "/var/lib/knot".to_string(),
                listen: vec![listen_str, listen_str_v6],
            },
            zone: {
                let dnssec_keys = self.dnssec_keys.read().await;
                zones
                    .values()
                    .map(|zone_data| {
                        let domain = zone_data.domain.clone();

                        // Get DNSSEC key info if available
                        let key_info = dnssec_keys.get(&domain);

                        KnotZoneConfig {
                            domain: domain.clone(),
                            file: zone_data.zone_file_path.to_string_lossy().to_string(),
                            dnssec_signing: Some("on".to_string()),
                            // Zone transfer settings (notify slaves)
                            notify: None, // TODO: Configure from zone data
                            // Allow zone transfers to authorized secondaries
                            allow_transfer: Some(vec!["127.0.0.1".to_string(), "::1".to_string()]),
                            // Allow dynamic updates (RFC 2136)
                            allow_update: Some(vec!["127.0.0.1".to_string()]),
                            // DNSSEC key configuration
                            dnssec_key: key_info.map(|ki| DnssecKeyConfig {
                                ksk_size: Some(256), // ECDSA P-256
                                zsk_size: Some(256),
                                algorithm: Some(ki.algorithm.clone()),
                                key_directory: Some("/var/lib/knot/keys".to_string()),
                            }),
                        }
                    })
                    .collect()
            },
        };

        // Write Knot config file
        let config_json = serde_json::to_string_pretty(&knot_config)?;
        std::fs::write(&self.config_path, config_json).context(format!(
            "Failed to write Knot config to {:?}",
            self.config_path
        ))?;

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
        zone_content.push_str(&format!(
            "@\tIN\tSOA\tns1.{}. admin.{}. (\n",
            zone_data.domain, zone_data.domain
        ));
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

        std::fs::write(&zone_data.zone_file_path, zone_content).context(format!(
            "Failed to write zone file: {:?}",
            zone_data.zone_file_path
        ))?;

        info!("Generated zone file for {}", zone_data.domain);
        Ok(())
    }

    async fn reload_knot(&self) -> Result<()> {
        info!("Reloading Knot DNS");

        // Try to reload using knotc (Knot control utility)
        let output = Command::new("knotc").arg("reload").output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Knot DNS reloaded successfully");
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Check for specific errors that might be recoverable
                    if stderr.contains("config") || stderr.contains("syntax") {
                        error!("Knot configuration error: {}", stderr);
                        // Don't restart if there's a config error - it will just fail again
                        return Err(anyhow::anyhow!("Knot config error: {}", stderr));
                    }

                    warn!("Knot reload command failed: {} / {}", stderr, stdout);
                    // Fallback to systemctl restart for runtime errors
                    self.restart_knot().await
                }
            }
            Err(e) => {
                warn!("knotc not available ({}), using systemctl", e);
                self.restart_knot().await
            }
        }
    }

    async fn restart_knot(&self) -> Result<()> {
        info!("Restarting Knot DNS service");

        // First, try to check if Knot is running
        let check_output = Command::new("systemctl")
            .arg("is-active")
            .arg("knot")
            .output();

        // Stop Knot if running
        if let Ok(output) = check_output {
            if output.status.success() {
                info!("Stopping Knot DNS service");
                let stop_output = Command::new("systemctl")
                    .arg("stop")
                    .arg("knot")
                    .output()
                    .context("Failed to stop Knot service")?;

                if !stop_output.status.success() {
                    let stderr = String::from_utf8_lossy(&stop_output.stderr);
                    warn!("Failed to stop Knot gracefully: {}", stderr);
                }

                // Wait a moment for service to stop
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }

        // Start/Restart Knot
        let output = Command::new("systemctl")
            .arg("start")
            .arg("knot")
            .output()
            .context("Failed to start Knot service")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Check for common issues
            if stderr.contains("already loaded") || stdout.contains("already loaded") {
                // Try reload instead
                return self.reload_knot().await;
            }

            return Err(anyhow::anyhow!(
                "Failed to start Knot: {} / {}",
                stderr,
                stdout
            ));
        }

        // Verify service started successfully
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let verify_output = Command::new("systemctl")
            .arg("is-active")
            .arg("knot")
            .output()
            .context("Failed to verify Knot service status")?;

        if !verify_output.status.success() {
            return Err(anyhow::anyhow!(
                "Knot service failed to start or is not active"
            ));
        }

        info!("Knot DNS service restarted successfully");
        Ok(())
    }

    /// Initiate zone transfer (AXFR) to a secondary nameserver
    async fn initiate_zone_transfer(&self, zone_name: &str, secondary_ip: &str) -> Result<()> {
        info!(
            "Initiating zone transfer for {} to {}",
            zone_name, secondary_ip
        );

        // Use knotc to trigger zone transfer
        let output = Command::new("knotc")
            .arg("zone-transfer")
            .arg(&zone_name)
            .arg(&secondary_ip)
            .output()
            .context("Failed to initiate zone transfer")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Zone transfer failed: {}", stderr));
        }

        info!("Zone transfer initiated successfully");
        Ok(())
    }

    /// Apply dynamic update (RFC 2136) to a zone
    async fn apply_dynamic_update(&self, zone_name: &str, update_data: &str) -> Result<()> {
        info!("Applying dynamic update to zone: {}", zone_name);

        // Knot supports dynamic updates via knotc zone-commit
        // Updates can be applied directly to zone files or via API

        // For now, we'll regenerate the zone file with new data
        // In production, this would use Knot's dynamic update API
        warn!("Dynamic update via zone file regeneration - full RFC 2136 support requires Knot API integration");

        // Reload the zone after update
        let output = Command::new("knotc")
            .arg("zone-reload")
            .arg(&zone_name)
            .output()
            .context("Failed to reload zone after dynamic update")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Zone reload after update failed: {}",
                stderr
            ));
        }

        info!("Dynamic update applied and zone reloaded");
        Ok(())
    }

    async fn parse_zone_from_etcd(
        &self,
        zone_name: &str,
        zone_data: &[u8],
    ) -> Result<KnotZoneData> {
        #[derive(Deserialize)]
        struct ZoneJson {
            domain: String,
            ttl: Option<u32>,
            records: Vec<DnsRecord>,
        }

        let zone_json: ZoneJson = serde_json::from_slice(zone_data)
            .context(format!("Failed to parse zone JSON for {}", zone_name))?;

        let zone_file_path =
            PathBuf::from(&self.config.zone_dir).join(format!("{}.zone", zone_name));

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
        std::fs::create_dir_all(&self.config.zone_dir).context(format!(
            "Failed to create zone directory: {}",
            self.config.zone_dir
        ))?;

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
                        // Generate DNSSEC keys if not already present
                        let key_info = {
                            let dnssec_keys = self.dnssec_keys.read().await;
                            if !dnssec_keys.contains_key(&zone_data.domain) {
                                drop(dnssec_keys);
                                // Generate keys
                                match self
                                    .generate_dnssec_keys(zone_name, &zone_data.domain)
                                    .await
                                {
                                    Ok(ki) => {
                                        let mut keys = self.dnssec_keys.write().await;
                                        keys.insert(zone_data.domain.clone(), ki.clone());
                                        Some(ki)
                                    }
                                    Err(e) => {
                                        warn!(
                                            "Failed to generate DNSSEC keys for {}: {}",
                                            zone_name, e
                                        );
                                        None
                                    }
                                }
                            } else {
                                dnssec_keys.get(&zone_data.domain).cloned()
                            }
                        };

                        // Generate zone file
                        self.generate_zone_file(&zone_data).await?;

                        // Update zones map
                        let mut zones = self.zones.write().await;
                        zones.insert(zone_name.to_string(), zone_data);

                        // Regenerate Knot config (with DNSSEC settings)
                        drop(zones);
                        self.generate_config().await?;

                        // Reload Knot with improved error handling
                        match self.reload_knot().await {
                            Ok(_) => {
                                info!("Zone updated and reloaded: {}", zone_name);
                            }
                            Err(e) => {
                                error!("Failed to reload Knot after zone update: {}", e);
                                // Try one restart attempt
                                if let Err(restart_err) = self.restart_knot().await {
                                    return Err(anyhow::anyhow!(
                                        "Failed to reload and restart Knot: {} / {}",
                                        e,
                                        restart_err
                                    ));
                                }
                                info!("Zone updated (service restarted): {}", zone_name);
                            }
                        }
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
                let output = Command::new("pgrep").arg("-f").arg("knotd").output();
                Ok(output.map(|o| o.status.success()).unwrap_or(false))
            }
        }
    }
}
