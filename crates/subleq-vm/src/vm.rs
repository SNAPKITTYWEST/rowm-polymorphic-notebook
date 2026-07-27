//! SUBLEQ Virtual Machine — Core execution engine

use crate::memory::{Memory, Address};
use crate::checkpoint::{Checkpoint, CheckpointManager};
use crate::telemetry::{TelemetryEmitter, MutationDelta};
use anyhow::{Result, Context};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{trace, debug, warn};

/// SUBLEQ Instruction: M[b] ← M[b] - M[a]; if M[b] ≤ 0 then IP ← c
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub a: Address,
    pub b: Address,
    pub c: Address,
}

impl Instruction {
    pub const SIZE: usize = 3;

    pub fn from_memory(mem: &Memory, ip: Address) -> Option<Self> {
        let a = mem.read(ip)? as Address;
        let b = mem.read(ip + 1)? as Address;
        let c = mem.read(ip + 2)? as Address;
        Some(Self { a, b, c })
    }
}

/// Execution modes for the VM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    /// Standard execution with mutation logging
    Standard,
    /// Proof validation mode
    Verified,
    /// Dry-run for invariant extraction
    Analysis,
}

/// High-performance SUBLEQ VM with self-modification tracking
pub struct SubleqVM {
    memory: Arc<RwLock<Memory>>,
    ip: Address,
    mode: ExecMode,
    mutation_log: Vec<MutationDelta>,
    checkpoint_mgr: CheckpointManager,
    telemetry: Option<Arc<TelemetryEmitter>>,
    step_count: u64,
    max_steps: Option<u64>,
}

impl SubleqVM {
    pub fn new(memory: Memory, mode: ExecMode) -> Self {
        let mem = Arc::new(RwLock::new(memory));
        Self {
            memory: mem.clone(),
            ip: 0,
            mode,
            mutation_log: Vec::new(),
            checkpoint_mgr: CheckpointManager::new(mem),
            telemetry: None,
            step_count: 0,
            max_steps: None,
        }
    }

    pub fn with_telemetry(mut self, emitter: Arc<TelemetryEmitter>) -> Self {
        self.telemetry = Some(emitter);
        self
    }

    pub fn with_max_steps(mut self, max: u64) -> Self {
        self.max_steps = Some(max);
        self
    }

    /// Execute a single SUBLEQ instruction
    pub fn step(&mut self) -> Result<Option<bool>> {
        if let Some(max) = self.max_steps {
            if self.step_count >= max {
                return Ok(None);
            }
        }

        let mem = self.memory.read();
        let instr = Instruction::from_memory(&mem, self.ip)
            .context("Instruction fetch failed: IP out of bounds")?;
        drop(mem);

        trace!(ip = self.ip, ?instr, "Executing SUBLEQ");

        // Fetch operands
        let (val_a, val_b) = {
            let mem = self.memory.read();
            let val_a = mem.read(instr.a).unwrap_or(0);
            let val_b = mem.read(instr.b).unwrap_or(0);
            (val_a, val_b)
        };

        // Perform subtraction
        let res = val_b.wrapping_sub(val_a);

        // Write result + log mutation
        {
            let mut mem = self.memory.write();
            mem.write(instr.b, res)?;
            self.mutation_log.push(MutationDelta {
                address: instr.b,
                old_value: val_b,
                new_value: res,
                ip: self.ip,
                step: self.step_count,
            });
        }

        // Emit telemetry
        if let Some(ref tel) = self.telemetry {
            tel.emit(MutationDelta {
                address: instr.b,
                old_value: val_b,
                new_value: res,
                ip: self.ip,
                step: self.step_count,
            });
        }

        // Branch logic
        let branched = res <= 0;
        if branched {
            self.ip = instr.c;
        } else {
            self.ip += Instruction::SIZE;
        }

        self.step_count += 1;
        Ok(Some(branched))
    }

    /// Run until halt or max steps
    pub fn run(&mut self) -> Result<()> {
        while self.step()?.is_some() {}
        Ok(())
    }

    /// Create WORM checkpoint
    pub fn checkpoint(&mut self) -> Checkpoint {
        self.checkpoint_mgr.create(self.ip, self.step_count, self.mutation_log.len())
    }

    /// Rollback to checkpoint
    pub fn rollback(&mut self, checkpoint: &Checkpoint) -> Result<()> {
        self.checkpoint_mgr.restore(checkpoint, &mut self.memory.write())?;
        self.ip = checkpoint.ip;
        self.step_count = checkpoint.step_count;
        self.mutation_log.truncate(checkpoint.mutation_log_len);
        warn!(?checkpoint, "Rolled back");
        Ok(())
    }

    pub fn mutation_log(&self) -> &[MutationDelta] {
        &self.mutation_log
    }

    pub fn ip(&self) -> Address {
        self.ip
    }

    pub fn steps(&self) -> u64 {
        self.step_count
    }

    pub fn memory(&self) -> Arc<RwLock<Memory>> {
        self.memory.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_subleq() {
        let mut mem = Memory::new(1024);
        mem.write(0, 1).unwrap();
        mem.write(1, 5).unwrap();
        mem.write(2, 6).unwrap();
        mem.write(3, 0).unwrap();

        let mut vm = SubleqVM::new(mem, ExecMode::Standard);
        vm.run().unwrap();

        let final_mem = vm.memory.read();
        assert_eq!(final_mem.read(2), Some(1));
    }
}
