// Non-Recursive Orchestration Runtime
// Bounded iterative state machine with explicit work queues

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Instruction type
#[derive(Debug, Clone)]
pub struct Instruction {
    pub protocol_version: String,
    pub instruction_id: String,
    pub symbol: String,
    pub target_runtime: String,
    pub verb: String,
    pub arguments: std::collections::HashMap<String, String>,
    pub timestamp: u64,
    pub instruction_hash: String,
    pub capability_id: String,
}

/// Execution stage in the LOC triad pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Receive,
    Translate,
    Verify,
    Dispatch,
    Execute,
    Encode,
    Seal,
    Complete,
}

impl Stage {
    pub fn next(self) -> Option<Stage> {
        match self {
            Stage::Receive => Some(Stage::Translate),
            Stage::Translate => Some(Stage::Verify),
            Stage::Verify => Some(Stage::Dispatch),
            Stage::Dispatch => Some(Stage::Execute),
            Stage::Execute => Some(Stage::Encode),
            Stage::Encode => Some(Stage::Seal),
            Stage::Seal => Some(Stage::Complete),
            Stage::Complete => None,
        }
    }
}

/// Work item in the orchestration queue
#[derive(Debug, Clone)]
pub struct WorkItem {
    pub stage: Stage,
    pub instruction: Instruction,
    pub receipt_id: String,
    pub sequence_number: u64,
}

/// Receipt entry
#[derive(Debug, Clone)]
pub struct Receipt {
    pub receipt_id: String,
    pub sequence_number: u64,
    pub agent_id: String,
    pub capability_id: String,
    pub action: String,
    pub input_hash: String,
    pub output_hash: String,
    pub status: String,
    pub timestamp: u64,
}

/// Non-recursive orchestrator
pub struct Orchestrator {
    work_queue: VecDeque<WorkItem>,
    receipt_chain: Vec<Receipt>,
    sequence_counter: u64,
    max_iterations: usize,
}

impl Orchestrator {
    pub fn new() -> Self {
        Orchestrator {
            work_queue: VecDeque::new(),
            receipt_chain: Vec::new(),
            sequence_counter: 0,
            max_iterations: 1000,
        }
    }

    /// Enqueue a new instruction for processing
    pub fn enqueue(&mut self, instruction: Instruction) {
        let work = WorkItem {
            stage: Stage::Receive,
            instruction,
            receipt_id: format!("rcpt_{:07}_orch", self.sequence_counter),
            sequence_number: self.sequence_counter,
        };
        self.work_queue.push_back(work);
        self.sequence_counter += 1;
    }

    /// Non-recursive execution loop
    /// Processes all work items iteratively, respecting stage ordering
    pub fn execute(&mut self) -> Result<Vec<Receipt>, String> {
        let mut iterations = 0;

        while !self.work_queue.is_empty() && iterations < self.max_iterations {
            iterations += 1;

            // Dequeue next work item
            let mut work = match self.work_queue.pop_front() {
                Some(w) => w,
                None => break,
            };

            // Execute current stage
            let result = self.execute_stage(&work)?;

            // If stage complete, advance to next
            if let Some(next_stage) = work.stage.next() {
                work.stage = next_stage;

                // Generate receipt for completed stage
                let receipt = Receipt {
                    receipt_id: work.receipt_id.clone(),
                    sequence_number: work.sequence_number,
                    agent_id: self.stage_agent(&work.stage),
                    capability_id: work.instruction.capability_id.clone(),
                    action: format!("{:?}", work.stage),
                    input_hash: work.instruction.instruction_hash.clone(),
                    output_hash: result,
                    status: "success".to_string(),
                    timestamp: self.now_unix(),
                };

                self.receipt_chain.push(receipt);

                // Re-enqueue for next stage
                self.work_queue.push_back(work);
            } else {
                // Pipeline complete
                let receipt = Receipt {
                    receipt_id: format!("{}_final", work.receipt_id),
                    sequence_number: work.sequence_number,
                    agent_id: "metatron".to_string(),
                    capability_id: work.instruction.capability_id.clone(),
                    action: "finalize".to_string(),
                    input_hash: work.instruction.instruction_hash.clone(),
                    output_hash: format!("final_{}", work.receipt_id),
                    status: "sealed".to_string(),
                    timestamp: self.now_unix(),
                };

                self.receipt_chain.push(receipt);
            }
        }

        if iterations >= self.max_iterations {
            return Err("Max iterations exceeded".to_string());
        }

        Ok(self.receipt_chain.clone())
    }

    /// Execute a single stage (stub)
    fn execute_stage(&self, work: &WorkItem) -> Result<String, String> {
        match work.stage {
            Stage::Receive => Ok(format!("received_{}", work.instruction.instruction_id)),
            Stage::Translate => Ok(format!("translated_{}", work.instruction.symbol)),
            Stage::Verify => Ok(format!("verified_{}", work.instruction.target_runtime)),
            Stage::Dispatch => Ok(format!("dispatched_{}", work.instruction.verb)),
            Stage::Execute => Ok(format!("executed_{}", work.instruction.instruction_id)),
            Stage::Encode => Ok(format!("encoded_{}", work.instruction.symbol)),
            Stage::Seal => Ok(format!("sealed_{}", work.receipt_id)),
            Stage::Complete => Ok("completed".to_string()),
        }
    }

    /// Get agent responsible for a stage
    fn stage_agent(&self, stage: &Stage) -> String {
        match stage {
            Stage::Receive => "loc".to_string(),
            Stage::Translate => "resonance".to_string(),
            Stage::Verify => "sentinel".to_string(),
            Stage::Dispatch => "loc".to_string(),
            Stage::Execute => "forge".to_string(),
            Stage::Encode => "resonance".to_string(),
            Stage::Seal => "metatron".to_string(),
            Stage::Complete => "metatron".to_string(),
        }
    }

    /// Get current Unix timestamp
    fn now_unix(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get receipt chain head
    pub fn receipt_chain_head(&self) -> Option<&Receipt> {
        self.receipt_chain.last()
    }

    /// Verify receipt chain integrity
    pub fn verify_chain(&self) -> bool {
        if self.receipt_chain.is_empty() {
            return true;
        }

        for i in 1..self.receipt_chain.len() {
            let prev = &self.receipt_chain[i - 1];
            let current = &self.receipt_chain[i];

            // Verify monotonic sequence
            if current.sequence_number <= prev.sequence_number {
                return false;
            }

            // Verify chain link (timestamps should be increasing)
            if current.timestamp < prev.timestamp {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_transitions() {
        assert_eq!(Stage::Receive.next(), Some(Stage::Translate));
        assert_eq!(Stage::Translate.next(), Some(Stage::Verify));
        assert_eq!(Stage::Complete.next(), None);
    }

    #[test]
    fn test_orchestrator_execution() {
        let mut orch = Orchestrator::new();

        let mut args = std::collections::HashMap::new();
        args.insert("fn".to_string(), "FreqAnchor1618".to_string());

        let instr = Instruction {
            protocol_version: "1.0.0".to_string(),
            instruction_id: "test_001".to_string(),
            symbol: "⚡".to_string(),
            target_runtime: "holyc".to_string(),
            verb: "Execute".to_string(),
            arguments: args,
            timestamp: 1719432000,
            instruction_hash: "abc123".to_string(),
            capability_id: "capa_001".to_string(),
        };

        orch.enqueue(instr);
        let receipts = orch.execute().expect("Orchestration failed");

        // Should have receipts for all stages
        assert!(!receipts.is_empty());
        assert!(orch.verify_chain());
    }
}
