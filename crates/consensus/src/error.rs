use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Not leader")]
    NotLeader,
    
    #[error("Election timeout")]
    ElectionTimeout,
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Consensus error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ConsensusError>;