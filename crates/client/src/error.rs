use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Authentication failed")]
    Authentication,
    
    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;