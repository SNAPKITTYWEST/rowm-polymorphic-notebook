// Instruction type and validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Canonical instruction for LOC dispatch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instruction {
    pub protocol_version: String,
    pub instruction_id: String,
    pub symbol: String,
    pub target_runtime: String,
    pub verb: String,
    pub arguments: HashMap<String, serde_json::Value>,
    pub timestamp: u64,
    pub instruction_hash: String,
    pub capability_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cell_id: Option<String>,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(
        symbol: String,
        target_runtime: String,
        verb: String,
        arguments: HashMap<String, serde_json::Value>,
        capability_id: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let instruction_id = Uuid::new_v4().to_string();
        let instruction_hash = Self::compute_hash(&symbol, &target_runtime, &verb, &arguments);

        Instruction {
            protocol_version: "1.0.0".to_string(),
            instruction_id,
            symbol,
            target_runtime,
            verb,
            arguments,
            timestamp,
            instruction_hash,
            capability_id,
            source_cell_id: None,
        }
    }

    /// Compute canonical hash of instruction
    fn compute_hash(symbol: &str, runtime: &str, verb: &str, args: &HashMap<String, serde_json::Value>) -> String {
        use sha2::{Sha256, Digest};

        let canonical = serde_json::json!({
            "symbol": symbol,
            "runtime": runtime,
            "verb": verb,
            "arguments": args
        });

        let mut hasher = Sha256::new();
        hasher.update(canonical.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify instruction hash
    pub fn verify_hash(&self) -> bool {
        let computed = Self::compute_hash(&self.symbol, &self.target_runtime, &self.verb, &self.arguments);
        computed == self.instruction_hash
    }

    /// Add source cell ID
    pub fn with_source_cell(mut self, cell_id: String) -> Self {
        self.source_cell_id = Some(cell_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_creation() {
        let mut args = HashMap::new();
        args.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let instr = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args,
            "capa_001".to_string(),
        );

        assert_eq!(instr.protocol_version, "1.0.0");
        assert_eq!(instr.symbol, "⚡");
        assert_eq!(instr.target_runtime, "holyc");
        assert!(instr.verify_hash());
    }

    #[test]
    fn test_instruction_hash_determinism() {
        let mut args1 = HashMap::new();
        args1.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let mut args2 = HashMap::new();
        args2.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let instr1 = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args1,
            "capa_001".to_string(),
        );

        let instr2 = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args2,
            "capa_001".to_string(),
        );

        // Same semantic instruction should have same hash
        // (Note: different instruction_ids due to UUID, but hash should match)
        assert_eq!(instr1.instruction_hash, instr2.instruction_hash);
    }
}

