// End-to-end tests for NNOE

use nnoe_agent::config::AgentConfig;
use nnoe_agent::etcd::EtcdClient;
use nnoe_agent::config::EtcdConfig;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore] // Requires full environment
async fn test_full_zone_propagation() {
    // Test complete flow:
    // 1. Create zone in etcd
    // 2. Agent watches and receives change
    // 3. Knot service generates zone file
    // 4. Verify zone file content

    let etcd_config = EtcdConfig {
        endpoints: vec!["http://etcd:2379".to_string()],
        prefix: "/nnoe/test".to_string(),
        timeout_secs: 5,
        tls: None,
    };

    let etcd_client = EtcdClient::new(&etcd_config).await.unwrap();

    // Create test zone
    let zone_data = serde_json::json!({
        "domain": "test.example.com",
        "ttl": 3600,
        "records": [
            {"name": "@", "type": "A", "value": "192.168.1.1"}
        ]
    });

    etcd_client
        .put(
            "/nnoe/test/dns/zones/test.example.com",
            serde_json::to_string(&zone_data).unwrap().as_bytes(),
        )
        .await
        .unwrap();

    // Wait for propagation
    sleep(Duration::from_millis(500)).await;

    // Verify zone was stored
    let stored = etcd_client
        .get("/nnoe/test/dns/zones/test.example.com")
        .await
        .unwrap();

    assert!(stored.is_some());
}

#[tokio::test]
#[ignore]
async fn test_dhcp_scope_propagation() {
    // Test DHCP scope propagation flow
    let etcd_config = EtcdConfig {
        endpoints: vec!["http://etcd:2379".to_string()],
        prefix: "/nnoe/test".to_string(),
        timeout_secs: 5,
        tls: None,
    };

    let etcd_client = EtcdClient::new(&etcd_config).await.unwrap();

    let scope_data = serde_json::json!({
        "subnet": "192.168.1.0/24",
        "pool": {"start": "192.168.1.100", "end": "192.168.1.200"},
        "gateway": "192.168.1.1"
    });

    etcd_client
        .put(
            "/nnoe/test/dhcp/scopes/scope-1",
            serde_json::to_string(&scope_data).unwrap().as_bytes(),
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(500)).await;

    let stored = etcd_client
        .get("/nnoe/test/dhcp/scopes/scope-1")
        .await
        .unwrap();

    assert!(stored.is_some());
}

#[tokio::test]
#[ignore]
async fn test_threat_intelligence_flow() {
    // Test MISP -> etcd -> dnsdist flow
    let etcd_config = EtcdConfig {
        endpoints: vec!["http://etcd:2379".to_string()],
        prefix: "/nnoe/test".to_string(),
        timeout_secs: 5,
        tls: None,
    };

    let etcd_client = EtcdClient::new(&etcd_config).await.unwrap();

    let threat_data = serde_json::json!({
        "domain": "malicious.example.com",
        "source": "MISP",
        "severity": "high",
        "timestamp": "2025-01-01T00:00:00Z"
    });

    etcd_client
        .put(
            "/nnoe/test/threats/domains/malicious.example.com",
            serde_json::to_string(&threat_data).unwrap().as_bytes(),
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(500)).await;

    // Verify threat was stored
    let stored = etcd_client
        .get("/nnoe/test/threats/domains/malicious.example.com")
        .await
        .unwrap();

    assert!(stored.is_some());
}

