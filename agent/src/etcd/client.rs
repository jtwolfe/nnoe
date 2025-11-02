use crate::config::EtcdConfig;
use anyhow::{Context, Result};
use etcd_client::{Client, ConnectOptions, WatchOptions};
use std::fs;
use std::sync::Arc;
use tokio_stream::StreamExt;
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
            let mut ca_cert_reader = ca_cert_data.as_bytes();
            let ca_certs: Result<Vec<_>, _> = rustls_pemfile::certs(&mut ca_cert_reader).collect();
            let ca_certs = ca_certs.context("Failed to parse CA certificate")?;

            // Load client certificate
            let client_cert_data = fs::read_to_string(&tls_config.cert).with_context(|| {
                format!("Failed to read client certificate from {}", tls_config.cert)
            })?;
            let mut client_cert_reader = client_cert_data.as_bytes();
            let client_certs: Result<Vec<_>, _> = rustls_pemfile::certs(&mut client_cert_reader).collect();
            let client_certs = client_certs.context("Failed to parse client certificate")?;

            // Load client private key
            let client_key_data = fs::read_to_string(&tls_config.key)
                .with_context(|| format!("Failed to read client key from {}", tls_config.key))?;
            let mut key_reader = client_key_data.as_bytes();
            let client_key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
                .next()
                .transpose()
                .context("Failed to parse client private key")?
                .ok_or_else(|| anyhow::anyhow!("No private key found in key file"))?;

            // Build TLS configuration
            let mut root_cert_store = rustls::RootCertStore::empty();
            for cert in ca_certs {
                root_cert_store
                    .add(cert)
                    .context("Failed to add CA certificate to trust store")?;
            }

            let client_cert_chain: Vec<rustls::Certificate> =
                client_certs.into_iter().map(rustls::Certificate).collect();

            let client_key = rustls::PrivateKey(client_key);

            let tls_config = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_cert_store)
                .with_client_auth_cert(client_cert_chain, client_key)
                .map_err(|e| anyhow::anyhow!("Failed to build TLS config: {}", e))?;

            // etcd-client accepts rustls ClientConfig through ConnectOptions
            // The exact method may vary by etcd-client version
            // Try with_tls_config which returns a new ConnectOptions
            connect_options = connect_options.with_tls_config(tls_config);
            info!("TLS configuration applied successfully");
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
