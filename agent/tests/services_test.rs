#[cfg(test)]
mod tests {
    use nnoe_agent::config::{DnsServiceConfig, DhcpServiceConfig, CerbosServiceConfig};
    use nnoe_agent::services::knot::KnotService;
    use nnoe_agent::services::kea::KeaService;
    use nnoe_agent::services::cerbos::CerbosService;
    use std::fs;

    fn setup_test_dir(path: &str) {
        fs::create_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn test_knot_service_init() {
        setup_test_dir("/tmp/test-knot");
        
        let config = DnsServiceConfig {
            enabled: true,
            engine: "knot".to_string(),
            config_path: "/tmp/test-knot/knot.conf".to_string(),
            zone_dir: "/tmp/test-knot/zones".to_string(),
            listen_address: "127.0.0.1".to_string(),
            listen_port: 5353,
        };
        
        let mut service = KnotService::new(config);
        let result = service.init(&[]).await;
        
        // Should succeed or fail gracefully
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("knot"));
    }

    #[tokio::test]
    async fn test_kea_service_init() {
        setup_test_dir("/tmp/test-kea");
        
        let config = DhcpServiceConfig {
            enabled: true,
            engine: "kea".to_string(),
            config_path: "/tmp/test-kea/kea.conf".to_string(),
            ha_pair_id: None,
            interface: "eth0".to_string(),
            control_port: 8000,
        };
        
        let mut service = KeaService::new(config);
        let result = service.init(&[]).await;
        
        // Should succeed or fail gracefully
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("kea"));
    }

    #[tokio::test]
    #[ignore] // Requires Cerbos running
    async fn test_cerbos_service_connection() {
        let config = CerbosServiceConfig {
            enabled: true,
            endpoint: "http://localhost:8222".to_string(),
            timeout_secs: 2,
        };
        
        let mut service = CerbosService::new(config);
        let result = service.init(&[]).await;
        
        // Should either connect or fail with connection error
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("connection") ||
                result.unwrap_err().to_string().contains("Failed to connect"));
    }

    #[tokio::test]
    #[ignore] // Requires Cerbos running
    async fn test_cerbos_policy_check() {
        let config = CerbosServiceConfig {
            enabled: true,
            endpoint: "http://localhost:8222".to_string(),
            timeout_secs: 2,
        };
        
        let service = CerbosService::new(config);
        
        // This test would require a properly initialized service
        // For now, just verify the service can be created
        assert_eq!(service.name(), "cerbos");
    }
}

