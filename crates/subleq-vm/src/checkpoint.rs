//! WORM Checkpointing — Append-only rollback snapshots

use crate::memory::{Memory, Address};
use anyhow::{Result, Context};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: String,
    pub ip: Address,
    pub step_count: u64,
    pub mutation_log_len: usize,
    pub memory_snapshot: Vec<i64>,
    pub timestamp: u64,
}

impl Checkpoint {
    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct CheckpointManager {
    memory: Arc<RwLock<Memory>>,
    checkpoints: Vec<Checkpoint>,
}

impl CheckpointManager {
    pub fn new(memory: Arc<RwLock<Memory>>) -> Self {
        Self {
            memory,
            checkpoints: Vec::new(),
        }
    }

    pub fn create(&mut self, ip: Address, step_count: u64, mutation_log_len: usize) -> Checkpoint {
        use std::time::{SystemTime, UNIX_EPOCH};

        let mem = self.memory.read();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let checkpoint = Checkpoint {
            id: Uuid::new_v4().to_string(),
            ip,
            step_count,
            mutation_log_len,
            memory_snapshot: mem.as_slice().to_vec(),
            timestamp,
        };

        self.checkpoints.push(checkpoint.clone());
        checkpoint
    }

    pub fn restore(&self, checkpoint: &Checkpoint, memory: &mut Memory) -> Result<()> {
        if checkpoint.memory_snapshot.len() != memory.size() {
            return Err(anyhow::anyhow!("Checkpoint memory size mismatch"));
        }

        for (addr, &word) in checkpoint.memory_snapshot.iter().enumerate() {
            memory.write(addr, word)?;
        }

        Ok(())
    }

    pub fn list_checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_create_restore() {
        let mut mem = Memory::new(1024);
        mem.write(0, 42).unwrap();
        mem.write(1, 100).unwrap();

        let mem = Arc::new(RwLock::new(mem));
        let mut mgr = CheckpointManager::new(mem.clone());

        let cp = mgr.create(0, 0, 0);
        assert_eq!(cp.step_count, 0);

        {
            let mut m = mem.write();
            m.write(0, 999).unwrap();
        }

        {
            let mut m = mem.write();
            mgr.restore(&cp, &mut m).unwrap();
        }

        let m = mem.read();
        assert_eq!(m.read(0), Some(42));
    }
}
