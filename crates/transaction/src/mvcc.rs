use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub isolation_level: IsolationLevel,
    pub start_timestamp: u64,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Active,
    Committed,
    Aborted,
}

impl Transaction {
    pub fn new(isolation_level: IsolationLevel) -> Self {
        Self {
            id: TransactionId::new(),
            isolation_level,
            start_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            status: TransactionStatus::Active,
        }
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, TransactionStatus::Active)
    }
}