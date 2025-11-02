use crate::config::DhcpServiceConfig;
use crate::plugin::ServicePlugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct KeaConfig {
    Dhcp4: KeaDhcp4Config,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaDhcp4Config {
    #[serde(rename = "interfaces-config")]
    interfaces_config: KeaInterfacesConfig,
    #[serde(rename = "lease-database")]
    lease_database: KeaLeaseDatabase,
    subnet4: Vec<KeaSubnet>,
    #[serde(rename = "hooks-libraries", default)]
    hooks_libraries: Vec<KeaHook>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaInterfacesConfig {
    interfaces: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaLeaseDatabase {
    #[serde(rename = "type")]
    db_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaSubnet {
    subnet: String,
    pools: Vec<KeaPool>,
    #[serde(default)]
    option_data: Vec<KeaOption>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaPool {
    pool: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaOption {
    name: String,
    data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeaHook {
    library: String,
}

/// Kea DHCP service integration
pub struct KeaService {
    config: DhcpServiceConfig,
    scopes: Arc<RwLock<HashMap<String, KeaScopeData>>>,
    config_path: PathBuf,
}

#[derive(Debug, Clone)]
struct KeaScopeData {
    subnet: String,
    pool_start: String,
    pool_end: String,
    gateway: Option<String>,
    dns_servers: Vec<String>,
    options: HashMap<String, String>,
}

impl KeaService {
    pub fn new(config: DhcpServiceConfig) -> Self {
        let config_path = PathBuf::from(&config.config_path);
        Self {
            config,
            scopes: Arc::new(RwLock::new(HashMap::new())),
            config_path,
        }
    }

    async fn generate_config(&self) -> Result<()> {
        let scopes = self.scopes.read().await;

        let kea_config = KeaConfig {
            Dhcp4: KeaDhcp4Config {
                interfaces_config: KeaInterfacesConfig {
                    interfaces: vec![self.config.interface.clone()],
                },
                lease_database: KeaLeaseDatabase {
                    db_type: "memfile".to_string(),
                },
                subnet4: scopes
                    .values()
                    .map(|scope| KeaSubnet {
                        subnet: scope.subnet.clone(),
                        pools: vec![KeaPool {
                            pool: format!("{} - {}", scope.pool_start, scope.pool_end),
                        }],
                        option_data: {
                            let mut options = Vec::new();
                            if let Some(ref gateway) = scope.gateway {
                                options.push(KeaOption {
                                    name: "routers".to_string(),
                                    data: gateway.clone(),
                                });
                            }
                            if !scope.dns_servers.is_empty() {
                                options.push(KeaOption {
                                    name: "domain-name-servers".to_string(),
                                    data: scope.dns_servers.join(", "),
                                });
                            }
                            for (name, value) in &scope.options {
                                options.push(KeaOption {
                                    name: name.clone(),
                                    data: value.clone(),
                                });
                            }
                            options
                        },
                    })
                    .collect(),
                hooks_libraries: vec![
                    // Custom hook for etcd sync would go here
                    // KeaHook { library: "/usr/lib/kea/hooks/libdhcp_etcd.so".to_string() }
                ],
            },
        };

        // Write Kea config file
        let config_json = serde_json::to_string_pretty(&kea_config)?;
        std::fs::write(&self.config_path, config_json).context(format!(
            "Failed to write Kea config to {:?}",
            self.config_path
        ))?;

        info!("Generated Kea config with {} scopes", scopes.len());
        Ok(())
    }

    async fn reload_kea(&self) -> Result<()> {
        info!("Reloading Kea DHCP");

        // Try to reload using kea-shell (Kea control channel)
        let output = Command::new("kea-shell")
            .arg("--host")
            .arg("localhost")
            .arg("--port")
            .arg(&self.config.control_port.to_string())
            .arg("--service")
            .arg("dhcp4")
            .arg("config-reload")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Kea DHCP reloaded successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Kea reload command failed: {}", stderr);
                    // Fallback to systemctl restart
                    return self.restart_kea().await;
                }
            }
            Err(e) => {
                warn!("kea-shell not available, using systemctl: {}", e);
                return self.restart_kea().await;
            }
        }

        Ok(())
    }

    async fn restart_kea(&self) -> Result<()> {
        info!("Restarting Kea DHCP service");

        let output = Command::new("systemctl")
            .arg("restart")
            .arg("kea-dhcp4")
            .output()
            .context("Failed to restart Kea service")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to restart Kea: {}", stderr));
        }

        info!("Kea DHCP service restarted");
        Ok(())
    }

    async fn parse_scope_from_etcd(
        &self,
        scope_id: &str,
        scope_data: &[u8],
    ) -> Result<KeaScopeData> {
        #[derive(Deserialize)]
        struct ScopeJson {
            subnet: String,
            pool: PoolConfig,
            #[serde(default)]
            gateway: Option<String>,
            #[serde(default)]
            options: HashMap<String, String>,
        }

        #[derive(Deserialize)]
        struct PoolConfig {
            start: String,
            end: String,
        }

        let scope_json: ScopeJson = serde_json::from_slice(scope_data)
            .context(format!("Failed to parse scope JSON for {}", scope_id))?;

        // Extract DNS servers from options if present
        let dns_servers = scope_json
            .options
            .get("dns-servers")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["8.8.8.8".to_string()]); // Default DNS

        Ok(KeaScopeData {
            subnet: scope_json.subnet,
            pool_start: scope_json.pool.start,
            pool_end: scope_json.pool.end,
            gateway: scope_json.gateway,
            dns_servers,
            options: scope_json.options,
        })
    }

    async fn handle_ha_coordination(&self) -> Result<()> {
        if self.config.ha_pair_id.is_none() {
            return Ok(()); // Not in HA mode
        }

        let ha_pair_id = self.config.ha_pair_id.as_ref().unwrap();
        debug!("HA pair coordination for: {}", ha_pair_id);

        // Check VIP status
        let has_vip = self.check_vip().await.unwrap_or(false);
        self.has_vip.store(has_vip, Ordering::Release);

        // Check peer status
        let peer_state = self.check_peer_status().await?;

        let mut ha_state_guard = self.ha_state.write().await;
        let mut service_running_guard = self.service_running.write().await;

        // Determine our state
        let new_state = if has_vip {
            HaState::Primary
        } else if peer_state == Some(HaState::Primary) {
            HaState::Standby
        } else {
            // No clear primary, try to become primary if we don't see a peer
            if peer_state.is_none() {
                HaState::Primary // Assume primary if no peer seen
            } else {
                HaState::Standby
            }
        };

        let state_changed = *ha_state_guard != new_state;
        *ha_state_guard = new_state;

        // Start/stop service based on HA state
        match *ha_state_guard {
            HaState::Primary => {
                if !*service_running_guard {
                    info!("Becoming primary, starting Kea service");
                    self.start_kea().await?;
                    *service_running_guard = true;
                }
            }
            HaState::Standby => {
                if *service_running_guard {
                    info!("Becoming standby, stopping Kea service");
                    self.stop_kea().await?;
                    *service_running_guard = false;
                }
            }
            HaState::Unknown => {
                warn!("HA state unknown, keeping service as-is");
            }
        }

        drop(service_running_guard);
        drop(ha_state_guard);

        // Update status in etcd
        self.update_ha_status_in_etcd().await?;

        if state_changed {
            info!("HA state changed to: {:?}", new_state);
        }

        Ok(())
    }

    async fn start_kea(&self) -> Result<()> {
        // Start Kea DHCP service
        let output = Command::new("systemctl")
            .arg("start")
            .arg("kea-dhcp4")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                info!("Kea DHCP service started");
                Ok(())
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to start Kea: {}", stderr))
            }
            Err(e) => Err(anyhow::anyhow!("Failed to execute systemctl: {}", e)),
        }
    }

    async fn stop_kea(&self) -> Result<()> {
        // Stop Kea DHCP service
        let output = Command::new("systemctl")
            .arg("stop")
            .arg("kea-dhcp4")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                info!("Kea DHCP service stopped");
                Ok(())
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to stop Kea gracefully: {}", stderr);
                Ok(()) // Non-fatal
            }
            Err(e) => {
                warn!("Failed to execute systemctl: {}", e);
                Ok(()) // Non-fatal
            }
        }
    }
}

