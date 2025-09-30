use crate::error::{Result, ConsensusError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    pub node_id: NodeId,
    pub peers: Vec<NodeId>,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RaftState {
    Follower,
    Candidate,  
    Leader,
}

/// Simplified Raft node implementation
pub struct RaftNode {
    config: RaftConfig,
    state: RaftState,
    current_term: u64,
    voted_for: Option<NodeId>,
    log: Vec<LogEntry>,
    commit_index: u64,
    last_applied: u64,
    
    // Leader state
    next_index: HashMap<NodeId, u64>,
    match_index: HashMap<NodeId, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

impl RaftNode {
    pub fn new(config: RaftConfig) -> Self {
        Self {
            config,
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }
    
    pub fn is_leader(&self) -> bool {
        matches!(self.state, RaftState::Leader)
    }
    
    pub fn current_term(&self) -> u64 {
        self.current_term
    }
    
    pub fn state(&self) -> &RaftState {
        &self.state
    }
    
    pub async fn propose(&mut self, data: Vec<u8>) -> Result<u64> {
        if !self.is_leader() {
            return Err(ConsensusError::NotLeader);
        }
        
        let index = self.log.len() as u64;
        let entry = LogEntry {
            term: self.current_term,
            index,
            data,
        };
        
        self.log.push(entry);
        
        // In a real implementation, this would replicate to followers
        Ok(index)
    }
    
    pub fn get_log_entry(&self, index: u64) -> Option<&LogEntry> {
        self.log.get(index as usize)
    }
    
    pub fn log_len(&self) -> u64 {
        self.log.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_raft_node_creation() {
        let config = RaftConfig {
            node_id: NodeId::new(),
            peers: vec![NodeId::new(), NodeId::new()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };
        
        let node = RaftNode::new(config);
        assert_eq!(node.state(), &RaftState::Follower);
        assert_eq!(node.current_term(), 0);
        assert!(!node.is_leader());
    }
    
    #[tokio::test]
    async fn test_raft_proposal_not_leader() {
        let config = RaftConfig {
            node_id: NodeId::new(),
            peers: vec![],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };
        
        let mut node = RaftNode::new(config);
        let result = node.propose(b"test data".to_vec()).await;
        assert!(matches!(result, Err(ConsensusError::NotLeader)));
    }
}