use crate::{
    error::{Result, StorageError},
    memtable::MemTable,
    wal::WriteAheadLog,
    sstable::{SSTable, SSTableBuilder},
    cache::BlockCache,
    compression::CompressionType,
    StorageConfig, KVPair,
};

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use parking_lot::Mutex;

/// LSM-Tree storage engine implementation
pub struct LSMTree {
    config: StorageConfig,
    sequence_number: AtomicU64,
    
    // Active memtable for writes
    active_memtable: Arc<RwLock<MemTable>>,
    
    // Immutable memtables waiting for flush
    immutable_memtables: Arc<Mutex<Vec<Arc<MemTable>>>>,
    
    // Write-ahead log for durability
    wal: Arc<WriteAheadLog>,
    
    // SSTable levels (Level 0, Level 1, ...)
    levels: Arc<RwLock<Vec<Vec<Arc<SSTable>>>>>,
    
    // Block cache for hot data
    cache: Arc<BlockCache>,
}

impl LSMTree {
    pub async fn open(config: StorageConfig) -> Result<Self> {
        // Create directories if they don't exist
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| StorageError::Config(format!("Failed to create data dir: {}", e)))?;
        std::fs::create_dir_all(&config.wal_dir)
            .map_err(|e| StorageError::Config(format!("Failed to create WAL dir: {}", e)))?;
        
        // Initialize WAL
        let wal = Arc::new(WriteAheadLog::open(&config.wal_dir).await?);
        
        // Initialize block cache
        let cache = Arc::new(BlockCache::new(config.cache_size_mb * 1024 * 1024));
        
        // Initialize empty levels
        let levels = Arc::new(RwLock::new(vec![vec![]; config.max_levels]));
        
        // Create initial memtable
        let active_memtable = Arc::new(RwLock::new(MemTable::new()));
        
        let lsm = Self {
            config,
            sequence_number: AtomicU64::new(0),
            active_memtable,
            immutable_memtables: Arc::new(Mutex::new(Vec::new())),
            wal,
            levels,
            cache,
        };
        
        // Recover from WAL if needed
        lsm.recover_from_wal().await?;
        
        Ok(lsm)
    }
    
    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let seq = self.sequence_number.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        let kv_pair = KVPair::new(key.clone(), value, timestamp, seq);
        
        // Write to WAL first for durability
        self.wal.append(&kv_pair).await?;
        
        // Write to active memtable
        {
            let mut memtable = self.active_memtable.write().await;
            memtable.put(key, kv_pair.value.clone().unwrap(), seq);
            
            // Check if memtable is full
            if memtable.size() >= self.config.memtable_size_mb * 1024 * 1024 {
                drop(memtable); // Release lock before rotation
                self.rotate_memtable().await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Check active memtable first
        {
            let memtable = self.active_memtable.read().await;
            if let Some(value) = memtable.get(key) {
                return Ok(value);
            }
        }
        
        // Check immutable memtables
        {
            let immutable = self.immutable_memtables.lock();
            for memtable in immutable.iter().rev() {
                if let Some(value) = memtable.get(key) {
                    return Ok(value);
                }
            }
        }
        
        // Check SSTables from newest to oldest
        let levels = self.levels.read().await;
        for level in levels.iter() {
            for sstable in level.iter().rev() {
                if let Some(value) = sstable.get(key, &self.cache).await? {
                    return Ok(value);
                }
            }
        }
        
        Ok(None)
    }
    
    pub async fn delete(&self, key: &[u8]) -> Result<()> {
        let seq = self.sequence_number.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        let kv_pair = KVPair::delete(key.to_vec(), timestamp, seq);
        
        // Write tombstone to WAL
        self.wal.append(&kv_pair).await?;
        
        // Write tombstone to memtable
        {
            let mut memtable = self.active_memtable.write().await;
            memtable.delete(key.to_vec(), seq);
            
            if memtable.size() >= self.config.memtable_size_mb * 1024 * 1024 {
                drop(memtable);
                self.rotate_memtable().await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn flush(&self) -> Result<()> {
        self.rotate_memtable().await?;
        self.flush_immutable_memtables().await?;
        Ok(())
    }
    
    async fn rotate_memtable(&self) -> Result<()> {
        let old_memtable;
        {
            let mut active = self.active_memtable.write().await;
            old_memtable = std::mem::replace(&mut *active, MemTable::new());
        }
        
        if !old_memtable.is_empty() {
            let old_memtable = Arc::new(old_memtable);
            self.immutable_memtables.lock().push(old_memtable);
        }
        
        self.flush_immutable_memtables().await
    }
    
    async fn flush_immutable_memtables(&self) -> Result<()> {
        let memtables_to_flush = {
            let mut immutable = self.immutable_memtables.lock();
            std::mem::take(&mut *immutable)
        };
        
        for memtable in memtables_to_flush {
            self.flush_memtable_to_l0(memtable).await?;
        }
        
        Ok(())
    }
    
    async fn flush_memtable_to_l0(&self, memtable: Arc<MemTable>) -> Result<()> {
        if memtable.is_empty() {
            return Ok(());
        }
        
        let file_number = self.sequence_number.fetch_add(1, Ordering::SeqCst);
        let file_path = Path::new(&self.config.data_dir)
            .join(format!("{}.sst", file_number));
        
        let mut builder = SSTableBuilder::new(
            file_path,
            self.config.compression.clone(),
        ).await?;
        
        for (key, entry) in memtable.iter() {
            builder.add(key, &entry.value, entry.sequence)?;
        }
        
        let sstable = builder.finish().await?;
        
        // Add to level 0
        {
            let mut levels = self.levels.write().await;
            levels[0].push(Arc::new(sstable));
        }
        
        // Check if L0 compaction is needed
        {
            let levels = self.levels.read().await;
            if levels[0].len() >= self.config.l0_compaction_trigger {
                // Schedule compaction (simplified for now)
                tracing::info!("L0 compaction triggered");
            }
        }
        
        Ok(())
    }
    
    async fn recover_from_wal(&self) -> Result<()> {
        let entries = self.wal.recover().await?;
        
        let mut memtable = self.active_memtable.write().await;
        for entry in entries {
            if let Some(value) = entry.value {
                memtable.put(entry.key, value, entry.sequence);
            } else {
                memtable.delete(entry.key, entry.sequence);
            }
            
            // Update sequence number
            let current_seq = self.sequence_number.load(Ordering::SeqCst);
            if entry.sequence >= current_seq {
                self.sequence_number.store(entry.sequence + 1, Ordering::SeqCst);
            }
        }
        
        Ok(())
    }
}