use crate::config::CacheConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    value: Vec<u8>,
    timestamp: u64,
}

pub struct CacheManager {
    db: Arc<Db>,
    config: CacheConfig,
    sweep_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
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
        // Note: sled doesn't have set_cache_capacity in all versions
        // Cache size is typically managed by sled internally
        // db.set_cache_capacity(config.max_size_mb * 1024 * 1024);

        info!(
            "Cache initialized successfully (TTL: {}s, Max size: {}MB)",
            config.default_ttl_secs, config.max_size_mb
        );

        let db = Arc::new(db);
        let sweep_handle = Arc::new(RwLock::new(None));

        // Start background sweep task - spawn in runtime if available
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let db_clone = Arc::clone(&db);
            let config_clone = config.clone();
            let sweep_handle_clone = Arc::clone(&sweep_handle);
            handle.spawn(async move {
                // Create a temporary manager just for starting the sweep task
                let temp_manager = Self {
                    db: db_clone,
                    config: config_clone,
                    sweep_handle: sweep_handle_clone,
                };
                temp_manager.start_sweep_task_internal();
            });
        }

        let manager = Self {
            db,
            config: config.clone(),
            sweep_handle,
        };

        Ok(manager)
    }

    fn start_sweep_task_internal(&self) {
        let db = Arc::clone(&self.db);
        let ttl_secs = self.config.default_ttl_secs;
        let max_size_bytes = self.config.max_size_mb * 1024 * 1024;
        let sweep_handle = Arc::clone(&self.sweep_handle);

        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(60)); // Sweep every 60 seconds

            loop {
                interval_timer.tick().await;

                // Clean expired entries
                if let Err(e) = Self::sweep_expired(&db, ttl_secs).await {
                    warn!("Error during cache sweep: {}", e);
                }

                // Check size and evict if needed
                if let Err(e) = Self::enforce_size_limit(&db, max_size_bytes).await {
                    warn!("Error enforcing cache size limit: {}", e);
                }
            }
        });

        // Store handle
        tokio::runtime::Handle::try_current().map(|h| {
            h.spawn(async move {
                *sweep_handle.write().await = Some(handle);
            })
        });
    }

    async fn sweep_expired(db: &Arc<Db>, ttl_secs: u64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut expired_count = 0;
        let mut keys_to_delete = Vec::new();

        for item in db.iter() {
            let (key, value) = item?;

            // Try to deserialize as CacheEntry
            if let Ok(entry) = bincode::deserialize::<CacheEntry>(&value) {
                if now.saturating_sub(entry.timestamp) > ttl_secs {
                    keys_to_delete.push(key);
                    expired_count += 1;
                }
            } else {
                // Legacy entry format (direct value), check if it has a timestamp key
                let timestamp_key = format!("{}_ts", String::from_utf8_lossy(&key));
                if let Ok(Some(ts_bytes)) = db.get(&timestamp_key) {
                    if let Ok(ts) = bincode::deserialize::<u64>(&ts_bytes) {
                        if now.saturating_sub(ts) > ttl_secs {
                            keys_to_delete.push(key);
                            keys_to_delete.push(timestamp_key.into_bytes().into());
                            expired_count += 1;
                        }
                    }
                }
            }
        }

        // Delete expired keys
        for key in keys_to_delete {
            db.remove(key)?;
        }

        if expired_count > 0 {
            debug!("Cleaned {} expired cache entries", expired_count);
        }

        Ok(())
    }

    async fn enforce_size_limit(db: &Arc<Db>, max_size_bytes: u64) -> Result<()> {
        // Estimate current size (rough approximation)
        let mut total_size = 0u64;
        let mut entries: Vec<(Vec<u8>, u64, u64)> = Vec::new(); // (key, size, timestamp)

        for item in db.iter() {
            let (key, value) = item?;
            let key_size = key.len() as u64;
            let value_size = value.len() as u64;
            let entry_size = key_size + value_size;

            total_size += entry_size;

            // Extract timestamp for LRU
            let timestamp = if let Ok(entry) = bincode::deserialize::<CacheEntry>(&value) {
                entry.timestamp
            } else {
                // Legacy format, try timestamp key
                let timestamp_key = format!("{}_ts", String::from_utf8_lossy(&key));
                if let Ok(Some(ts_bytes)) = db.get(&timestamp_key) {
                    bincode::deserialize::<u64>(&ts_bytes).unwrap_or(0)
                } else {
                    0
                }
            };

            entries.push((key.to_vec(), entry_size, timestamp));
        }

        if total_size > max_size_bytes {
            // Sort by timestamp (oldest first) for LRU eviction
            entries.sort_by_key(|(_, _, ts)| *ts);

            let mut evicted = 0;
            for (key, size, _) in entries {
                if total_size <= max_size_bytes {
                    break;
                }

                // Also try to delete timestamp key
                let timestamp_key = format!("{}_ts", String::from_utf8_lossy(&key));
                db.remove(&timestamp_key)?;
                db.remove(&key)?;

                total_size -= size;
                evicted += 1;
            }

            if evicted > 0 {
                info!("Evicted {} cache entries to enforce size limit", evicted);
            }
        }

        Ok(())
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        debug!("Cache get: {}", key);

        match self.db.get(key) {
            Ok(Some(value)) => {
                // Try to deserialize as CacheEntry first
                if let Ok(entry) = bincode::deserialize::<CacheEntry>(&value) {
                    // Check TTL
                    let now = Self::get_timestamp();
                    if now.saturating_sub(entry.timestamp) > self.config.default_ttl_secs {
                        // Expired, delete and return None
                        let _ = self.db.remove(key);
                        return Ok(None);
                    }
                    Ok(Some(entry.value))
                } else {
                    // Legacy format - check for timestamp key
                    let timestamp_key = format!("{}_ts", key);
                    if let Ok(Some(ts_bytes)) = self.db.get(&timestamp_key) {
                        if let Ok(timestamp) = bincode::deserialize::<u64>(&ts_bytes) {
                            let now = Self::get_timestamp();
                            if now.saturating_sub(timestamp) > self.config.default_ttl_secs {
                                // Expired, delete and return None
                                let _ = self.db.remove(key);
                                let _ = self.db.remove(&timestamp_key);
                                return Ok(None);
                            }
                        }
                    }
                    // No TTL info, return as-is (backward compatibility)
                    Ok(Some(value.to_vec()))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Cache get error for key {}: {}", key, e);
                Err(anyhow::anyhow!("Cache error: {}", e))
            }
        }
    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        debug!("Cache put: {} ({} bytes)", key, value.len());

        let timestamp = Self::get_timestamp();
        let entry = CacheEntry {
            value: value.to_vec(),
            timestamp,
        };

        let serialized = bincode::serialize(&entry).context("Failed to serialize cache entry")?;

        self.db
            .insert(key, serialized)
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
        let now = Self::get_timestamp();

        for item in self.db.scan_prefix(prefix) {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key).to_string();

            // Skip timestamp keys
            if key_str.ends_with("_ts") {
                continue;
            }

            // Deserialize and check TTL
            if let Ok(entry) = bincode::deserialize::<CacheEntry>(&value) {
                if now.saturating_sub(entry.timestamp) <= self.config.default_ttl_secs {
                    results.push((key_str, entry.value));
                }
            } else {
                // Legacy format
                results.push((key_str, value.to_vec()));
            }
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
