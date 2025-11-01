// Mock etcd server for testing

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct MockEtcdServer {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    watch_channels: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
}

impl MockEtcdServer {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            watch_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        let mut data = self.data.write().unwrap();
        data.insert(key.clone(), value.clone());
        
        // Notify watchers
        if let Some(sender) = self.watch_channels.read().unwrap().get(&key) {
            let _ = sender.send(value);
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    pub fn delete(&self, key: &str) {
        let mut data = self.data.write().unwrap();
        data.remove(key);
    }

    pub fn list_prefix(&self, prefix: &str) -> Vec<(String, Vec<u8>)> {
        let data = self.data.read().unwrap();
        data.iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn watch(&self, key: String) -> broadcast::Receiver<Vec<u8>> {
        let mut channels = self.watch_channels.write().unwrap();
        
        if !channels.contains_key(&key) {
            let (tx, _) = broadcast::channel(100);
            channels.insert(key.clone(), tx);
        }
        
        channels.get(&key).unwrap().subscribe()
    }

    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }
}

impl Default for MockEtcdServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_etcd_put_get() {
        let server = MockEtcdServer::new();
        server.put("test-key".to_string(), b"test-value".to_vec());
        
        let value = server.get("test-key");
        assert_eq!(value, Some(b"test-value".to_vec()));
    }

    #[tokio::test]
    async fn test_mock_etcd_watch() {
        let server = MockEtcdServer::new();
        let mut receiver = server.watch("test-key".to_string());
        
        server.put("test-key".to_string(), b"new-value".to_vec());
        
        let value = receiver.recv().await.unwrap();
        assert_eq!(value, b"new-value".to_vec());
    }
}

