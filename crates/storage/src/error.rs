use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Key not found: {key:?}")]
    KeyNotFound { key: Vec<u8> },
    
    #[error("Corruption detected: {0}")]
    Corruption(String),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("WAL error: {0}")]
    Wal(String),
    
    #[error("Compaction error: {0}")]
    Compaction(String),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;