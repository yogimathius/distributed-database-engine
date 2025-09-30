use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Plan error: {0}")]
    Plan(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Invalid query: {0}")]
    Invalid(String),
    
    #[error("Table not found: {0}")]
    TableNotFound(String),
    
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
}

pub type Result<T> = std::result::Result<T, QueryError>;