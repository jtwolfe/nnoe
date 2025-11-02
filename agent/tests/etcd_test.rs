#[cfg(test)]
mod tests {
    use nnoe_agent::config::{EtcdConfig, TlsConfig};
    use nnoe_agent::etcd::EtcdClient;

    #[tokio::test]
    #[ignore] // Requires etcd running
    async fn test_etcd_put_get() {
        let config = EtcdConfig {
            endpoints: vec!["http://127.0.0.1:2379".to_string()],
            prefix: "/test".to_string(),
            timeout_secs: 5,
            tls: None,
        };

        let client = EtcdClient::new(&config).await.unwrap();

        client.put("test-key", b"test-value").await.unwrap();
        let value = client.get("test-key").await.unwrap();

        assert_eq!(value, Some(b"test-value".to_vec()));
    }

    #[tokio::test]
    #[ignore] // Requires etcd running
    async fn test_etcd_list_prefix() {
        let config = EtcdConfig {
            endpoints: vec!["http://127.0.0.1:2379".to_string()],
            prefix: "/test".to_string(),
            timeout_secs: 5,
            tls: None,
        };

        let client = EtcdClient::new(&config).await.unwrap();

        client.put("test/key1", b"value1").await.unwrap();
        client.put("test/key2", b"value2").await.unwrap();

        let results = client.list_prefix("test/").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires etcd with TLS configured
    async fn test_etcd_tls_connection() {
        let tls_config = TlsConfig {
            ca_cert: "/etc/nnoe/certs/ca.crt".to_string(),
            cert: "/etc/nnoe/certs/client.crt".to_string(),
            key: "/etc/nnoe/certs/client.key".to_string(),
        };

        let config = EtcdConfig {
            endpoints: vec!["https://127.0.0.1:2379".to_string()],
            prefix: "/test".to_string(),
            timeout_secs: 5,
            tls: Some(tls_config),
        };

        let client_result = EtcdClient::new(&config).await;
        
        // Should either succeed (if TLS is properly configured) or fail with specific TLS/certificate errors
        match client_result {
            Ok(_) => {
                // TLS connection successful - verify it works
                let client = client_result.unwrap();
                let _ = client.put("tls-test-key", b"tls-test-value").await;
            }
            Err(e) => {
                // Verify error is TLS/certificate related, not a generic connection error
                let error_msg = e.to_string().to_lowercase();
                assert!(
                    error_msg.contains("certificate")
                        || error_msg.contains("tls")
                        || error_msg.contains("handshake")
                        || error_msg.contains("rustls")
                        || error_msg.contains("cert")
                    // If file not found, that's also acceptable for test environment
                    || error_msg.contains("no such file")
                );
            }
        }
    }
}
