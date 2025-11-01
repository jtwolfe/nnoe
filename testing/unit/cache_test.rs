// Unit tests for cache management

#[cfg(test)]
mod tests {
    use nnoe_agent::sled_cache::CacheManager;
    use nnoe_agent::config::CacheConfig;
    use std::fs;

    fn create_test_cache() -> CacheManager {
        let test_dir = "/tmp/nnoe-test-cache";
        let _ = fs::remove_dir_all(test_dir);
        
        let config = CacheConfig {
            path: test_dir.to_string(),
            default_ttl_secs: 300,
            max_size_mb: 10,
        };
        
        CacheManager::new(&config).unwrap()
    }

    #[test]
    fn test_cache_put_get() {
        let cache = create_test_cache();
        
        cache.put("test-key", b"test-value").unwrap();
        let value = cache.get("test-key").unwrap();
        
        assert_eq!(value, Some(b"test-value".to_vec()));
    }

    #[test]
    fn test_cache_delete() {
        let cache = create_test_cache();
        
        cache.put("test-key", b"test-value").unwrap();
        cache.delete("test-key").unwrap();
        
        let value = cache.get("test-key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_cache_prefix_list() {
        let cache = create_test_cache();
        
        cache.put("prefix/key1", b"value1").unwrap();
        cache.put("prefix/key2", b"value2").unwrap();
        cache.put("other/key", b"value3").unwrap();
        
        let results = cache.list_prefix("prefix/").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_cache_clear() {
        let cache = create_test_cache();
        
        cache.put("key1", b"value1").unwrap();
        cache.put("key2", b"value2").unwrap();
        cache.clear().unwrap();
        
        assert_eq!(cache.size(), 0);
    }
}

