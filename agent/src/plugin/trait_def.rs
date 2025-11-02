use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ServicePlugin: Send + Sync {
    /// Plugin name for identification
    fn name(&self) -> &str;

    /// Initialize the plugin with configuration
    async fn init(&mut self, config: &[u8]) -> Result<()>;

    /// Handle configuration change notification
    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()>;

    /// Reload/restart the service managed by this plugin
    async fn reload(&mut self) -> Result<()>;

    /// Shutdown the plugin and clean up resources
    async fn shutdown(&mut self) -> Result<()>;

    /// Health check for the service
    async fn health_check(&self) -> Result<bool>;
}
