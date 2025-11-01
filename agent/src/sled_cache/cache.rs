use crate::config::CacheConfig;
use anyhow::{Context, Result};
use sled::Db;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct CacheManager {
    db: Arc<Db>,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: &CacheConfig) -> Result<Self> {
        info!("Initializing cache at path: {}", config.path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(&config.path).parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create cache directory: {:?}", parent))?;
        }

        let db = sled::open(&config.path)
            .context(format!("Failed to open sled database at {}", config.path))?;

        // Configure cache size
        db.set_cache_capacity(config.max_size_mb * 1024 * 1024);

        info!("Cache initialized successfully");

        Ok(Self {
            db: Arc::new(db),
            config: config.clone(),
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        debug!("Cache get: {}", key);
        match self.db.get(key) {
            Ok(Some(value)) => Ok(Some(value.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Cache get error for key {}: {}", key, e);
                Err(anyhow::anyhow!("Cache error: {}", e))
            }
        }
    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        debug!("Cache put: {} ({} bytes)", key, value.len());
        self.db
            .insert(key, value)
            .context(format!("Failed to insert key into cache: {}", key))?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        debug!("Cache delete: {}", key);
        self.db
            .remove(key)
            .context(format!("Failed to delete key from cache: {}", key))?;
        Ok(())
    }

    pub fn list_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        debug!("Cache list prefix: {}", prefix);
        let mut results = Vec::new();

        for item in self.db.scan_prefix(prefix) {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            results.push((key_str, value.to_vec()));
        }

        Ok(results)
    }

    pub fn clear(&self) -> Result<()> {
        info!("Clearing cache");
        self.db.clear()?;
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.db.len()
    }

    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

impl Drop for CacheManager {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            warn!("Error flushing cache on drop: {}", e);
        }
    }
}

