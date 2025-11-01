use crate::config::NebulaConfig;
use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct NebulaManager {
    config: NebulaConfig,
    process: Arc<RwLock<Option<Child>>>,
}

impl NebulaManager {
    pub async fn new(config: &NebulaConfig) -> Result<Self> {
        info!("Initializing Nebula manager");

        if config.config_path.is_none() {
            return Err(anyhow::anyhow!(
                "Nebula config_path is required when enabled"
            ));
        }

        Ok(Self {
            config: config.clone(),
            process: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut process_guard = self.process.write().await;

        if process_guard.is_some() {
            warn!("Nebula process already running");
            return Ok(());
        }

        let config_path = self
            .config
            .config_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Nebula config_path not set"))?;

        info!("Starting Nebula with config: {}", config_path);

        let mut cmd = Command::new("nebula");
        cmd.arg("-config")
            .arg(config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().context("Failed to start Nebula process")?;

        *process_guard = Some(child);
        info!("Nebula process started");

        // Spawn a task to monitor the process
        let process_clone = Arc::clone(&self.process);
        tokio::spawn(async move {
            Self::monitor_process(process_clone).await;
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut process_guard = self.process.write().await;

        if let Some(mut child) = process_guard.take() {
            info!("Stopping Nebula process");
            child.kill().context("Failed to kill Nebula process")?;
            child.wait()?;
            info!("Nebula process stopped");
        }

        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        // This is a simple check - in production, we might want to check the process status
        // For now, we'll just check if we have a process handle
        // Note: This is async-safe but the check itself is sync
        // In a real implementation, we'd use a shared atomic boolean
        false // Placeholder - will be improved
    }

    async fn monitor_process(process: Arc<RwLock<Option<Child>>>) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let process_guard = process.read().await;
            if let Some(ref mut child) = *process_guard {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        error!("Nebula process exited with status: {:?}", status);
                        drop(process_guard);
                        let mut write_guard = process.write().await;
                        *write_guard = None;
                        break;
                    }
                    Ok(None) => {
                        // Process still running
                        continue;
                    }
                    Err(e) => {
                        error!("Error checking Nebula process status: {}", e);
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
}

impl Drop for NebulaManager {
    fn drop(&mut self) {
        // Note: Drop is sync, but we need async cleanup
        // In production, we'd use a blocking runtime or handle this differently
        warn!("NebulaManager dropped - process cleanup should be handled manually");
    }
}

