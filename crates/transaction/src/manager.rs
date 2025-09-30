use crate::{
    error::{Result, TransactionError},
    mvcc::{Transaction, TransactionId, TransactionStatus, IsolationLevel},
};
use dashmap::DashMap;
use std::sync::Arc;

/// Transaction manager with MVCC support
pub struct TransactionManager {
    active_transactions: Arc<DashMap<TransactionId, Transaction>>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active_transactions: Arc::new(DashMap::new()),
        }
    }
    
    pub async fn begin(&self, isolation_level: IsolationLevel) -> Result<TransactionId> {
        let txn = Transaction::new(isolation_level);
        let txn_id = txn.id;
        
        self.active_transactions.insert(txn_id, txn);
        
        Ok(txn_id)
    }
    
    pub async fn commit(&self, txn_id: TransactionId) -> Result<()> {
        if let Some(mut txn) = self.active_transactions.get_mut(&txn_id) {
            if !txn.is_active() {
                return Err(TransactionError::NotFound(txn_id.0.to_string()));
            }
            
            txn.status = TransactionStatus::Committed;
            Ok(())
        } else {
            Err(TransactionError::NotFound(txn_id.0.to_string()))
        }
    }
    
    pub async fn abort(&self, txn_id: TransactionId) -> Result<()> {
        if let Some(mut txn) = self.active_transactions.get_mut(&txn_id) {
            txn.status = TransactionStatus::Aborted;
            Ok(())
        } else {
            Err(TransactionError::NotFound(txn_id.0.to_string()))
        }
    }
    
    pub fn get_transaction(&self, txn_id: &TransactionId) -> Option<Transaction> {
        self.active_transactions.get(txn_id).map(|t| t.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let manager = TransactionManager::new();
        
        // Begin transaction
        let txn_id = manager.begin(IsolationLevel::ReadCommitted).await.unwrap();
        
        // Verify transaction exists
        let txn = manager.get_transaction(&txn_id).unwrap();
        assert!(txn.is_active());
        
        // Commit transaction
        manager.commit(txn_id).await.unwrap();
        
        // Verify transaction is committed
        let txn = manager.get_transaction(&txn_id).unwrap();
        assert!(!txn.is_active());
    }
    
    #[tokio::test]
    async fn test_transaction_abort() {
        let manager = TransactionManager::new();
        
        let txn_id = manager.begin(IsolationLevel::RepeatableRead).await.unwrap();
        
        // Abort transaction
        manager.abort(txn_id).await.unwrap();
        
        // Verify transaction is aborted
        let txn = manager.get_transaction(&txn_id).unwrap();
        assert!(!txn.is_active());
        assert!(matches!(txn.status, TransactionStatus::Aborted));
    }
}