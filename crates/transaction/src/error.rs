use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction not found: {0}")]
    NotFound(String),
    
    #[error("Transaction conflict")]
    Conflict,
    
    #[error("Transaction aborted")]
    Aborted,
    
    #[error("Deadlock detected")]
    Deadlock,
    
    #[error("Lock timeout")]
    LockTimeout,
    
    #[error("Invalid isolation level")]
    InvalidIsolation,
}

pub type Result<T> = std::result::Result<T, TransactionError>;