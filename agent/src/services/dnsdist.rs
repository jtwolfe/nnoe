use crate::config::DnsdistServiceConfig;
use crate::plugin::ServicePlugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// dnsdist service integration for DNS filtering and load balancing
pub struct DnsdistService {
    config: DnsdistServiceConfig,
    rules: Arc<RwLock<Vec<DnsdistRule>>>,
    rpz_domains: Arc<RwLock<HashMap<String, String>>>, // domain -> source
    config_path: PathBuf,
    lua_script_path: PathBuf,
}

#[derive(Debug, Clone)]
struct DnsdistRule {
    name: String,
    lua_code: String,
    priority: u32,
}

impl DnsdistService {
    pub fn new(config: DnsdistServiceConfig) -> Self {
        let config_path = PathBuf::from(&config.config_path);
        let lua_script_path = PathBuf::from(&config.lua_script_path);
        Self {
            config,
            rules: Arc::new(RwLock::new(Vec::new())),
            rpz_domains: Arc::new(RwLock::new(HashMap::new())),
            config_path,
            lua_script_path,
        }
    }

    async fn generate_lua_script(&self) -> Result<()> {
        let rules = self.rules.read().await;
        let rpz_domains = self.rpz_domains.read().await;

        // Ensure Lua script directory exists
        if let Some(parent) = self.lua_script_path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create Lua script directory: {:?}", parent))?;
        }

        let mut lua_content = String::from("-- NNOE Generated dnsdist Lua Rules\n");
        lua_content.push_str("-- Auto-generated, do not edit manually\n\n");

        // Add RPZ blocking rules
        if !rpz_domains.is_empty() {
            lua_content.push_str("-- Response Policy Zone (RPZ) Rules\n");
            lua_content.push_str("local rpz_domains = {\n");
            for domain in rpz_domains.keys() {
                lua_content.push_str(&format!("  [\"{}\"] = true,\n", domain));
            }
            lua_content.push_str("}\n\n");

            lua_content.push_str("addLuaAction(AllRule(), function(dq)\n");
            lua_content.push_str("  local qname = dq.qname:toString()\n");
            lua_content.push_str("  if rpz_domains[qname] then\n");
            lua_content.push_str("    return DNSAction.Drop\n");
            lua_content.push_str("  end\n");
            lua_content.push_str("  return DNSAction.None\n");
            lua_content.push_str("end)\n\n");
        }

        // Add custom rules (sorted by priority)
        let mut sorted_rules = rules.clone();
        sorted_rules.sort_by_key(|r| r.priority);

        for rule in sorted_rules {
            lua_content.push_str(&format!("-- Rule: {}\n", rule.name));
            lua_content.push_str(&rule.lua_code);
            lua_content.push_str("\n\n");
        }

        // Add anomaly detection rule placeholder
        lua_content.push_str("-- Anomaly Detection Rule\n");
        lua_content.push_str("addLuaAction(AllRule(), function(dq)\n");
        lua_content.push_str("  -- Anomaly detection logic will be implemented\n");
        lua_content.push_str("  -- if isAnomalous(dq) then return DNSAction.Drop end\n");
        lua_content.push_str("  return DNSAction.None\n");
        lua_content.push_str("end)\n");

        std::fs::write(&self.lua_script_path, lua_content)
            .context(format!("Failed to write Lua script to {:?}", self.lua_script_path))?;

