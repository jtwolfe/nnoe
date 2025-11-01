# Plugin Development Guide

Guide for developing custom service plugins for NNOE.

## Plugin System Overview

NNOE uses a plugin architecture that allows extending functionality without modifying core code. Plugins implement the `ServicePlugin` trait and are managed by the `PluginRegistry`.

## Creating a Plugin

### 1. Implement ServicePlugin Trait

```rust
use async_trait::async_trait;
use crate::plugin::ServicePlugin;
use anyhow::Result;

pub struct MyService {
    // Plugin state
}

#[async_trait]
impl ServicePlugin for MyService {
    fn name(&self) -> &str {
        "my-service"
    }

    async fn init(&mut self, config: &[u8]) -> Result<()> {
        // Initialize plugin
        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
        // Handle configuration changes
        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        // Reload service
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Cleanup resources
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check service health
        Ok(true)
    }
}
```

### 2. Register Plugin

In `agent/src/core/orchestrator.rs`:

```rust
use crate::services::MyService;

// In register_services method
if my_service_config.enabled {
    let service = MyService::new(config);
    let plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>> = 
        Arc::new(RwLock::new(Box::new(service)));
    
    {
        let mut service = plugin.write().await;
        service.init(&[]).await?;
    }
    
    self.plugin_registry.register(plugin).await?;
}
```

## Plugin Lifecycle

1. **Initialization**: `init()` called during agent startup
2. **Configuration Changes**: `on_config_change()` called when etcd keys change
3. **Reload**: `reload()` called for manual reloads
4. **Health Checks**: `health_check()` called periodically
5. **Shutdown**: `shutdown()` called during agent shutdown

## Configuration

Plugins receive configuration via:

1. **Agent Config**: Add to `AgentConfig` in `agent/src/config.rs`
2. **etcd Keys**: Watch for changes in etcd
3. **Init Config**: Pass initial config bytes in `init()`

### Example Config

```toml
[services.my_service]
enabled = true
endpoint = "http://localhost:8080"
timeout_secs = 5
```

## etcd Integration

Watch for configuration changes:

```rust
async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
    if key.contains("/my-service/") {
        let config: MyConfig = serde_json::from_slice(value)?;
        self.update_config(config).await?;
        self.reload().await?;
    }
    Ok(())
}
```

## Example Plugin: Custom DNS Service

```rust
pub struct CustomDnsService {
    config: CustomDnsConfig,
    zones: Arc<RwLock<HashMap<String, ZoneData>>>,
}

#[async_trait]
impl ServicePlugin for CustomDnsService {
    fn name(&self) -> &str {
        "custom-dns"
    }

    async fn init(&mut self, _config: &[u8]) -> Result<()> {
        // Start DNS server
        info!("Initializing custom DNS service");
        self.start_dns_server().await?;
        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
        if key.contains("/dns/zones/") {
            let zone: ZoneData = serde_json::from_slice(value)?;
            let mut zones = self.zones.write().await;
            zones.insert(zone.name.clone(), zone);
            self.reload_zones().await?;
        }
        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        self.reload_zones().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.stop_dns_server().await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if DNS server is responding
        Ok(self.is_dns_server_healthy().await)
    }
}
```

## Testing Plugins

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_init() {
        let mut plugin = MyService::new(config);
        assert!(plugin.init(&[]).await.is_ok());
    }
}
```

### Integration Tests

Use mock etcd for testing:

```rust
#[tokio::test]
async fn test_config_change() {
    let mock_etcd = MockEtcdServer::new();
    let mut plugin = MyService::new(config);
    
    // Simulate etcd change
    mock_etcd.put("/nnoe/my-service/config".to_string(), config_bytes);
    
    // Verify plugin received change
    assert!(plugin.on_config_change("/nnoe/my-service/config", config_bytes).await.is_ok());
}
```

## Best Practices

1. **Error Handling**: Return clear errors, log appropriately
2. **Async Operations**: Use async/await for I/O operations
3. **Resource Management**: Clean up resources in shutdown()
4. **Configuration Validation**: Validate config in init()
5. **Health Checks**: Implement meaningful health checks
6. **Logging**: Use tracing for structured logging
7. **Documentation**: Document plugin behavior and configuration

## Plugin Registry

Plugins are registered in the `PluginRegistry` which:
- Manages plugin lifecycle
- Routes config changes to plugins
- Provides health check aggregation
- Handles plugin reloads

## Advanced Patterns

### Plugin Dependencies

If plugins depend on each other:

```rust
pub struct DependentPlugin {
    dependency: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>>,
}
```

### Shared State

Use Arc<RwLock<>> for shared state:

```rust
pub struct SharedStatePlugin {
    shared_state: Arc<RwLock<SharedData>>,
}
```

### Event Emitters

Emit events for other plugins:

```rust
pub struct EventEmittingPlugin {
    event_tx: broadcast::Sender<Event>,
}
```

## Publishing Plugins

1. Create separate crate or module
2. Export ServicePlugin implementation
3. Document configuration
4. Provide examples
5. Add to NNOE documentation

## Resources

- [ServicePlugin Trait](agent/src/plugin/trait_def.rs)
- [Plugin Registry](agent/src/plugin/registry.rs)
- [Example Plugins](agent/src/services/)

