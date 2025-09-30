use crate::{
    error::{Result, StorageError},
    cache::BlockCache,
    compression::{compress, decompress, CompressionType},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use std::collections::BTreeMap;

const BLOCK_SIZE: usize = 4096;
const FOOTER_SIZE: usize = 48;

#[derive(Debug, Serialize, Deserialize)]
struct SSTableFooter {
    index_offset: u64,
    index_size: u64,
    bloom_filter_offset: u64,
    bloom_filter_size: u64,
    compression: CompressionType,
    num_entries: u64,
    crc: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexEntry {
    key: Vec<u8>,
    offset: u64,
    size: u32,
}

/// Immutable sorted table stored on disk
pub struct SSTable {
    file_path: PathBuf,
    footer: SSTableFooter,
    index: BTreeMap<Vec<u8>, IndexEntry>,
}

impl SSTable {
    pub async fn open<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path = file_path.as_ref().to_path_buf();
        let mut file = File::open(&path).await
            .map_err(|e| StorageError::Io(e))?;
        
        // Read footer from end of file
        let file_size = file.metadata().await?.len();
        if file_size < FOOTER_SIZE as u64 {
            return Err(StorageError::Corruption("SSTable too small".to_string()));
        }
        
        file.seek(SeekFrom::End(-(FOOTER_SIZE as i64))).await?;
        let mut footer_bytes = vec![0u8; FOOTER_SIZE];
        file.read_exact(&mut footer_bytes).await?;
        
        let footer: SSTableFooter = serde_json::from_slice(&footer_bytes)
            .map_err(|e| StorageError::Corruption(format!("Invalid footer: {}", e)))?;
        
        // Read and parse index
        file.seek(SeekFrom::Start(footer.index_offset)).await?;
        let mut index_bytes = vec![0u8; footer.index_size as usize];
        file.read_exact(&mut index_bytes).await?;
        
        let decompressed = decompress(&index_bytes, &footer.compression)?;
        let index_entries: Vec<IndexEntry> = serde_json::from_slice(&decompressed)
            .map_err(|e| StorageError::Corruption(format!("Invalid index: {}", e)))?;
        
        let mut index = BTreeMap::new();
        for entry in index_entries {
            index.insert(entry.key.clone(), entry);
        }
        
        Ok(Self {
            file_path: path,
            footer,
            index,
        })
    }
    
    pub async fn get(&self, key: &[u8], cache: &BlockCache) -> Result<Option<Option<Vec<u8>>>> {
        // Find the index entry for this key or the next larger key
        let entry = self.index.range(..=key.to_vec())
            .next_back()
            .map(|(_, entry)| entry);
        
        if let Some(entry) = entry {
            // Check cache first
            let cache_key = format!("{}:{}", self.file_path.display(), entry.offset);
            if let Some(cached_value) = cache.get(&cache_key) {
                return Ok(Some(Some(cached_value)));
            }
            
            // Read from disk
            let mut file = File::open(&self.file_path).await?;
            file.seek(SeekFrom::Start(entry.offset)).await?;
            
            let mut compressed_data = vec![0u8; entry.size as usize];
            file.read_exact(&mut compressed_data).await?;
            
            let decompressed = decompress(&compressed_data, &self.footer.compression)?;
            
            // Parse the block to find the exact key
            // TODO: Implement proper block parsing
            // For now, just return a placeholder value since we have serialization issues with binary keys
            if let Some(v) = cache.get(&cache_key) {
                return Ok(Some(Some(v)));
            }
            
            // TODO: Parse decompressed data into key-value pairs
            // This is a placeholder implementation
            if !decompressed.is_empty() {
                // For now, return Some(None) to indicate key exists but value needs proper parsing
                return Ok(Some(None));
            }
        }
        
        Ok(None)
    }
    
    pub fn key_range(&self) -> Option<(&[u8], &[u8])> {
        if self.index.is_empty() {
            return None;
        }
        
        let first_key = self.index.keys().next().unwrap();
        let last_key = self.index.keys().next_back().unwrap();
        Some((first_key, last_key))
    }
    
    pub fn file_size(&self) -> u64 {
        // This would need to be populated during creation
        0 // Simplified for now
    }
}

/// Builder for creating new SSTables
pub struct SSTableBuilder {
    file_path: PathBuf,
    file: File,
    compression: CompressionType,
    current_block: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    blocks_written: u64,
    index_entries: Vec<IndexEntry>,
    current_offset: u64,
    num_entries: u64,
}

impl SSTableBuilder {
    pub async fn new<P: AsRef<Path>>(
        file_path: P, 
        compression: CompressionType
    ) -> Result<Self> {
        let path = file_path.as_ref().to_path_buf();
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .await?;
        
        Ok(Self {
            file_path: path,
            file,
            compression,
            current_block: BTreeMap::new(),
            blocks_written: 0,
            index_entries: Vec::new(),
            current_offset: 0,
            num_entries: 0,
        })
    }
    
    pub fn add(&mut self, key: &[u8], value: &Option<Vec<u8>>, _sequence: u64) -> Result<()> {
        self.current_block.insert(key.to_vec(), value.clone());
        self.num_entries += 1;
        
        // Check if block is full
        let block_size = self.estimate_block_size();
        if block_size >= BLOCK_SIZE {
            self.flush_current_block()?;
        }
        
        Ok(())
    }
    
    pub async fn finish(mut self) -> Result<SSTable> {
        
        // Flush any remaining data
        if !self.current_block.is_empty() {
            self.flush_current_block()?;
        }
        
        // Write index
        let index_offset = self.current_offset;
        let index_data = serde_json::to_vec(&self.index_entries)?;
        let compressed_index = compress(&index_data, &self.compression)?;
        
        self.file.write_all(&compressed_index).await?;
        let index_size = compressed_index.len() as u64;
        self.current_offset += index_size;
        
        // Write footer
        let footer = SSTableFooter {
            index_offset,
            index_size,
            bloom_filter_offset: 0, // Simplified - no bloom filter yet
            bloom_filter_size: 0,
            compression: self.compression.clone(),
            num_entries: self.num_entries,
            crc: 0, // Simplified - no CRC yet
        };
        
        let footer_data = serde_json::to_vec(&footer)?;
        if footer_data.len() > FOOTER_SIZE {
            return Err(StorageError::Internal("Footer too large".to_string()));
        }
        
        let mut footer_bytes = vec![0u8; FOOTER_SIZE];
        footer_bytes[..footer_data.len()].copy_from_slice(&footer_data);
        self.file.write_all(&footer_bytes).await?;
        
        self.file.sync_all().await?;
        drop(self.file);
        
        // Open the completed SSTable
        SSTable::open(&self.file_path).await
    }
    
    fn flush_current_block(&mut self) -> Result<()> {
        if self.current_block.is_empty() {
            return Ok(());
        }
        
        // Record index entry for first key in block
        if let Some(first_key) = self.current_block.keys().next().cloned() {
            self.index_entries.push(IndexEntry {
                key: first_key,
                offset: self.current_offset,
                size: 0, // Will be updated after compression
            });
        }
        
        let block_data = serde_json::to_vec(&self.current_block)?;
        let compressed_block = compress(&block_data, &self.compression)?;
        
        // Update the size in the last index entry
        if let Some(last_entry) = self.index_entries.last_mut() {
            last_entry.size = compressed_block.len() as u32;
        }
        
        // Note: In real implementation, we would write to file here
        // For now, just track the offset
        self.current_offset += compressed_block.len() as u64;
        self.blocks_written += 1;
        
        self.current_block.clear();
        Ok(())
    }
    
    fn estimate_block_size(&self) -> usize {
        let serialized = serde_json::to_vec(&self.current_block).unwrap_or_default();
        serialized.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::BlockCache;
    use tempfile::TempDir;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_sstable_builder_and_reader() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sst");
        
        // Build SSTable
        {
            let mut builder = SSTableBuilder::new(&file_path, CompressionType::None).await.unwrap();
            builder.add(b"key1", &Some(b"value1".to_vec()), 1).unwrap();
            builder.add(b"key2", &Some(b"value2".to_vec()), 2).unwrap();
            builder.add(b"key3", &None, 3).unwrap(); // Deletion
            
            let _sstable = builder.finish().await.unwrap();
        }
        
        // Read SSTable
        {
            let sstable = SSTable::open(&file_path).await.unwrap();
            let cache = Arc::new(BlockCache::new(1024 * 1024));
            
            let result1 = sstable.get(b"key1", &cache).await.unwrap();
            assert_eq!(result1, Some(Some(b"value1".to_vec())));
            
            let result2 = sstable.get(b"key2", &cache).await.unwrap();
            assert_eq!(result2, Some(Some(b"value2".to_vec())));
            
            let result3 = sstable.get(b"key3", &cache).await.unwrap();
            assert_eq!(result3, Some(None)); // Deletion marker
            
            let result4 = sstable.get(b"nonexistent", &cache).await.unwrap();
            assert_eq!(result4, None);
        }
    }
}