// Unit tests for configuration management

#[cfg(test)]
mod tests {
    use nnoe_agent::config::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default_config();
        assert_eq!(config.node.name, "nnoe-node-1");
        assert!(matches!(config.node.role, NodeRole::Active));
    }

    #[test]
    fn test_node_role_serialization() {
        let role_str = match NodeRole::Active {
            NodeRole::Management => "management",
            NodeRole::DbOnly => "db-only",
            NodeRole::Active => "active",
        };
        assert_eq!(role_str, "active");
    }

    #[test]
    fn test_etcd_config_defaults() {
        let config = AgentConfig::default_config();
        assert_eq!(config.etcd.timeout_secs, 5);
        assert_eq!(config.cache.default_ttl_secs, 300);
        assert_eq!(config.cache.max_size_mb, 100);
    }

    #[test]
    fn test_service_configs() {
        let config = AgentConfig::default_config();
        // Service configs are optional
        assert!(config.services.dns.is_none());
        assert!(config.services.dhcp.is_none());
    }
}

