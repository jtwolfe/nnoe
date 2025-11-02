use crate::config::CerbosServiceConfig;
use crate::plugin::ServicePlugin;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::{info, warn};

mod cerbos {
    tonic::include_proto!("cerbos.svc.v1");
}

use cerbos::{
    cerbos_service_client::CerbosServiceClient, CheckResourcesRequest, Effect, Principal, Resource,
};

/// Cerbos Policy Decision Point (PDP) service integration
pub struct CerbosService {
    config: CerbosServiceConfig,
    client: Arc<RwLock<Option<CerbosServiceClient<Channel>>>>,
    policy_cache: Arc<RwLock<std::collections::HashMap<String, Vec<u8>>>>,
}

impl CerbosService {
    pub fn new(config: CerbosServiceConfig) -> Self {
        Self {
            config,
            client: Arc::new(RwLock::new(None)),
            policy_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn check_policy(
        &self,
        resource_kind: &str,
        resource_id: &str,
        action: &str,
        principal_id: &str,
        principal_roles: Vec<String>,
    ) -> Result<bool> {
        let mut client_guard = self.client.write().await;
        let client = client_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Cerbos client not initialized"))?;

        let request = tonic::Request::new(CheckResourcesRequest {
            request_id: format!("nnoe-{}", uuid::Uuid::new_v4()),
            principal: Some(Principal {
                id: principal_id.to_string(),
                roles: principal_roles,
                attr: std::collections::HashMap::new(),
            }),
            resources: vec![Resource {
                kind: resource_kind.to_string(),
                policy_version: "default".to_string(),
                id: resource_id.to_string(),
                attr: std::collections::HashMap::new(),
            }],
            aux_data: None,
        });

        let response = client
            .check_resources(request)
            .await
            .context("Failed to call Cerbos CheckResources")?
            .into_inner();

        // Check if any action result is ALLOW
        for result in response.results {
            for action_effect in result.actions {
                if action_effect.action == action {
                    return Ok(action_effect.effect() == Effect::Allow);
                }
            }
        }

        // Default deny if no explicit allow
        Ok(false)
    }

    async fn ensure_client_connected(&self) -> Result<()> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            info!("Connecting to Cerbos at {}", self.config.endpoint);
            let channel = Channel::from_shared(self.config.endpoint.clone())
                .context("Invalid Cerbos endpoint URL")?
                .timeout(Duration::from_secs(self.config.timeout_secs))
                .connect()
                .await
                .context("Failed to connect to Cerbos")?;

            *client_guard = Some(CerbosServiceClient::new(channel));
            info!("Connected to Cerbos successfully");
        }
        Ok(())
    }

    async fn load_policy_from_cache(&self, policy_id: &str) -> Option<Vec<u8>> {
        let cache = self.policy_cache.read().await;
        cache.get(policy_id).cloned()
    }

    async fn update_policy_cache(&self, policy_id: &str, policy: Vec<u8>) {
        let mut cache = self.policy_cache.write().await;
        cache.insert(policy_id.to_string(), policy);
        info!("Updated policy cache for: {}", policy_id);
    }
}

#[async_trait]
impl ServicePlugin for CerbosService {
    fn name(&self) -> &str {
        "cerbos"
    }

    async fn init(&mut self, _config: &[u8]) -> Result<()> {
        info!("Initializing Cerbos service at {}", self.config.endpoint);

        // Initialize gRPC client connection
        self.ensure_client_connected().await?;

        info!("Cerbos service initialized successfully");

        Ok(())
    }

    async fn on_config_change(&mut self, key: &str, value: &[u8]) -> Result<()> {
        if key.contains("/policies/") {
            // Extract policy ID from key
            if let Some(policy_id) = key.split('/').last() {
                self.update_policy_cache(policy_id, value.to_vec()).await;
                info!("Policy updated: {}", policy_id);
            }
        }
        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        info!("Reloading Cerbos service");
        // Clear policy cache to force reload
        let mut cache = self.policy_cache.write().await;
        cache.clear();
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Cerbos service");
        let mut cache = self.policy_cache.write().await;
        cache.clear();
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        // Try to ensure client is connected
        if let Err(e) = self.ensure_client_connected().await {
            warn!("Cerbos health check failed: {}", e);
            return Ok(false);
        }

        // Perform a simple policy check to verify service is responding
        // Using a default resource/action that should exist
        match self
            .check_policy(
                "dns_query",
                "health",
                "allow",
                "health-check",
                vec!["admin".to_string()],
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Cerbos health check policy test failed: {}", e);
                Ok(false)
            }
        }
    }
}
