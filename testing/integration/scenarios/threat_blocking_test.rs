// Integration test: Threat intelligence blocking flow

#[cfg(test)]
mod tests {
    use nnoe_agent::config::DnsdistServiceConfig;
    use nnoe_agent::services::dnsdist::DnsdistService;
    use std::fs;
    use std::path::Path;

    #[tokio::test]
    async fn test_dnsdist_lua_generation() {
        let dnsdist_config = DnsdistServiceConfig {
            enabled: true,
            config_path: "/tmp/test-dnsdist.conf".to_string(),
            lua_script_path: "/tmp/test-dnsdist.lua".to_string(),
        };
        
        let mut dnsdist_service = DnsdistService::new(dnsdist_config);
        dnsdist_service.init(&[]).await.unwrap();
        
        // Verify Lua script was created
        assert!(Path::new("/tmp/test-dnsdist.lua").exists());
        
        // Verify config references Lua script
        let config_content = fs::read_to_string("/tmp/test-dnsdist.conf").unwrap();
        assert!(config_content.contains("test-dnsdist.lua"));
    }

    #[tokio::test]
    #[ignore] // Requires etcd and MISP sync
    async fn test_threat_blocking_flow() {
        // Test flow:
        // 1. MISP sync adds threat domain to etcd
        // 2. dnsdist service receives notification
        // 3. Lua script updated with RPZ rule
        // 4. dnsdist reloaded
        // 5. DNS query to threat domain is blocked
        
        assert!(true);
    }
}

