use crate::config::DnsdistServiceConfig;
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
            std::fs::create_dir_all(parent).context(format!(
                "Failed to create Lua script directory: {:?}",
                parent
            ))?;
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

        std::fs::write(&self.lua_script_path, lua_content).context(format!(
            "Failed to write Lua script to {:?}",
            self.lua_script_path
        ))?;

        info!(
            "Generated dnsdist Lua script with {} rules and {} RPZ domains",
            rules.len(),
            rpz_domains.len()
        );
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
        config_content.push_str(&format!(
            "controlSocket(\"127.0.0.1:{}\")\n",
            self.config.control_port
        ));
        config_content.push_str(&format!(
            "setLocal(\"{}:{}\")\n\n",
            self.config.listen_address, self.config.listen_port
        ));

        // Add Lua script reference
        config_content.push_str(&format!(
            "addLuaAction(AllRule(), LoadString(\"{}\"))\n",
            self.lua_script_path.to_string_lossy()
        ));

        // Add upstream resolvers
        config_content.push_str("\n# Upstream resolvers\n");
        if self.config.upstream_resolvers.is_empty() {
            // Default upstreams if none configured
            config_content.push_str("newServer({address=\"127.0.0.1:5353\", name=\"local\"})\n");
            config_content.push_str("newServer({address=\"8.8.8.8\", name=\"google\"})\n");
        } else {
            for resolver in &self.config.upstream_resolvers {
                config_content.push_str(&format!(
                    "newServer({{address=\"{}\", name=\"{}\"}})\n",
                    resolver,
                    resolver.split(':').next().unwrap_or("resolver")
                ));
            }
        }

        std::fs::write(&self.config_path, config_content).context(format!(
            "Failed to write dnsdist config to {:?}",
            self.config_path
        ))?;

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

    async fn process_cerbos_policy(&self, key: &str, policy_data: &[u8]) -> Result<()> {
        debug!("Processing Cerbos policy from key: {}", key);

        #[derive(Debug, Deserialize)]
        struct CerbosPolicy {
            #[serde(rename = "apiVersion")]
            api_version: Option<String>,
            resource_policy: Option<ResourcePolicy>,
        }

        #[derive(Debug, Deserialize)]
        struct ResourcePolicy {
            version: String,
            resource: String,
            rules: Vec<PolicyRule>,
        }

        #[derive(Debug, Deserialize)]
        struct PolicyRule {
            actions: Vec<String>,
            effect: String,
            roles: Vec<String>,
            condition: Option<PolicyCondition>,
        }

        #[derive(Debug, Deserialize)]
        struct PolicyCondition {
            #[serde(rename = "match")]
            match_expr: Option<MatchExpr>,
        }

        #[derive(Debug, Deserialize)]
        struct MatchExpr {
            expr: Option<String>,
        }

        // Try to parse as YAML first, then JSON
        let policy: CerbosPolicy = if let Ok(p) = serde_yaml::from_slice(policy_data) {
            p
        } else {
            serde_json::from_slice(policy_data).context("Failed to parse policy as YAML or JSON")?
        };

        // Only process DNS-related policies
        if let Some(ref resource_policy) = policy.resource_policy {
            if resource_policy.resource != "dns_query" {
                debug!("Skipping non-DNS policy: {}", resource_policy.resource);
                return Ok(());
            }

            let mut rules = self.rules.write().await;

            // Extract policy ID from key
            let policy_id = key.split('/').last().unwrap_or("unknown");

            // Convert each rule to Lua
            for (idx, rule) in resource_policy.rules.iter().enumerate() {
                if rule.effect != "EFFECT_ALLOW" && rule.actions.contains(&"allow".to_string()) {
                    continue; // Skip deny rules for now, or handle them differently
                }

                let lua_code = self.cerbos_rule_to_lua(rule, policy_id, idx)?;

                rules.push(DnsdistRule {
                    name: format!("cerbos_{}_{}", policy_id, idx),
                    lua_code,
                    priority: (1000 + idx) as u32, // Higher priority than RPZ
                });
            }

            info!(
                "Converted {} rules from Cerbos policy {}",
                resource_policy.rules.len(),
                policy_id
            );

            drop(rules);

            // Regenerate Lua script
            self.generate_lua_script().await?;
            self.reload_dnsdist().await?;
        }

        Ok(())
    }

    fn cerbos_rule_to_lua(
        &self,
        rule: &serde_json::Value,
        policy_id: &str,
        rule_idx: usize,
    ) -> Result<String> {
        // This is a simplified parser - in production, use a proper expression parser
        let mut lua = String::from("addLuaAction(AllRule(), function(dq)\n");
        lua.push_str("  local qname = dq.qname:toString()\n");
        lua.push_str("  local current_time = os.time(os.date(\"*t\"))\n");
        lua.push_str("  local current_hour = tonumber(os.date(\"%H\", current_time))\n");

        // Extract roles
        if let Some(roles) = rule.get("roles").and_then(|r| r.as_array()) {
            let role_checks: Vec<String> = roles
                .iter()
                .filter_map(|r| r.as_str())
                .map(|r| format!("role == \"{}\"", r))
                .collect();

            if !role_checks.is_empty() {
                lua.push_str(
                    "  -- Role-based check (placeholder - role would come from request context)\n",
                );
                lua.push_str("  local role = \"user\" -- TODO: Extract from request\n");
                lua.push_str(&format!(
                    "  local has_role = ({})\n",
                    role_checks.join(" or ")
                ));
            }
        }

        // Extract condition expressions
        if let Some(condition) = rule.get("condition") {
            if let Some(match_expr) = condition.get("match") {
                if let Some(expr) = match_expr.get("expr").and_then(|e| e.as_str()) {
                    // Convert Cerbos expression to Lua
                    let lua_expr = self.convert_cerbos_expr_to_lua(expr);
                    lua.push_str(&format!("  local condition_result = {}\n", lua_expr));
                    lua.push_str("  if not condition_result then\n");
                    lua.push_str("    return DNSAction.None\n");
                    lua.push_str("  end\n");
                }
            }
        }

        // Extract domain checks from expressions
        let expr = rule
            .get("condition")
            .and_then(|c| c.get("match"))
            .and_then(|m| m.get("expr"))
            .and_then(|e| e.as_str());

        if let Some(expr_str) = expr {
            // Check for domain.contains checks
            if expr_str.contains("domain.contains") {
                if expr_str.contains("malicious") {
                    lua.push_str("  if string.find(qname, \"malicious\") then\n");
                    lua.push_str("    return DNSAction.Drop\n");
                    lua.push_str("  end\n");
                }
                if expr_str.contains("blocked") {
                    lua.push_str("  if string.find(qname, \"blocked\") then\n");
                    lua.push_str("    return DNSAction.Drop\n");
                    lua.push_str("  end\n");
                }
            }
        }

        // Time-based conditions
        if let Some(expr_str) = expr {
            if expr_str.contains("time.hour") {
                if expr_str.contains("< 18") {
                    lua.push_str("  if current_hour >= 18 then\n");
                    lua.push_str("    return DNSAction.Drop\n");
                    lua.push_str("  end\n");
                }
            }
        }

        lua.push_str("  return DNSAction.None\n");
        lua.push_str("end)\n");

        Ok(lua)
    }

    fn convert_cerbos_expr_to_lua(&self, expr: &str) -> String {
        // Simple expression converter - in production, use a proper parser
        let mut lua = expr.to_string();

        // Replace common patterns
        lua = lua.replace("request.time.hour", "current_hour");
        lua = lua.replace("request.domain.contains", "string.find(qname");
        lua = lua.replace("!", "not ");
        lua = lua.replace("&&", " and ");
        lua = lua.replace("||", " or ");

        // Handle contains with proper string matching
        if lua.contains("string.find(qname") {
            lua = lua.replace("\"", "'");
            // Close the find call
            if !lua.contains(")") {
                lua = format!("{} ~= nil", lua);
            }
        }

        lua
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
            // Parse Cerbos policy and convert to dnsdist Lua rule
            if let Err(e) = self.process_cerbos_policy(key, value).await {
                error!("Failed to process Cerbos policy: {}", e);
            }
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
                let output = Command::new("pgrep").arg("-f").arg("dnsdist").output();
                Ok(output.map(|o| o.status.success()).unwrap_or(false))
            }
        }
    }
}
