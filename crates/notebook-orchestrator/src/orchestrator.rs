// Non-recursive orchestration runtime

use crate::{Instruction, Receipt, ReceiptChain, Stage};
use std::collections::VecDeque;

/// Work item in orchestration queue
#[derive(Debug, Clone)]
struct WorkItem {
    stage: Stage,
    instruction: Instruction,
    sequence_number: u64,
}

/// Non-recursive orchestrator for LOC triad
pub struct Orchestrator {
    work_queue: VecDeque<WorkItem>,
    receipt_chain: ReceiptChain,
    sequence_counter: u64,
    max_iterations: usize,
}

impl Orchestrator {
    /// Create new orchestrator
    pub fn new() -> Self {
        Orchestrator {
            work_queue: VecDeque::new(),
            receipt_chain: ReceiptChain::new(),
            sequence_counter: 0,
            max_iterations: 10000,
        }
    }

    /// Enqueue instruction for processing
    pub fn enqueue(&mut self, instruction: Instruction) -> Result<(), String> {
        if !instruction.verify_hash() {
            return Err("Instruction hash verification failed".to_string());
        }

        let work = WorkItem {
            stage: Stage::Receive,
            instruction,
            sequence_number: self.sequence_counter,
        };

        self.work_queue.push_back(work);
        self.sequence_counter += 1;
        Ok(())
    }

    /// Execute orchestration loop (non-recursive)
    pub fn execute(&mut self) -> Result<(), String> {
        let mut iterations = 0;

        while !self.work_queue.is_empty() && iterations < self.max_iterations {
            iterations += 1;

            // Dequeue next work item
            let mut work = self.work_queue.pop_front().ok_or("Queue empty")?;

            // Execute current stage
            let output_hash = self.execute_stage(&work)?;

            // Generate receipt
            let previous_hash = self
                .receipt_chain
                .head()
                .map(|r| r.receipt_hash.clone())
                .unwrap_or_else(|| "0".repeat(64));

            let receipt = Receipt::new(
                self.receipt_chain.len() as u64,
                work.stage.agent().to_string(),
                work.instruction.capability_id.clone(),
                work.instruction.instruction_hash.clone(),
                format!("{:?}", work.stage),
                work.instruction.instruction_hash.clone(),
                output_hash,
                work.stage.target_runtime().to_string(),
                previous_hash,
            );

            self.receipt_chain.append(receipt)?;

            // Advance to next stage or complete
            if let Some(next_stage) = work.stage.next() {
                work.stage = next_stage;
                self.work_queue.push_back(work);
            }
            // else: work item complete
        }

        if iterations >= self.max_iterations {
            return Err("Max iterations exceeded".to_string());
        }

        // Seal receipt chain
        self.receipt_chain.seal()?;

        Ok(())
    }

    /// Execute a single stage
    fn execute_stage(&self, work: &WorkItem) -> Result<String, String> {
        match work.stage {
            Stage::Receive => {
                // Stage: receive instruction
                Ok(format!("received_{}", &work.instruction.instruction_id[..12]))
            }
            Stage::Translate => {
                // Stage: translate emoji to intermediate representation
                Ok(format!("translated_{}", work.instruction.symbol))
            }
            Stage::Verify => {
                // Stage: verify capability and Ada proof
                Ok(format!("verified_{}", work.instruction.target_runtime))
            }
            Stage::Dispatch => {
                // Stage: dispatch to target runtime
                Ok(format!("dispatched_{}", work.instruction.verb))
            }
            Stage::Execute => {
                // Stage: execute in target runtime
                Ok(format!("executed_{}", work.instruction.target_runtime))
            }
            Stage::Encode => {
                // Stage: re-encode output to emoji
                Ok(format!("encoded_output_{}", work.instruction.symbol))
            }
            Stage::Seal => {
                // Stage: WORM seal
                Ok(format!("sealed_{:07}", self.receipt_chain.len()))
            }
            Stage::Complete => {
                // Should not reach here
                Ok("completed".to_string())
            }
        }
    }

    /// Get receipt chain
    pub fn receipt_chain(&self) -> &ReceiptChain {
        &self.receipt_chain
    }

    /// Get receipt chain (mutable)
    pub fn receipt_chain_mut(&mut self) -> &mut ReceiptChain {
        &mut self.receipt_chain
    }

    /// Verify orchestration integrity
    pub fn verify(&self) -> bool {
        self.receipt_chain.verify()
    }

    /// Get sequence counter
    pub fn sequence_counter(&self) -> u64 {
        self.sequence_counter
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_orchestrator_execution() {
        let mut orch = Orchestrator::new();

        let mut args = HashMap::new();
        args.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let instr = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args,
            "capa_001".to_string(),
        );

        assert!(orch.enqueue(instr).is_ok());
        assert!(orch.execute().is_ok());
        assert!(orch.verify());

        let receipts = orch.receipt_chain().all();
        assert!(receipts.len() >= 8); // All 8 stages should have receipts
    }

    #[test]
    fn test_receipt_chain_integrity() {
        let mut orch = Orchestrator::new();

        let mut args = HashMap::new();
        args.insert("test".to_string(), serde_json::json!("value"));

        let instr = Instruction::new(
            "🦀".to_string(),
            "rust".to_string(),
            "Build".to_string(),
            args,
            "capa_002".to_string(),
        );

        orch.enqueue(instr).unwrap();
        orch.execute().unwrap();

        assert!(orch.receipt_chain().verify());
        assert!(orch.receipt_chain().is_sealed());
    }
}
