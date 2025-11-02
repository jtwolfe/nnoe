#[cfg(test)]
mod tests {
    use nnoe_agent::config::CacheConfig;
    use nnoe_agent::sled_cache::cache::CacheManager;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_put_get() {
        let config = CacheConfig {
            path: "/tmp/test-cache-put-get".to_string(),
            default_ttl_secs: 60,
            max_size_mb: 10,
        };
        let cache = CacheManager::new(&config).unwrap();

        cache.put("test-key", b"test-value").unwrap();
        let value = cache.get("test-key").unwrap();

        assert_eq!(value, Some(b"test-value".to_vec()));

        // Cleanup
        std::fs::remove_file("/tmp/test-cache-put-get").ok();
    }

    #[tokio::test]
    async fn test_cache_ttl_expiry() {
        let config = CacheConfig {
            path: "/tmp/test-cache-ttl".to_string(),
            default_ttl_secs: 1,
            max_size_mb: 10,
        };
        let cache = CacheManager::new(&config).unwrap();

        cache.put("test-key", b"test-value").unwrap();

        // Value should be available immediately
        assert!(cache.get("test-key").unwrap().is_some());

        // Wait for TTL to expire
        sleep(Duration::from_secs(2)).await;

        // Value should be expired
        assert!(cache.get("test-key").unwrap().is_none());

        // Cleanup
        std::fs::remove_file("/tmp/test-cache-ttl").ok();
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let config = CacheConfig {
            path: "/tmp/test-cache-delete".to_string(),
            default_ttl_secs: 60,
            max_size_mb: 10,
        };
        let cache = CacheManager::new(&config).unwrap();

        cache.put("test-key", b"test-value").unwrap();
        cache.delete("test-key").unwrap();

        assert!(cache.get("test-key").unwrap().is_none());

        // Cleanup
        std::fs::remove_file("/tmp/test-cache-delete").ok();
    }

    #[tokio::test]
    async fn test_cache_list_prefix() {
        let config = CacheConfig {
            path: "/tmp/test-cache-list".to_string(),
            default_ttl_secs: 60,
            max_size_mb: 10,
        };
        let cache = CacheManager::new(&config).unwrap();

        cache.put("prefix/key1", b"value1").unwrap();
        cache.put("prefix/key2", b"value2").unwrap();
        cache.put("other/key3", b"value3").unwrap();

        let results = cache.list_prefix("prefix/").unwrap();
        assert_eq!(results.len(), 2);

        // Cleanup
        std::fs::remove_file("/tmp/test-cache-list").ok();
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let config = CacheConfig {
            path: "/tmp/test-cache-clear".to_string(),
            default_ttl_secs: 60,
            max_size_mb: 10,
        };
        let cache = CacheManager::new(&config).unwrap();

        cache.put("key1", b"value1").unwrap();
        cache.put("key2", b"value2").unwrap();

        cache.clear().unwrap();

        assert!(cache.get("key1").unwrap().is_none());
        assert!(cache.get("key2").unwrap().is_none());

        // Cleanup
        std::fs::remove_file("/tmp/test-cache-clear").ok();
    }
}
