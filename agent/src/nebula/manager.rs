use crate::config::NebulaConfig;
use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct NebulaManager {
    config: NebulaConfig,
    process: Arc<RwLock<Option<Child>>>,
    is_running_flag: Arc<AtomicBool>,
    restart_count: Arc<std::sync::Mutex<u32>>,
    max_restarts: u32,
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
            is_running_flag: Arc::new(AtomicBool::new(false)),
            restart_count: Arc::new(std::sync::Mutex::new(0)),
            max_restarts: 5,
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
        let pid = child.id();

        *process_guard = Some(child);
        self.is_running_flag.store(true, Ordering::Release);
        info!("Nebula process started (PID: {})", pid);

        // Spawn a task to monitor the process with automatic restart
        let process_clone = Arc::clone(&self.process);
        let is_running_clone = Arc::clone(&self.is_running_flag);
        let restart_count_clone = Arc::clone(&self.restart_count);
        let config_clone = self.config.clone();
        let max_restarts = self.max_restarts;

        tokio::spawn(async move {
            Self::monitor_process_with_restart(
                process_clone,
                is_running_clone,
                restart_count_clone,
                config_clone,
                max_restarts,
            )
            .await;
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut process_guard = self.process.write().await;

        if let Some(mut child) = process_guard.take() {
            info!("Stopping Nebula process");
            self.is_running_flag.store(false, Ordering::Release);

            // Try graceful shutdown first
            if let Err(e) = child.kill() {
                warn!("Failed to kill Nebula process gracefully: {}", e);
            }

            // Wait for process to exit (with timeout)
            tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                tokio::task::spawn_blocking(move || child.wait()),
            )
            .await
            .context("Timeout waiting for Nebula to stop")?
            .context("Failed to wait for Nebula process")??;

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
        self.is_running_flag.load(Ordering::Acquire)
    }

    async fn monitor_process_with_restart(
        process: Arc<RwLock<Option<Child>>>,
        is_running_flag: Arc<AtomicBool>,
        restart_count: Arc<std::sync::Mutex<u32>>,
        config: NebulaConfig,
        max_restarts: u32,
    ) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let mut process_guard = process.write().await;
            if let Some(ref mut child) = *process_guard {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        error!("Nebula process exited with status: {:?}", status);
                        *process_guard = None;
                        is_running_flag.store(false, Ordering::Release);

                        // Check restart count
                        let should_restart = {
                            let mut count = restart_count.lock().unwrap();
                            if *count < max_restarts {
                                *count += 1;
                                let current_count = *count;
                                warn!(
                                    "Restarting Nebula (attempt {}/{})",
                                    current_count, max_restarts
                                );
                                drop(count); // Drop guard before await
                                drop(process_guard);

                                // Exponential backoff
                                let delay = std::cmp::min(2u64.pow(current_count as u32), 60);
                                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                                true
                            } else {
                                false
                            }
                        };

                        if should_restart {
                            // Attempt restart
                            if let Some(ref config_path) = config.config_path {
                                match Self::attempt_restart(config_path).await {
                                    Ok(new_child) => {
                                        let mut guard = process.write().await;
                                        *guard = Some(new_child);
                                        is_running_flag.store(true, Ordering::Release);
                                        {
                                            let mut count = restart_count.lock().unwrap();
                                        *count = 0; // Reset on successful restart
                                        }
                                        info!("Nebula process restarted successfully");
                                    }
                                    Err(e) => {
                                        error!("Failed to restart Nebula: {}", e);
                                    }
                                }
                            }
                        } else {
                            error!("Nebula process failed {} times, giving up", max_restarts);
                            break;
                        }
                    }
                    Ok(None) => {
                        // Process still running
                        continue;
                    }
                    Err(e) => {
                        error!("Error checking Nebula process status: {}", e);
                        is_running_flag.store(false, Ordering::Release);
                        break;
                    }
                }
            } else {
                // Process not started or stopped
                is_running_flag.store(false, Ordering::Release);
                break;
            }
        }
    }

    async fn attempt_restart(config_path: &str) -> Result<Child> {
        let mut cmd = Command::new("nebula");
        cmd.arg("-config")
            .arg(config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd.spawn().context("Failed to restart Nebula process")
    }
}

impl Drop for NebulaManager {
    fn drop(&mut self) {
        // Try to stop the process synchronously if possible
        // This is best-effort cleanup
        if self.is_running_flag.load(Ordering::Acquire) {
            warn!("NebulaManager being dropped while process is running");
            // Attempt to kill the process using a blocking runtime handle
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                if let Ok(guard) = handle.block_on(async { self.process.write().await }) {
                    if let Some(mut child) = guard.as_ref() {
                        let _ = std::process::Command::new("kill")
                            .arg("-TERM")
                            .arg(child.id().to_string())
                            .output();
                    }
                }
            }
        }
    }
}
