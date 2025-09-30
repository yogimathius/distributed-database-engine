pub mod lsm;
pub mod wal;
pub mod memtable;
pub mod sstable;
pub mod cache;
pub mod compression;
pub mod error;

pub use error::{StorageError, Result};
pub use lsm::LSMTree;
pub use wal::WriteAheadLog;
pub use memtable::MemTable;
pub use sstable::SSTable;
pub use cache::BlockCache;

use serde::{Deserialize, Serialize};

/// Storage engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    pub wal_dir: String,
    pub memtable_size_mb: usize,
    pub l0_compaction_trigger: usize,
    pub max_levels: usize,
    pub target_file_size_mb: usize,
    pub compression: CompressionType,
    pub cache_size_mb: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            wal_dir: "./wal".to_string(),
            memtable_size_mb: 64,
            l0_compaction_trigger: 4,
            max_levels: 7,
            target_file_size_mb: 64,
            compression: CompressionType::LZ4,
            cache_size_mb: 256,
        }
    }
}

pub use compression::CompressionType;

/// Key-Value pair with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVPair {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>, // None for deletions
    pub timestamp: u64,
    pub sequence: u64,
}

impl KVPair {
    pub fn new(key: Vec<u8>, value: Vec<u8>, timestamp: u64, sequence: u64) -> Self {
        Self {
            key,
            value: Some(value),
            timestamp,
            sequence,
        }
    }
    
    pub fn delete(key: Vec<u8>, timestamp: u64, sequence: u64) -> Self {
        Self {
            key,
            value: None,
            timestamp,
            sequence,
        }
    }
    
    pub fn is_deleted(&self) -> bool {
        self.value.is_none()
    }
}