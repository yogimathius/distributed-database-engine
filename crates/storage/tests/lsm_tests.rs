use nextdb_storage::{LSMTree, StorageConfig, KVPair};
use tempfile::TempDir;

#[tokio::test]
async fn test_lsm_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
    config.wal_dir = temp_dir.path().join("wal").to_string_lossy().to_string();
    
    let lsm = LSMTree::open(config).await.expect("Failed to open LSM tree");
    
    // Test basic put/get operations
    let key = b"test_key".to_vec();
    let value = b"test_value".to_vec();
    
    lsm.put(key.clone(), value.clone()).await.expect("Failed to put");
    
    let retrieved = lsm.get(&key).await.expect("Failed to get");
    assert_eq!(retrieved, Some(value));
    
    // Test key that doesn't exist
    let nonexistent = lsm.get(b"nonexistent").await.expect("Failed to get nonexistent");
    assert_eq!(nonexistent, None);
}

#[tokio::test]
async fn test_lsm_delete_operations() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
    config.wal_dir = temp_dir.path().join("wal").to_string_lossy().to_string();
    
    let lsm = LSMTree::open(config).await.expect("Failed to open LSM tree");
    
    let key = b"delete_test".to_vec();
    let value = b"to_be_deleted".to_vec();
    
    // Put then delete
    lsm.put(key.clone(), value.clone()).await.expect("Failed to put");
    let retrieved = lsm.get(&key).await.expect("Failed to get");
    assert_eq!(retrieved, Some(value));
    
    lsm.delete(&key).await.expect("Failed to delete");
    let deleted = lsm.get(&key).await.expect("Failed to get after delete");
    assert_eq!(deleted, None);
}

#[tokio::test]
async fn test_lsm_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
    config.wal_dir = temp_dir.path().join("wal").to_string_lossy().to_string();
    
    let key = b"persistent_key".to_vec();
    let value = b"persistent_value".to_vec();
    
    // Write data and close
    {
        let lsm = LSMTree::open(config.clone()).await.expect("Failed to open LSM tree");
        lsm.put(key.clone(), value.clone()).await.expect("Failed to put");
        lsm.flush().await.expect("Failed to flush");
    }
    
    // Reopen and verify data persists
    {
        let lsm = LSMTree::open(config).await.expect("Failed to reopen LSM tree");
        let retrieved = lsm.get(&key).await.expect("Failed to get after reopen");
        assert_eq!(retrieved, Some(value));
    }
}