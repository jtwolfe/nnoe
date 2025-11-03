use crate::config::EtcdConfig;
use anyhow::{Context, Result};
use etcd_client::{Client, ConnectOptions, WatchOptions, Certificate, Identity, TlsOptions};
use std::fs;
use tracing::{debug, info};

pub struct EtcdClient {
    client: Client,
    prefix: String,
}

// Type alias for watch stream
pub type WatchStream = etcd_client::WatchStream;

impl EtcdClient {
    pub async fn new(config: &EtcdConfig) -> Result<Self> {
        info!("Connecting to etcd at endpoints: {:?}", config.endpoints);

        let mut connect_options = ConnectOptions::new();

        // Configure TLS if provided
        if let Some(ref tls_config) = config.tls {
            info!("Configuring TLS for etcd connection");

            // Load CA certificate
            let ca_cert_data = fs::read_to_string(&tls_config.ca_cert).with_context(|| {
                format!("Failed to read CA certificate from {}", tls_config.ca_cert)
            })?;

            // Load client certificate
            let client_cert_data = fs::read_to_string(&tls_config.cert).with_context(|| {
                format!("Failed to read client certificate from {}", tls_config.cert)
            })?;

            // Load client private key
            let client_key_data = fs::read_to_string(&tls_config.key)
                .with_context(|| format!("Failed to read client key from {}", tls_config.key))?;

            // Build TlsOptions (which is tonic::transport::ClientTlsConfig)
            // Convert PEM certificates to tonic types
            let ca_cert = Certificate::from_pem(ca_cert_data.as_bytes())
                .context("Failed to convert CA certificate to tonic Certificate")?;
            
            // Combine client cert and key into PEM format for Identity
            let identity_pem = format!("{}\n{}", client_cert_data, client_key_data);
            let identity = Identity::from_pem(identity_pem.as_bytes())
                .context("Failed to create Identity from client certificate and key")?;
            
            let tls_options = TlsOptions::new()
                .ca_certificate(ca_cert)
                .identity(identity);
            
            // Apply TLS configuration to ConnectOptions
            // etcd-client 0.11 uses tonic for gRPC, which uses ClientTlsConfig
            connect_options = connect_options.with_tls(tls_options);
            info!("TLS configuration applied to etcd client");
        }

        let client = Client::connect(&config.endpoints, Some(connect_options))
            .await
            .context("Failed to connect to etcd")?;

        Ok(Self {
            client,
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
        let resp = client
            .get(
                full_prefix,
                Some(etcd_client::GetOptions::new().with_prefix()),
            )
            .await?;

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
            .watch(full_prefix, Some(WatchOptions::new().with_prefix()))
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