#[async_trait]
impl ServicePlugin for KeaService {
    fn name(&self) -> &str {
        "kea-dhcp"
    }

    async fn init(&mut self, _config: &[u8]) -> Result<()> {
        info!("Initializing Kea DHCP service");
        info!("Config path: {:?}", self.config_path);

        if let Some(ref ha_pair_id) = self.config.ha_pair_id {
            info!("HA pair ID: {}", ha_pair_id);
        }

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create config directory: {:?}", parent))?;
        }

        // Handle HA coordination
        self.handle_ha_coordination().await?;

        // Generate initial config
        self.generate_config().await?;

        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
        if key.contains("/dhcp/scopes/") {
            // Extract scope ID from key (format: /nnoe/dhcp/scopes/<scope-id>)
            let parts: Vec<&str> = key.split('/').collect();
            if let Some(scope_id) = parts.last() {
                match self.parse_scope_from_etcd(scope_id, value).await {
                    Ok(scope_data) => {
                        // Update scopes map
                        let mut scopes = self.scopes.write().await;
                        scopes.insert(scope_id.to_string(), scope_data);
                        drop(scopes);

                        // Regenerate Kea config
                        self.generate_config().await?;

                        // Reload Kea
                        self.reload_kea().await?;

                        info!("DHCP scope updated: {}", scope_id);
                    }
                    Err(e) => {
                        error!("Failed to parse scope {}: {}", scope_id, e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        info!("Reloading Kea DHCP service");
        self.generate_config().await?;
        self.reload_kea().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Kea DHCP service");
        let mut scopes = self.scopes.write().await;
        scopes.clear();
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if Kea is running
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg("kea-dhcp4")
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => {
                // Fallback: try to check if kea process exists
                let output = Command::new("pgrep").arg("-f").arg("kea-dhcp4").output();
                Ok(output.map(|o| o.status.success()).unwrap_or(false))
            }
        }
    }
}
