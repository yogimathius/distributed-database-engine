pub mod raft;
pub mod error;

pub use error::{ConsensusError, Result};
pub use raft::{RaftNode, RaftConfig};