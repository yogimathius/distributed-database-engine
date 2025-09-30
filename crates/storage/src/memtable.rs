use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Entry in the memtable with metadata
#[derive(Debug, Clone)]
pub struct MemTableEntry {
    pub value: Option<Vec<u8>>, // None for deletions
    pub sequence: u64,
}

/// In-memory sorted table using a skip list (BTreeMap for simplicity)
pub struct MemTable {
    data: BTreeMap<Vec<u8>, MemTableEntry>,
    size: AtomicUsize,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            size: AtomicUsize::new(0),
        }
    }
    
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>, sequence: u64) {
        let old_size = if let Some(old_entry) = self.data.get(&key) {
            key.len() + old_entry.value.as_ref().map_or(0, |v| v.len()) + 8 + 8 // key + value + seq + ts
        } else {
            0
        };
        
        let new_size = key.len() + value.len() + 8 + 8;
        
        let entry = MemTableEntry {
            value: Some(value),
            sequence,
        };
        
        self.data.insert(key, entry);
        
        // Update size accounting
        self.size.store(
            self.size.load(Ordering::Relaxed) - old_size + new_size,
            Ordering::Relaxed,
        );
    }
    
    pub fn delete(&mut self, key: Vec<u8>, sequence: u64) {
        let old_size = if let Some(old_entry) = self.data.get(&key) {
            key.len() + old_entry.value.as_ref().map_or(0, |v| v.len()) + 8 + 8
        } else {
            0
        };
        
        let new_size = key.len() + 8 + 8; // key + seq + ts (no value for tombstone)
        
        let entry = MemTableEntry {
            value: None, // Tombstone
            sequence,
        };
        
        self.data.insert(key, entry);
        
        self.size.store(
            self.size.load(Ordering::Relaxed) - old_size + new_size,
            Ordering::Relaxed,
        );
    }
    
    pub fn get(&self, key: &[u8]) -> Option<Option<Vec<u8>>> {
        self.data.get(key).map(|entry| entry.value.clone())
    }
    
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &MemTableEntry)> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memtable_basic_operations() {
        let mut memtable = MemTable::new();
        
        // Test put and get
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        
        memtable.put(key.clone(), value.clone(), 1);
        assert_eq!(memtable.get(&key), Some(Some(value)));
        
        // Test size tracking
        assert!(memtable.size() > 0);
        
        // Test delete (tombstone)
        memtable.delete(key.clone(), 2);
        assert_eq!(memtable.get(&key), Some(None));
    }
    
    #[test]
    fn test_memtable_ordering() {
        let mut memtable = MemTable::new();
        
        // Insert keys in random order
        memtable.put(b"c".to_vec(), b"value_c".to_vec(), 1);
        memtable.put(b"a".to_vec(), b"value_a".to_vec(), 2);
        memtable.put(b"b".to_vec(), b"value_b".to_vec(), 3);
        
        // Verify they come out sorted
        let keys: Vec<_> = memtable.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);
    }
}