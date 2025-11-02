// Integration test: DNS zone management flow

#[cfg(test)]
mod tests {
    use nnoe_agent::config::{DnsServiceConfig, EtcdConfig};
    use nnoe_agent::services::knot::KnotService;
    use nnoe_agent::etcd::EtcdClient;
    use std::fs;
    use std::path::Path;
    use testcontainers::{clients, Container, images};
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;
    use serde_json::json;

    fn setup_test_environment() {
        // Clean up test directories
        let _ = fs::remove_dir_all("/tmp/nnoe-test-cache");
        let _ = fs::remove_dir_all("/tmp/test-zones");
        fs::create_dir_all("/tmp/test-zones").unwrap();
    }

    async fn create_etcd_client(etcd_url: &str) -> EtcdClient {
        let etcd_config = EtcdConfig {
            endpoints: vec![etcd_url.to_string()],
            prefix: "/nnoe".to_string(),
            timeout_secs: 5,
            tls: None,
        };
        EtcdClient::new(&etcd_config).await.unwrap()
    }

    #[tokio::test]
    async fn test_zone_creation_flow() {
        setup_test_environment();
        
        // Start etcd container
        let docker = clients::Cli::default();
        let etcd_image = GenericImage::new("quay.io/coreos/etcd", "v3.5.9")
            .with_env_var("ETCD_ENABLE_V2", "true")
            .with_env_var("ETCD_LISTEN_CLIENT_URLS", "http://0.0.0.0:2379")
            .with_env_var("ETCD_ADVERTISE_CLIENT_URLS", "http://127.0.0.1:2379")
            .with_exposed_port(2379)
            .with_wait_for(WaitFor::message_on_stdout("ready to serve client requests"));
        
        let etcd_container = docker.run(etcd_image);
        let etcd_port = etcd_container.get_host_port_ipv4(2379);
        let etcd_url = format!("http://127.0.0.1:{}", etcd_port);
        
        // Create etcd client
        let etcd_client = create_etcd_client(&etcd_url).await;
        
        // Create zone data
        let zone_data = json!({
            "domain": "example.com",
            "ttl": 3600,
            "records": [
                {
                    "name": "@",
                    "type": "A",
                    "value": "192.0.2.1",
                    "ttl": 3600
                },
                {
                    "name": "www",
                    "type": "A",
                    "value": "192.0.2.1",
                    "ttl": 3600
                }
            ]
        });
        
        // Write zone to etcd
        let zone_key = "/nnoe/dns/zones/example.com";
        let zone_json = serde_json::to_vec(&zone_data).unwrap();
        etcd_client.put(zone_key, &zone_json).await.unwrap();
        
        // Create Knot service
        let dns_config = DnsServiceConfig {
            enabled: true,
            engine: "knot".to_string(),
            config_path: "/tmp/test-knot.conf".to_string(),
            zone_dir: "/tmp/test-zones".to_string(),
            listen_address: "127.0.0.1".to_string(),
            listen_port: 5353,
        };
        
        let mut knot_service = KnotService::new(dns_config.clone());
        knot_service.init(&[]).await.unwrap();
        
        // Simulate config change notification
        knot_service.on_config_change(zone_key, &zone_json).await.unwrap();
        
        // Verify zone file was created
        let zone_file = Path::new("/tmp/test-zones/example.com.zone");
        assert!(zone_file.exists(), "Zone file should exist");
        
        // Verify zone file content
        let zone_content = fs::read_to_string(zone_file).unwrap();
        assert!(zone_content.contains("example.com"), "Zone file should contain domain");
        assert!(zone_content.contains("192.0.2.1"), "Zone file should contain A record");
        assert!(zone_content.contains("SOA"), "Zone file should contain SOA record");
        
        // Verify Knot config was updated
        let knot_config_content = fs::read_to_string("/tmp/test-knot.conf").unwrap();
        assert!(knot_config_content.contains("example.com"), "Knot config should contain zone");
    }

    #[tokio::test]
    async fn test_knot_config_generation() {
        setup_test_environment();
        
        let dns_config = DnsServiceConfig {
            enabled: true,
            engine: "knot".to_string(),
            config_path: "/tmp/test-knot.conf".to_string(),
            zone_dir: "/tmp/test-zones".to_string(),
            listen_address: "127.0.0.1".to_string(),
            listen_port: 5353,
        };
        
        let mut knot_service = KnotService::new(dns_config);
        knot_service.init(&[]).await.unwrap();
        
        // Verify config file was created
        assert!(Path::new("/tmp/test-knot.conf").exists());
        
        // Verify config contains server section
        let config_content = fs::read_to_string("/tmp/test-knot.conf").unwrap();
        assert!(config_content.contains("server"), "Config should contain server section");
        assert!(config_content.contains("zone"), "Config should contain zone section");
    }

    #[tokio::test]
    async fn test_zone_update_flow() {
        setup_test_environment();
        
        // Start etcd container
        let docker = clients::Cli::default();
        let etcd_image = GenericImage::new("quay.io/coreos/etcd", "v3.5.9")
            .with_env_var("ETCD_ENABLE_V2", "true")
            .with_env_var("ETCD_LISTEN_CLIENT_URLS", "http://0.0.0.0:2379")
            .with_env_var("ETCD_ADVERTISE_CLIENT_URLS", "http://127.0.0.1:2379")
            .with_exposed_port(2379);
        
        let etcd_container = docker.run(etcd_image);
        let etcd_port = etcd_container.get_host_port_ipv4(2379);
        let etcd_url = format!("http://127.0.0.1:{}", etcd_port);
        
        let etcd_client = create_etcd_client(&etcd_url).await;
        
        let dns_config = DnsServiceConfig {
            enabled: true,
            engine: "knot".to_string(),
            config_path: "/tmp/test-knot.conf".to_string(),
            zone_dir: "/tmp/test-zones".to_string(),
            listen_address: "127.0.0.1".to_string(),
            listen_port: 5353,
        };
        
        let mut knot_service = KnotService::new(dns_config);
        knot_service.init(&[]).await.unwrap();
        
        // Create initial zone
        let zone_data = json!({
            "domain": "test.com",
            "ttl": 3600,
            "records": [{"name": "@", "type": "A", "value": "192.0.2.1", "ttl": 3600}]
        });
        let zone_key = "/nnoe/dns/zones/test.com";
        let zone_json = serde_json::to_vec(&zone_data).unwrap();
        knot_service.on_config_change(zone_key, &zone_json).await.unwrap();
        
        // Update zone with new record
        let updated_zone_data = json!({
            "domain": "test.com",
            "ttl": 3600,
            "records": [
                {"name": "@", "type": "A", "value": "192.0.2.1", "ttl": 3600},
                {"name": "mail", "type": "A", "value": "192.0.2.2", "ttl": 3600}
            ]
        });
        let updated_zone_json = serde_json::to_vec(&updated_zone_data).unwrap();
        knot_service.on_config_change(zone_key, &updated_zone_json).await.unwrap();
        
        // Verify zone file was updated
        let zone_file = Path::new("/tmp/test-zones/test.com.zone");
        let zone_content = fs::read_to_string(zone_file).unwrap();
        assert!(zone_content.contains("mail"), "Zone file should contain updated record");
        assert!(zone_content.contains("192.0.2.2"), "Zone file should contain new IP");
    }
}

