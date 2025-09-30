use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Simple LRU cache for hot data blocks
pub struct BlockCache {
    cache: RwLock<LRUCache>,
}

struct LRUCache {
    data: HashMap<String, CacheEntry>,
    capacity: usize,
    current_size: usize,
    access_order: Vec<String>,
}

struct CacheEntry {
    value: Vec<u8>,
    size: usize,
}

impl BlockCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(LRUCache {
                data: HashMap::new(),
                capacity,
                current_size: 0,
                access_order: Vec::new(),
            }),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.write();
        
        if let Some(entry) = cache.data.get(key) {
            let value = entry.value.clone();
            
            // Move to end (most recently used)
            if let Some(pos) = cache.access_order.iter().position(|k| k == key) {
                cache.access_order.remove(pos);
            }
            cache.access_order.push(key.to_string());
            
            return Some(value);
        }
        
        None
    }
    
    pub fn put(&self, key: String, value: Vec<u8>) {
        let mut cache = self.cache.write();
        let entry_size = key.len() + value.len();
        
        // Remove existing entry if present
        if let Some(old_entry) = cache.data.remove(&key) {
            cache.current_size -= key.len() + old_entry.size;
            if let Some(pos) = cache.access_order.iter().position(|k| k == &key) {
                cache.access_order.remove(pos);
            }
        }
        
        // Evict entries if necessary
        while cache.current_size + entry_size > cache.capacity && !cache.access_order.is_empty() {
            if let Some(lru_key) = cache.access_order.first().cloned() {
                if let Some(entry) = cache.data.remove(&lru_key) {
                    cache.current_size -= lru_key.len() + entry.size;
                }
                cache.access_order.remove(0);
            } else {
                break;
            }
        }
        
        // Insert new entry if it fits
        if entry_size <= cache.capacity {
            cache.data.insert(key.clone(), CacheEntry {
                value,
                size: entry_size,
            });
            cache.access_order.push(key);
            cache.current_size += entry_size;
        }
    }
    
    pub fn size(&self) -> usize {
        self.cache.read().current_size
    }
    
    pub fn capacity(&self) -> usize {
        self.cache.read().capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = BlockCache::new(100);
        
        // Test put and get
        cache.put("key1".to_string(), b"value1".to_vec());
        assert_eq!(cache.get("key1"), Some(b"value1".to_vec()));
        
        // Test nonexistent key
        assert_eq!(cache.get("nonexistent"), None);
    }
    
    #[test]
    fn test_cache_eviction() {
        let cache = BlockCache::new(50); // Small capacity
        
        // Fill cache
        cache.put("key1".to_string(), b"value1".to_vec()); // ~12 bytes
        cache.put("key2".to_string(), b"value2".to_vec()); // ~12 bytes
        cache.put("key3".to_string(), b"value3".to_vec()); // ~12 bytes
        
        // Access key1 to make it recently used
        cache.get("key1");
        
        // Add another entry that should evict key2 (least recently used)
        cache.put("key4".to_string(), b"value4444444".to_vec()); // Larger value
        
        // key1 should still be there (recently accessed)
        assert_eq!(cache.get("key1"), Some(b"value1".to_vec()));
        
        // key4 should be there (just inserted)
        assert_eq!(cache.get("key4"), Some(b"value4444444".to_vec()));
    }
}