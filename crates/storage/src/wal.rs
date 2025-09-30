use crate::{error::{Result, StorageError}, KVPair};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Deserialize)]
struct WALEntry {
    crc: u32,
    length: u32,
    data: KVPair,
}

/// Write-Ahead Log for durability guarantees
pub struct WriteAheadLog {
    file: tokio::sync::Mutex<File>,
    path: PathBuf,
    sequence: AtomicU64,
}

impl WriteAheadLog {
    pub async fn open<P: AsRef<Path>>(wal_dir: P) -> Result<Self> {
        let path = wal_dir.as_ref().join("wal.log");
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .await
            .map_err(|e| StorageError::Wal(format!("Failed to open WAL: {}", e)))?;
        
        Ok(Self {
            file: tokio::sync::Mutex::new(file),
            path,
            sequence: AtomicU64::new(0),
        })
    }
    
    pub async fn append(&self, kv_pair: &KVPair) -> Result<()> {
        let mut file = self.file.lock().await;
        
        // Serialize the KV pair
        let data = serde_json::to_vec(kv_pair)
            .map_err(|e| StorageError::Wal(format!("Failed to serialize WAL entry: {}", e)))?;
        
        // Calculate CRC
        let crc = crc32fast::hash(&data);
        
        let entry = WALEntry {
            crc,
            length: data.len() as u32,
            data: kv_pair.clone(),
        };
        
        // Serialize the complete entry
        let entry_bytes = serde_json::to_vec(&entry)
            .map_err(|e| StorageError::Wal(format!("Failed to serialize WAL entry: {}", e)))?;
        
        // Write length prefix, then entry
        file.write_u32(entry_bytes.len() as u32).await
            .map_err(|e| StorageError::Wal(format!("Failed to write WAL entry length: {}", e)))?;
        
        file.write_all(&entry_bytes).await
            .map_err(|e| StorageError::Wal(format!("Failed to write WAL entry: {}", e)))?;
        
        // Ensure durability
        file.sync_all().await
            .map_err(|e| StorageError::Wal(format!("Failed to sync WAL: {}", e)))?;
        
        self.sequence.fetch_add(1, Ordering::SeqCst);
        
        Ok(())
    }
    
    pub async fn recover(&self) -> Result<Vec<KVPair>> {
        let mut entries = Vec::new();
        
        // Open file for reading from beginning
        let mut read_file = File::open(&self.path).await
            .map_err(|e| StorageError::Wal(format!("Failed to open WAL for recovery: {}", e)))?;
        
        let mut position = 0;
        let file_size = read_file.metadata().await
            .map_err(|e| StorageError::Wal(format!("Failed to get WAL metadata: {}", e)))?
            .len();
        
        while position < file_size {
            // Read entry length
            let entry_len = match read_file.read_u32().await {
                Ok(len) => len,
                Err(_) => break, // End of file or corruption
            };
            position += 4;
            
            if position + entry_len as u64 > file_size {
                tracing::warn!("Truncated WAL entry at position {}, skipping", position);
                break;
            }
            
            // Read entry data
            let mut entry_bytes = vec![0u8; entry_len as usize];
            read_file.read_exact(&mut entry_bytes).await
                .map_err(|e| StorageError::Wal(format!("Failed to read WAL entry: {}", e)))?;
            position += entry_len as u64;
            
            // Deserialize entry
            match serde_json::from_slice::<WALEntry>(&entry_bytes) {
                Ok(entry) => {
                    // Verify CRC
                    let data_bytes = serde_json::to_vec(&entry.data)
                        .map_err(|e| StorageError::Wal(format!("Failed to serialize for CRC check: {}", e)))?;
                    
                    let expected_crc = crc32fast::hash(&data_bytes);
                    if entry.crc != expected_crc {
                        tracing::warn!("CRC mismatch in WAL entry, skipping");
                        continue;
                    }
                    
                    let sequence = entry.data.sequence;
                    entries.push(entry.data);
                    self.sequence.store(sequence + 1, Ordering::SeqCst);
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize WAL entry: {}, skipping", e);
                    continue;
                }
            }
        }
        
        tracing::info!("Recovered {} entries from WAL", entries.len());
        Ok(entries)
    }
    
    pub async fn truncate(&self) -> Result<()> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(0)).await
            .map_err(|e| StorageError::Wal(format!("Failed to seek WAL: {}", e)))?;
        
        file.set_len(0).await
            .map_err(|e| StorageError::Wal(format!("Failed to truncate WAL: {}", e)))?;
        
        file.sync_all().await
            .map_err(|e| StorageError::Wal(format!("Failed to sync WAL after truncate: {}", e)))?;
        
        self.sequence.store(0, Ordering::SeqCst);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_wal_append_and_recover() {
        let temp_dir = TempDir::new().unwrap();
        let wal = WriteAheadLog::open(temp_dir.path()).await.unwrap();
        
        let kv1 = KVPair::new(b"key1".to_vec(), b"value1".to_vec(), 1000, 1);
        let kv2 = KVPair::new(b"key2".to_vec(), b"value2".to_vec(), 1001, 2);
        let kv3 = KVPair::delete(b"key1".to_vec(), 1002, 3);
        
        // Append entries
        wal.append(&kv1).await.unwrap();
        wal.append(&kv2).await.unwrap();
        wal.append(&kv3).await.unwrap();
        
        // Recover entries
        let recovered = wal.recover().await.unwrap();
        
        assert_eq!(recovered.len(), 3);
        assert_eq!(recovered[0].key, kv1.key);
        assert_eq!(recovered[0].value, kv1.value);
        assert_eq!(recovered[1].key, kv2.key);
        assert_eq!(recovered[1].value, kv2.value);
        assert_eq!(recovered[2].key, kv3.key);
        assert!(recovered[2].value.is_none()); // Deletion
    }
}