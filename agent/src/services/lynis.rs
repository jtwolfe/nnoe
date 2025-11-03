use crate::config::LynisServiceConfig;
use crate::plugin::ServicePlugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Lynis security auditing service integration
pub struct LynisService {
    config: LynisServiceConfig,
    node_id: Arc<RwLock<Option<String>>>,
    last_audit: Arc<RwLock<Option<std::time::SystemTime>>>,
    audit_interval: Duration,
    etcd_client: Arc<RwLock<Option<Arc<crate::etcd::EtcdClient>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LynisReport {
    node: String,
    timestamp: String,
    score: Option<u32>,
    warnings: Vec<String>,
    suggestions: Vec<String>,
    sections: HashMap<String, LynisSection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LynisSection {
    score: Option<u32>,
    status: String,
    items: Vec<LynisItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LynisItem {
    plugin: String,
    option: String,
    status: String,
    message: Option<String>,
}

impl LynisService {
    pub fn new(config: LynisServiceConfig, node_id: Option<String>) -> Self {
        let audit_interval = Duration::from_secs(config.audit_interval_secs);
        Self {
            config,
            node_id: Arc::new(RwLock::new(node_id)),
            last_audit: Arc::new(RwLock::new(None)),
            audit_interval,
            etcd_client: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_etcd_client(&self, client: Arc<crate::etcd::EtcdClient>) {
        let mut etcd_guard = self.etcd_client.write().await;
        *etcd_guard = Some(client);
    }

    async fn run_audit(&self) -> Result<LynisReport> {
        info!("Running Lynis security audit");

        // Check if lynis is available
        let lynis_output = Command::new("lynis")
            .arg("audit")
            .arg("system")
            .arg("--quiet")
            .arg("--report-file")
            .arg(&self.config.report_path)
            .output()
            .context("Failed to execute Lynis audit")?;

        if !lynis_output.status.success() {
            let stderr = String::from_utf8_lossy(&lynis_output.stderr);
            return Err(anyhow::anyhow!("Lynis audit failed: {}", stderr));
        }

        // Parse Lynis report file
        let report = self.parse_lynis_report().await?;

        info!("Lynis audit completed with score: {:?}", report.score);
        Ok(report)
    }

    /// Parse Lynis report file and extract structured data
    async fn parse_lynis_report(&self) -> Result<LynisReport> {
        let report_path = PathBuf::from(&self.config.report_path);

        // Read report file
        let report_content = fs::read_to_string(&report_path)
            .context(format!("Failed to read Lynis report: {:?}", report_path))?;

        let node_id = self
            .node_id
            .read()
            .await
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let timestamp = chrono::Utc::now().to_rfc3339();

        let mut score: Option<u32> = None;
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        let mut sections = HashMap::new();

        // Parse hardening index (score)
        // Pattern: "Hardening index : [XX]" or "Hardening index : XX"
        let score_regex = Regex::new(r#"Hardening\s+index\s*[=:]\s*\[?(\d+)\]?"#).unwrap();
        if let Some(captures) = score_regex.captures(&report_content) {
            if let Ok(parsed_score) = captures.get(1).unwrap().as_str().parse::<u32>() {
                score = Some(parsed_score);
            }
        }

        // Parse warnings
        // Pattern: "[WARNING]" followed by text
        let warning_regex = Regex::new(r#"\[WARNING\]\s*(.+)"#).unwrap();
        for cap in warning_regex.captures_iter(&report_content) {
            if let Some(warning_text) = cap.get(1) {
                let warning = warning_text.as_str().trim().to_string();
                if !warning.is_empty() {
                    warnings.push(warning);
                }
            }
        }

        // Parse suggestions
        // Pattern: "[SUGGESTION]" followed by text
        let suggestion_regex = Regex::new(r#"\[SUGGESTION\]\s*(.+)"#).unwrap();
        for cap in suggestion_regex.captures_iter(&report_content) {
            if let Some(suggestion_text) = cap.get(1) {
                let suggestion = suggestion_text.as_str().trim().to_string();
                if !suggestion.is_empty() {
                    suggestions.push(suggestion);
                }
            }
        }

        // Parse sections (test categories)
        // Pattern: "[+] <Section Name>" followed by test items
        let section_regex = Regex::new(r#"\[\+\]\s+([^\[\n]+)"#).unwrap();
        let mut current_section: Option<String> = None;
        let mut current_items = Vec::new();

        for line in report_content.lines() {
            // Check for section headers
            if let Some(cap) = section_regex.captures(line) {
                // Save previous section if exists
                if let Some(section_name) = current_section.take() {
                    if !current_items.is_empty() {
                        sections.insert(
                            section_name.clone(),
                            LynisSection {
                                score: None, // Section scores not typically in report
                                status: "completed".to_string(),
                                items: current_items.clone(),
                            },
                        );
                    }
                    current_items.clear();
                }

                // Start new section
                let section_name = cap.get(1).unwrap().as_str().trim().to_string();
                current_section = Some(section_name);
            }

            // Parse test items within sections
            // Pattern: "  - [X]" or "  - [NO]" or "  - [OK]" etc.
            if current_section.is_some() {
                let item_regex = Regex::new(r#"^\s+- \[([A-Z_]+)\]\s+(.+)"#).unwrap();
                if let Some(cap) = item_regex.captures(line) {
                    let status = cap.get(1).unwrap().as_str().to_string();
                    let message = cap.get(2).map(|m| m.as_str().trim().to_string());

                    // Extract plugin/option from message (if format: "Plugin-Name: Option-Name")
                    let (plugin, option) = if let Some(msg) = &message {
                        if let Some((p, o)) = msg.split_once(':') {
                            (p.trim().to_string(), o.trim().to_string())
                        } else {
                            ("unknown".to_string(), msg.clone())
                        }
                    } else {
                        ("unknown".to_string(), "unknown".to_string())
                    };

                    current_items.push(LynisItem {
                        plugin,
                        option,
                        status,
                        message,
                    });
                }
            }
        }

        // Save last section
        if let Some(section_name) = current_section {
            if !current_items.is_empty() {
                sections.insert(
                    section_name,
                    LynisSection {
                        score: None,
                        status: "completed".to_string(),
                        items: current_items,
                    },
                );
            }
        }

        Ok(LynisReport {
            node: node_id,
            timestamp,
            score,
            warnings,
            suggestions,
            sections,
        })
    }

    async fn upload_report_to_etcd(&self, report: &LynisReport) -> Result<()> {
        let etcd_guard = self.etcd_client.read().await;
        if let Some(ref client) = *etcd_guard {
            let report_json = serde_json::to_string(report)?;
            let key = format!("/nnoe/audit/lynis/{}", report.node);

            client.put(&key, report_json.as_bytes()).await?;
            info!("Uploaded Lynis report to etcd: {}", key);
        } else {
            warn!("etcd client not available, skipping report upload");
        }
        Ok(())
    }

    async fn start_periodic_audits(&self) {
        let audit_interval = self.audit_interval;
        let etcd_client = Arc::clone(&self.etcd_client);
        let last_audit = Arc::clone(&self.last_audit);
        let config = self.config.clone();
        let node_id = Arc::clone(&self.node_id);

        tokio::spawn(async move {
            let mut interval_timer = interval(audit_interval);

            loop {
                interval_timer.tick().await;

                // Create a temporary service instance for the audit
                let report = {
                    let node_id_guard = node_id.read().await;
                    let node_id_str = node_id_guard
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    drop(node_id_guard);

                    // Run audit
                    let lynis_output = Command::new("lynis")
                        .arg("audit")
                        .arg("system")
                        .arg("--quiet")
                        .arg("--report-file")
                        .arg(&config.report_path)
                        .output();

                    match lynis_output {
                        Ok(output) if output.status.success() => {
                            // Parse the report file
                            // Create a temporary service instance with minimal setup
                            let service =
                                LynisService::new(config.clone(), Some(node_id_str.clone()));
                            // Set the node_id for parsing
                            {
                                let mut node_id_guard = service.node_id.write().await;
                                *node_id_guard = Some(node_id_str.clone());
                            }
                            service.parse_lynis_report().await
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            Err(anyhow::anyhow!("Lynis audit failed: {}", stderr))
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to execute Lynis: {}", e)),
                    }
                };

                match report {
                    Ok(report) => {
                        // Upload to etcd if available
                        let etcd_guard = etcd_client.read().await;
                        if let Some(ref client) = *etcd_guard {
                            match serde_json::to_string(&report) {
                                Ok(report_json) => {
                                    let key = format!("/nnoe/audit/lynis/{}", report.node);
                                    match client.put(&key, report_json.as_bytes()).await {
                                        Ok(_) => {
                                            info!("Uploaded Lynis report to etcd: {}", key);
                                        }
                                        Err(e) => {
                                            error!("Failed to upload Lynis report: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to serialize report: {}", e);
                                }
                            }
                        }
                        drop(etcd_guard);

                        // Update last audit time
                        let mut last_audit_guard = last_audit.write().await;
                        *last_audit_guard = Some(std::time::SystemTime::now());
                    }
                    Err(e) => {
                        error!("Lynis audit failed: {}", e);
                    }
                }
            }
        });
    }
}

#[async_trait]
impl ServicePlugin for LynisService {
    fn name(&self) -> &str {
        "lynis"
    }

    async fn init(&mut self, config: &[u8]) -> Result<()> {
        info!("Initializing Lynis service");
        info!(
            "Audit interval: {} seconds",
            self.config.audit_interval_secs
        );
        info!("Report path: {}", self.config.report_path);

        // Parse node ID from config if provided
        #[derive(Deserialize)]
        struct InitConfig {
            node_id: Option<String>,
        }

        if let Ok(init_config) = serde_json::from_slice::<InitConfig>(config) {
            if let Some(node_id) = init_config.node_id {
                let mut stored_id = self.node_id.write().await;
                *stored_id = Some(node_id);
            }
        }

        // Ensure report directory exists
        if let Some(parent) = PathBuf::from(&self.config.report_path).parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create report directory: {:?}", parent))?;
        }

        // Check if lynis is installed
        match Command::new("lynis").arg("--version").output() {
            Ok(_) => info!("Lynis is installed and available"),
            Err(_) => warn!("Lynis not found in PATH - audits will fail"),
        }

        // Start periodic audits task
        // Note: etcd_client should be set before this is called
        self.start_periodic_audits().await;
        info!("Lynis periodic audit task started");

        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, _value: &[u8]) -> Result<()> {
        if key.contains("/audit/lynis/config") {
            // Configuration change would trigger immediate audit if needed
            debug!("Lynis config changed, would trigger audit if needed");
        }
        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        info!("Reloading Lynis service (triggering immediate audit)");
        // Trigger immediate audit on reload
        let _report = self.run_audit().await?;

        // Upload report (would need etcd client passed in)
        // For now, just log success
        info!("Audit report generated: {}", self.config.report_path);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Lynis service");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if lynis is available
        let output = Command::new("lynis").arg("--version").output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false), // Lynis not available
        }
    }
}
