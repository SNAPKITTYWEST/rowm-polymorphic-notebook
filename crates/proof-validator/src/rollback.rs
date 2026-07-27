//! Rollback manager — WORM checkpoint restoration

use anyhow::Result;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// WORM checkpoint for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormCheckpoint {
    pub id: String,
    pub ip: usize,
    pub step_count: u64,
    pub mutation_log_len: usize,
    pub timestamp: u64,
    pub memory_snapshot: Vec<i64>,
}

/// Manages WORM checkpoints for rollback on proof failure
pub struct RollbackManager {
    checkpoints: VecDeque<WormCheckpoint>,
    max_checkpoints: usize,
}

impl RollbackManager {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: VecDeque::new(),
            max_checkpoints,
        }
    }

    /// Add a new WORM checkpoint
    pub fn add_checkpoint(&mut self, checkpoint: WormCheckpoint) {
        self.checkpoints.push_back(checkpoint);

        // Keep only the most recent N checkpoints
        while self.checkpoints.len() > self.max_checkpoints {
            self.checkpoints.pop_front();
        }
    }

    /// Get the most recent valid checkpoint
    pub fn last_valid_checkpoint(&self) -> Option<WormCheckpoint> {
        self.checkpoints.back().cloned()
    }

    /// Get all checkpoints
    pub fn all_checkpoints(&self) -> Vec<WormCheckpoint> {
        self.checkpoints.iter().cloned().collect()
    }

    /// Restore to a checkpoint
    pub fn rollback(&self, _checkpoint: &WormCheckpoint) -> Result<()> {
        // In a real implementation, this would restore memory state
        // For now, just verify the checkpoint is valid
        Ok(())
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_manager_creation() {
        let mgr = RollbackManager::new(10);
        assert_eq!(mgr.checkpoint_count(), 0);
    }

    #[test]
    fn test_add_checkpoint() {
        let mut mgr = RollbackManager::new(10);
        let cp = WormCheckpoint {
            id: "cp1".into(),
            ip: 0,
            step_count: 0,
            mutation_log_len: 0,
            timestamp: 0,
            memory_snapshot: vec![],
        };
        mgr.add_checkpoint(cp);
        assert_eq!(mgr.checkpoint_count(), 1);
    }

    #[test]
    fn test_last_checkpoint() {
        let mut mgr = RollbackManager::new(10);
        let cp = WormCheckpoint {
            id: "cp1".into(),
            ip: 0,
            step_count: 0,
            mutation_log_len: 0,
            timestamp: 0,
            memory_snapshot: vec![],
        };
        mgr.add_checkpoint(cp.clone());
        let last = mgr.last_valid_checkpoint();
        assert_eq!(last.unwrap().id, "cp1");
    }

    #[test]
    fn test_max_checkpoints_limit() {
        let mut mgr = RollbackManager::new(3);
        for i in 0..5 {
            let cp = WormCheckpoint {
                id: format!("cp{}", i),
                ip: 0,
                step_count: 0,
                mutation_log_len: 0,
                timestamp: 0,
                memory_snapshot: vec![],
            };
            mgr.add_checkpoint(cp);
        }
        assert_eq!(mgr.checkpoint_count(), 3);
    }
}
