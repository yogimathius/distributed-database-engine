use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NextDBError {
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, NextDBError>;