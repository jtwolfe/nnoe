use crate::config::AgentConfig;
use crate::etcd::EtcdClient;
use crate::sled_cache::CacheManager;
use crate::nebula::NebulaManager;
use crate::plugin::{PluginRegistry, ServicePlugin};
use crate::services::{CerbosService, DnsdistService, KeaService, KnotService, LynisService};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct Orchestrator {
    config: AgentConfig,
    etcd_client: Arc<EtcdClient>,
    cache_manager: Arc<CacheManager>,
    nebula_manager: Option<Arc<NebulaManager>>,
    plugin_registry: Arc<PluginRegistry>,
}

impl Orchestrator {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        info!("Initializing orchestrator for node: {}", config.node.name);

        // Initialize etcd client
        let etcd_client = Arc::new(
            EtcdClient::new(&config.etcd)
                .await
                .context("Failed to create etcd client")?,
        );
        info!("etcd client initialized");

        // Initialize cache manager (will start background sweep task)
        let cache_manager = Arc::new(
            CacheManager::new(&config.cache)
                .context("Failed to create cache manager")?,
        );
        info!("Cache manager initialized at {}", config.cache.path);

        // Initialize Nebula manager if enabled
        let nebula_manager = if config.nebula.enabled {
            info!("Initializing Nebula overlay network");
            Some(Arc::new(
                NebulaManager::new(&config.nebula)
                    .await
                    .context("Failed to create Nebula manager")?,
            ))
        } else {
            warn!("Nebula overlay network disabled");
            None
        };

        // Initialize plugin registry
        let plugin_registry = Arc::new(PluginRegistry::new());