        info!("Generated dnsdist Lua script with {} rules and {} RPZ domains", 
              rules.len(), rpz_domains.len());
        Ok(())
    }

    async fn generate_config(&self) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create config directory: {:?}", parent))?;
        }

        let mut config_content = String::from("# NNOE Generated dnsdist Configuration\n");
        config_content.push_str("# Auto-generated, do not edit manually\n\n");
        config_content.push_str(&format!("setKey(\"nnoe-dnsdist-key\")\n"));
        config_content.push_str(&format!("controlSocket(\"127.0.0.1:5199\")\n"));
        config_content.push_str(&format!("setLocal(\"0.0.0.0:53\")\n\n"));
        
        // Add Lua script reference
        config_content.push_str(&format!(
            "addLuaAction(AllRule(), LoadString(\"{}\"))\n",
            self.lua_script_path.to_string_lossy()
        ));

        // Add upstream resolvers (placeholder)
        config_content.push_str("\n# Upstream resolvers\n");
        config_content.push_str("newServer({address=\"127.0.0.1:5353\", name=\"local\"})\n");
        config_content.push_str("newServer({address=\"8.8.8.8\", name=\"google\"})\n");

        std::fs::write(&self.config_path, config_content)
            .context(format!("Failed to write dnsdist config to {:?}", self.config_path))?;

        info!("Generated dnsdist config");
        Ok(())
    }

    async fn reload_dnsdist(&self) -> Result<()> {
        info!("Reloading dnsdist");
        
        // Try to reload using dnsdist control channel
        let output = Command::new("dnsdist")
            .arg("-C")
            .arg(&self.config_path)
            .arg("reload")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("dnsdist reloaded successfully");
                } else {
                    warn!("dnsdist reload command failed, trying systemctl");
                    return self.restart_dnsdist().await;
                }
            }
            Err(e) => {
                warn!("dnsdist control not available, using systemctl: {}", e);
                return self.restart_dnsdist().await;
            }
        }

        Ok(())
    }

    async fn restart_dnsdist(&self) -> Result<()> {
        info!("Restarting dnsdist service");
        
        let output = Command::new("systemctl")
            .arg("restart")
            .arg("dnsdist")
            .output()
            .context("Failed to restart dnsdist service")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to restart dnsdist: {}", stderr));
        }

        info!("dnsdist service restarted");
        Ok(())
    }

    async fn update_rpz_from_threats(&self, threats: &HashMap<String, String>) -> Result<()> {
        let mut rpz = self.rpz_domains.write().await;
        rpz.clear();
        for (domain, source) in threats {
            rpz.insert(domain.clone(), source.clone());
        }
        info!("Updated RPZ with {} threat domains", rpz.len());
        Ok(())
    }
}

#[async_trait]
impl ServicePlugin for DnsdistService {
    fn name(&self) -> &str {
        "dnsdist"
    }

    async fn init(&mut self, _config: &[u8]) -> Result<()> {
        info!("Initializing dnsdist service");
        info!("Config path: {:?}", self.config_path);
        info!("Lua script path: {:?}", self.lua_script_path);

        // Generate initial config and Lua script
        self.generate_lua_script().await?;
        self.generate_config().await?;

        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
        if key.contains("/threats/domains/") {
            // Update RPZ with threat domain
            let parts: Vec<&str> = key.split('/').collect();
            if let Some(domain) = parts.last() {
                // Parse threat JSON
                #[derive(serde::Deserialize)]
                struct ThreatData {
                    domain: String,
                    source: String,
                }
                
                match serde_json::from_slice::<ThreatData>(value) {
                    Ok(threat) => {
                        let mut rpz = self.rpz_domains.write().await;
                        rpz.insert(threat.domain.clone(), threat.source);
                        drop(rpz);
                        
                        self.generate_lua_script().await?;
                        self.reload_dnsdist().await?;
                        
                        info!("Threat domain added to RPZ: {}", threat.domain);
                    }
                    Err(e) => {
                        error!("Failed to parse threat data: {}", e);
                    }
                }
            }
        } else if key.contains("/policies/") {
            // Policy-based rules would be processed here
            // Convert Cerbos policies to dnsdist Lua rules
            debug!("Policy change detected, would generate dnsdist rule");
        }

        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        info!("Reloading dnsdist service");
        self.generate_lua_script().await?;
        self.generate_config().await?;
        self.reload_dnsdist().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down dnsdist service");
        let mut rules = self.rules.write().await;
        rules.clear();
        let mut rpz = self.rpz_domains.write().await;
        rpz.clear();
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if dnsdist is running
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg("dnsdist")
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => {
                // Fallback: try to check if dnsdist process exists
                let output = Command::new("pgrep")
                    .arg("-f")
                    .arg("dnsdist")
                    .output();
                Ok(output.map(|o| o.status.success()).unwrap_or(false))
            }
        }
    }
}

