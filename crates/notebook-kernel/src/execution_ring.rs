//! Execution Ring — Decentralized cell scheduling

use crate::cell_agent::CellAgent;
use anyhow::Result;
use std::collections::VecDeque;

/// Execution ring configuration
#[derive(Debug, Clone)]
pub struct RingConfig {
    pub max_concurrent_cells: usize,
    pub default_timeout_ms: u64,
    pub enable_proof_validation: bool,
    pub enable_m4_morph: bool,
}

/// Decentralized cell execution scheduler
pub struct ExecutionRing {
    config: RingConfig,
    work_queue: VecDeque<String>,
    active_cells: Vec<String>,
}

impl ExecutionRing {
    pub fn new(config: RingConfig) -> Self {
        Self {
            config,
            work_queue: VecDeque::new(),
            active_cells: Vec::new(),
        }
    }

    /// Submit a cell for execution
    pub fn submit(&mut self, _agent: CellAgent) -> Result<()> {
        // In production: queue cell, schedule on next available slot
        Ok(())
    }

    /// Shutdown execution ring
    pub fn shutdown(&mut self) -> Result<()> {
        self.work_queue.clear();
        self.active_cells.clear();
        Ok(())
    }

    pub fn active_cell_count(&self) -> usize {
        self.active_cells.len()
    }

    pub fn queue_size(&self) -> usize {
        self.work_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_ring_creation() {
        let config = RingConfig {
            max_concurrent_cells: 8,
            default_timeout_ms: 30000,
            enable_proof_validation: true,
            enable_m4_morph: true,
        };

        let ring = ExecutionRing::new(config);
        assert_eq!(ring.active_cell_count(), 0);
        assert_eq!(ring.queue_size(), 0);
    }

    #[test]
    fn test_execution_ring_shutdown() {
        let config = RingConfig {
            max_concurrent_cells: 8,
            default_timeout_ms: 30000,
            enable_proof_validation: true,
            enable_m4_morph: true,
        };

        let mut ring = ExecutionRing::new(config);
        ring.shutdown().unwrap();
        assert_eq!(ring.queue_size(), 0);
    }
}
