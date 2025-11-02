use crate::plugin::ServicePlugin;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(
        &self,
        plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>>,
    ) -> Result<()> {
        let name = {
            let guard = plugin.read().await;
            guard.name().to_string()
        };
        info!("Registering plugin: {}", name);

        let mut plugins = self.plugins.write().await;
        plugins.insert(name, plugin);
        Ok(())
    }

    pub async fn get(
        &self,
        name: &str,
    ) -> Option<Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>>> {
        let plugins = self.plugins.read().await;
        plugins.get(name).map(Arc::clone)
    }

    pub async fn notify_config_change(&self, key: &str, value: &[u8]) -> Result<()> {
        debug!("Notifying plugins of config change: {}", key);

        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            if let Err(e) = {
                let mut guard = plugin.write().await;
                guard.on_config_change(key, value).await
            } {
                tracing::error!("Plugin {} failed to handle config change: {}", name, e);
            }
        }
        Ok(())
    }

    pub async fn reload_all(&self) -> Result<()> {
        info!("Reloading all plugins");

        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            if let Err(e) = {
                let mut guard = plugin.write().await;
                guard.reload().await
            } {
                tracing::error!("Plugin {} failed to reload: {}", name, e);
            }
        }
        Ok(())
    }

    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            let health = {
                let guard = plugin.read().await;
                guard.health_check().await.unwrap_or(false)
            };
            results.insert(name.clone(), health);
        }

        results
    }
}