        Ok(Self {
            config,
            etcd_client,
            cache_manager,
            nebula_manager,
            plugin_registry,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting orchestrator main loop");

        // Start Nebula if enabled
        if let Some(ref nebula) = self.nebula_manager {
            nebula.start().await?;
            info!("Nebula overlay network started");
        }

        // Register service plugins based on configuration
        self.register_services().await?;

        // Start watching etcd for configuration changes
        self.watch_config_changes().await?;

        // Main event loop (this should never return in normal operation)
        info!("Orchestrator running");
        tokio::signal::ctrl_c().await?;
        info!("Received shutdown signal");

        Ok(())
    }

    async fn register_services(&mut self) -> Result<()> {
        info!("Registering service plugins");

        // Skip service registration for DB-only nodes
        if matches!(self.config.node.role, crate::config::NodeRole::DbOnly) {
            info!("DB-only node: Skipping service registration (DNS/DHCP/dnsdist)");
            info!("This node will only maintain etcd replication and cache");
            return Ok(());
        }

        // Register DNS service (Knot) if enabled
        if let Some(ref dns_config) = self.config.services.dns {
            if dns_config.enabled {
                let knot_service = KnotService::new(dns_config.clone());
                let plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>> = 
                    Arc::new(RwLock::new(Box::new(knot_service)));
                
                // Initialize the plugin
                {
                    let mut service = plugin.write().await;
                    service.init(&[]).await?;
                }
                
                self.plugin_registry.register(plugin).await?;
                info!("Knot DNS service registered and initialized");
            }
        }

        // Register DHCP service (Kea) if enabled
        if let Some(ref dhcp_config) = self.config.services.dhcp {
            if dhcp_config.enabled {
                let kea_service = KeaService::new(dhcp_config.clone());
                let plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>> = 
                    Arc::new(RwLock::new(Box::new(kea_service)));
                
                {
                    let mut service = plugin.write().await;
                    service.init(&[]).await?;
                }
                
                self.plugin_registry.register(plugin).await?;
                info!("Kea DHCP service registered and initialized");
            }
        }

        // Register dnsdist service if enabled
        if let Some(ref dnsdist_config) = self.config.services.dnsdist {
            if dnsdist_config.enabled {
                let dnsdist_service = DnsdistService::new(dnsdist_config.clone());
                let plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>> = 
                    Arc::new(RwLock::new(Box::new(dnsdist_service)));
                
                {
                    let mut service = plugin.write().await;
                    service.init(&[]).await?;
                }
                
                self.plugin_registry.register(plugin).await?;
                info!("dnsdist service registered and initialized");
            }
        }

        // Register Cerbos service if enabled
        if let Some(ref cerbos_config) = self.config.services.cerbos {
            if cerbos_config.enabled {
                let cerbos_service = CerbosService::new(cerbos_config.clone());
                let plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>> = 
                    Arc::new(RwLock::new(Box::new(cerbos_service)));
                
                {
                    let mut service = plugin.write().await;
                    service.init(&[]).await?;
                }
                
                self.plugin_registry.register(plugin).await?;
                info!("Cerbos service registered and initialized");
            }
        }

        // Register Lynis service if enabled
        if let Some(ref lynis_config) = self.config.services.lynis {
            if lynis_config.enabled {
                let node_id = self.config.node.node_id.clone();
                let lynis_service = LynisService::new(lynis_config.clone(), node_id);
                
                // Set etcd client for Lynis service before initialization
                lynis_service.set_etcd_client(Arc::clone(&self.etcd_client)).await;
                
                let plugin: Arc<RwLock<Box<dyn ServicePlugin + Send + Sync>>> = 
                    Arc::new(RwLock::new(Box::new(lynis_service)));
                
                let init_config = serde_json::json!({
                    "node_id": self.config.node.name
                });
                let init_bytes = serde_json::to_vec(&init_config)?;
                
                {
                    let mut service = plugin.write().await;
                    service.init(&init_bytes).await?;
                }
                
                self.plugin_registry.register(plugin).await?;
                info!("Lynis service registered and initialized");
            }
        }

        Ok(())
    }

    async fn watch_config_changes(&self) -> Result<()> {
        info!("Starting etcd watch for configuration changes");

        let watch_prefixes = vec![
            format!("{}/dns/zones", self.config.etcd.prefix),
            format!("{}/dhcp/scopes", self.config.etcd.prefix),
            format!("{}/policies", self.config.etcd.prefix),
            format!("{}/threats", self.config.etcd.prefix),
        ];

        for prefix in watch_prefixes {
            let client = Arc::clone(&self.etcd_client);
            let cache = Arc::clone(&self.cache_manager);
            let registry = Arc::clone(&self.plugin_registry);

            tokio::spawn(async move {
                if let Err(e) = Self::watch_prefix(client, cache, registry, prefix).await {
                    error!("Watch error for prefix {}: {}", prefix, e);
                }
            });
        }

        Ok(())
    }

    async fn watch_prefix(
        client: Arc<EtcdClient>,
        cache: Arc<CacheManager>,
        registry: Arc<PluginRegistry>,
        prefix: String,
    ) -> Result<()> {
        let mut watcher = client.watch(&prefix).await?;

        info!("Watching etcd prefix: {}", prefix);

        while let Some(event_result) = watcher.next().await {
            match event_result {
                Ok(etcd_event) => {
                    // Update cache
                    for kv_event in etcd_event.events() {
                        match kv_event.event_type() {
                            etcd_client::EventType::Put => {
                                if let Some(kv) = kv_event.kv() {
                                    let key = String::from_utf8_lossy(kv.key());
                                    if let Some(value) = kv.value() {
                                        if let Err(e) = cache.put(key.as_ref(), value) {
                                            error!("Failed to update cache for {}: {}", key, e);
                                        } else {
                                            info!("Updated cache for key: {}", key);
                                        }

                                        // Notify plugins
                                        if let Err(e) = registry.notify_config_change(key.as_ref(), value).await {
                                            error!("Failed to notify plugins: {}", e);
                                        }
                                    }
                                }
                            }
                            etcd_client::EventType::Delete => {
                                if let Some(kv) = kv_event.kv() {
                                    let key = String::from_utf8_lossy(kv.key());
                                    if let Err(e) = cache.delete(key.as_ref()) {
                                        error!("Failed to delete from cache for {}: {}", key, e);
                                    } else {
                                        info!("Deleted from cache: {}", key);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Watch error for prefix {}: {}", prefix, e);
                    // Continue watching - etcd client should handle reconnection
                }
            }
        }

        warn!("Watch stream ended for prefix: {}", prefix);
        Ok(())
    }
}

