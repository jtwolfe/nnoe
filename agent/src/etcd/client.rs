use crate::config::EtcdConfig;
use anyhow::{Context, Result};
use etcd_client::{Client, WatchOptions};
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};

pub struct EtcdClient {
    client: Client,
    prefix: String,
}

// Type alias for watch stream
pub type WatchStream = etcd_client::WatchStream;

impl EtcdClient {
    pub async fn new(config: &EtcdConfig) -> Result<Self> {
        info!("Connecting to etcd at endpoints: {:?}", config.endpoints);

        let mut client_builder = Client::connect(&config.endpoints, None)
            .await
            .context("Failed to connect to etcd")?;

        // Configure TLS if provided
        if let Some(ref tls_config) = config.tls {
            // TLS configuration will be implemented when needed
            warn!("TLS configuration specified but not yet implemented");
        }

        Ok(Self {
            client: client_builder,
            prefix: config.prefix.clone(),
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full_key = self.full_key(key);
        debug!("Getting key: {}", full_key);

        let mut client = self.client.kv_client();
        let resp = client.get(full_key, None).await?;

        if let Some(kv) = resp.kvs().first() {
            Ok(Some(kv.value().to_vec()))
        } else {
            Ok(None)
        }
    }

    pub async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let full_key = self.full_key(key);
        debug!("Putting key: {}", full_key);

        let mut client = self.client.kv_client();
        client.put(full_key, value, None).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key);
        debug!("Deleting key: {}", full_key);

        let mut client = self.client.kv_client();
        client.delete(full_key, None).await?;
        Ok(())
    }

    pub async fn list_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        let full_prefix = self.full_key(prefix);
        debug!("Listing prefix: {}", full_prefix);

        let mut client = self.client.kv_client();
        let resp = client.get(full_prefix, Some(etcd_client::GetOptions::new().with_prefix())).await?;

        let mut results = Vec::new();
        for kv in resp.kvs() {
            let key = String::from_utf8_lossy(kv.key());
            let value = kv.value().to_vec();
            results.push((key.to_string(), value));
        }

        Ok(results)
    }

    pub async fn watch(&self, prefix: &str) -> Result<WatchStream> {
        let full_prefix = self.full_key(prefix);
        info!("Starting watch on prefix: {}", full_prefix);

        let mut client = self.client.watch_client();
        let (_, stream) = client
            .watch(
                full_prefix,
                Some(WatchOptions::new().with_prefix()),
            )
            .await?;

        Ok(stream)
    }

    fn full_key(&self, key: &str) -> String {
        if key.starts_with(&self.prefix) {
            key.to_string()
        } else {
            format!("{}{}", self.prefix, key)
        }
    }
}

