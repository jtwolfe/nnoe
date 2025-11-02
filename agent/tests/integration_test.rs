// Integration tests for NNOE agent
// More comprehensive tests will be added in Phase 4

#[cfg(test)]
mod tests {
    use nnoe_agent::config::{AgentConfig, NodeRole};

    #[test]
    fn test_config_loading() {
        // Test configuration loading will be implemented with actual config file
        let config = AgentConfig::default_config();
        assert_eq!(config.node.name, "nnoe-node-1");
    }

    #[test]
    fn test_node_role_parsing() {
        let config = AgentConfig::default_config();
        match config.node.role {
            NodeRole::Active => {
                // Expected for default config
            }
            _ => panic!("Unexpected role"),
        }
    }
}
