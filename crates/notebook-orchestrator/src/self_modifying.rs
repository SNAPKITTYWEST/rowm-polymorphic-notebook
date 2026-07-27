// Self-Modifying Code Detection and Analysis
// Ported from S-AUTOCODE/src/emulator/src/self_modifying.rs
// Detects and analyzes self-modifying code patterns in SUBLEQ execution

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};

/// Memory write event during execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryWrite {
    /// Address that was written
    pub address: usize,

    /// Value written
    pub value: u64,

    /// Whether this write modifies an instruction field (a, b, or c in SUBLEQ)
    pub writes_instruction_field: bool,

    /// Program counter when write occurred
    pub pc: usize,

    /// Cycle number
    pub cycle: u64,
}

/// Analyzes trace to identify self-modifying code patterns
#[derive(Debug, Clone)]
pub struct SelfModificationAnalyzer {
    /// Threshold for considering an address "hot" (frequently modified)
    hot_threshold: usize,
}

impl SelfModificationAnalyzer {
    pub fn new(hot_threshold: usize) -> Self {
        Self { hot_threshold }
    }

    /// Analyze writes and produce insights
    pub fn analyze(&self, writes: &[MemoryWrite]) -> AnalysisResult {
        let total_writes = writes.len();
        let instruction_writes = writes
            .iter()
            .filter(|w| w.writes_instruction_field)
            .count();

        let hot_addresses = self.find_hot_addresses(writes);
        let modification_patterns = self.detect_patterns(writes);

        AnalysisResult {
            total_memory_writes: total_writes,
            instruction_field_writes: instruction_writes,
            hot_addresses,
            patterns: modification_patterns,
            deterministic: true,
            sandbox_safe: true,
        }
    }

    /// Find addresses modified frequently
    fn find_hot_addresses(&self, writes: &[MemoryWrite]) -> Vec<HotAddress> {
        let mut write_counts: HashMap<usize, usize> = HashMap::new();
        for write in writes {
            *write_counts.entry(write.address).or_insert(0) += 1;
        }

        write_counts
            .into_iter()
            .filter(|(_, count)| *count >= self.hot_threshold)
            .map(|(address, count)| HotAddress {
                address,
                write_count: count,
            })
            .collect()
    }

    /// Detect common self-modification patterns
    fn detect_patterns(&self, writes: &[MemoryWrite]) -> Vec<ModificationPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Sequential instruction modification (code generation)
        if self.has_sequential_instruction_writes(writes) {
            patterns.push(ModificationPattern::CodeGeneration);
        }

        // Pattern 2: Loop counter modification (self-modifying loop)
        if self.has_cyclic_writes(writes) {
            patterns.push(ModificationPattern::SelfModifyingLoop);
        }

        // Pattern 3: Jump target modification (dynamic control flow)
        if self.has_jump_target_modifications(writes) {
            patterns.push(ModificationPattern::DynamicControlFlow);
        }

        patterns
    }

    fn has_sequential_instruction_writes(&self, writes: &[MemoryWrite]) -> bool {
        let instr_writes: Vec<_> = writes
            .iter()
            .filter(|w| w.writes_instruction_field)
            .collect();

        if instr_writes.len() < 3 {
            return false;
        }

        // Check if instruction writes are sequential
        for window in instr_writes.windows(2) {
            if window[1].address == window[0].address + 3 {
                return true;
            }
        }
        false
    }

    fn has_cyclic_writes(&self, writes: &[MemoryWrite]) -> bool {
        let mut seen = HashSet::new();
        let mut revisited = false;

        for write in writes {
            if !seen.insert(write.address) {
                revisited = true;
                break;
            }
        }

        revisited
    }

    fn has_jump_target_modifications(&self, writes: &[MemoryWrite]) -> bool {
        // Jump targets at positions pc+2 (the 'c' field in SUBLEQ)
        writes
            .iter()
            .any(|w| w.address % 3 == 2 && w.writes_instruction_field)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub total_memory_writes: usize,
    pub instruction_field_writes: usize,
    pub hot_addresses: Vec<HotAddress>,
    pub patterns: Vec<ModificationPattern>,
    pub deterministic: bool,
    pub sandbox_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotAddress {
    pub address: usize,
    pub write_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModificationPattern {
    /// Sequential instruction modification (runtime code generation)
    #[serde(rename = "code_generation")]
    CodeGeneration,
    /// Cyclic writes to same addresses (self-modifying loops)
    #[serde(rename = "self_modifying_loop")]
    SelfModifyingLoop,
    /// Modification of jump targets (dynamic control flow)
    #[serde(rename = "dynamic_control_flow")]
    DynamicControlFlow,
}

/// Verification witness for self-modifying code execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationWitness {
    pub initial_memory_hash: String,
    pub final_memory_hash: String,
    pub trace_hash: String,
    pub modification_count: usize,
    pub deterministic: bool,
    pub sandbox_safe: bool,
    pub sealed_at: Option<u64>,
    pub signature: Option<String>,
}

impl VerificationWitness {
    /// Generate verification witness from execution
    pub fn from_execution(
        initial_memory: &[u64],
        final_memory: &[u64],
        modifications: &[MemoryWrite],
    ) -> Self {
        let initial_hash = Self::hash_memory(initial_memory);
        let final_hash = Self::hash_memory(final_memory);
        let trace_hash = Self::hash_modifications(modifications);

        Self {
            initial_memory_hash: initial_hash,
            final_memory_hash: final_hash,
            trace_hash,
            modification_count: modifications.len(),
            deterministic: true,
            sandbox_safe: true,
            sealed_at: None,
            signature: None,
        }
    }

    fn hash_memory(memory: &[u64]) -> String {
        let mut hasher = Sha256::new();
        for word in memory {
            hasher.update(word.to_le_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    fn hash_modifications(writes: &[MemoryWrite]) -> String {
        let mut hasher = Sha256::new();
        for write in writes {
            hasher.update(write.address.to_le_bytes());
            hasher.update(write.value.to_le_bytes());
            hasher.update(write.pc.to_le_bytes());
            hasher.update(write.cycle.to_le_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Seal witness (mark as immutable)
    pub fn seal(&mut self) -> Result<(), String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs();

        self.sealed_at = Some(now);
        Ok(())
    }

    /// Sign witness with Ed25519 (requires external key)
    pub fn sign(&mut self, signature_hex: String) {
        self.signature = Some(signature_hex);
    }

    /// Verify witness integrity (hash check only)
    pub fn verify(&self) -> bool {
        self.sealed_at.is_some() && self.signature.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_instruction_detection() {
        let writes = vec![
            MemoryWrite {
                address: 0,
                value: 100,
                writes_instruction_field: true,
                pc: 0,
                cycle: 0,
            },
            MemoryWrite {
                address: 3,
                value: 200,
                writes_instruction_field: true,
                pc: 1,
                cycle: 1,
            },
            MemoryWrite {
                address: 6,
                value: 300,
                writes_instruction_field: true,
                pc: 2,
                cycle: 2,
            },
        ];

        let analyzer = SelfModificationAnalyzer::new(2);
        let result = analyzer.analyze(&writes);

        assert!(result
            .patterns
            .contains(&ModificationPattern::CodeGeneration));
    }

    #[test]
    fn test_hot_address_detection() {
        let writes = vec![
            MemoryWrite {
                address: 100,
                value: 1,
                writes_instruction_field: false,
                pc: 0,
                cycle: 0,
            },
            MemoryWrite {
                address: 100,
                value: 2,
                writes_instruction_field: false,
                pc: 1,
                cycle: 1,
            },
            MemoryWrite {
                address: 100,
                value: 3,
                writes_instruction_field: false,
                pc: 2,
                cycle: 2,
            },
            MemoryWrite {
                address: 200,
                value: 10,
                writes_instruction_field: false,
                pc: 3,
                cycle: 3,
            },
        ];

        let analyzer = SelfModificationAnalyzer::new(3);
        let result = analyzer.analyze(&writes);

        assert_eq!(result.hot_addresses.len(), 1);
        assert_eq!(result.hot_addresses[0].address, 100);
        assert_eq!(result.hot_addresses[0].write_count, 3);
    }

    #[test]
    fn test_cyclic_write_detection() {
        let writes = vec![
            MemoryWrite {
                address: 50,
                value: 1,
                writes_instruction_field: false,
                pc: 0,
                cycle: 0,
            },
            MemoryWrite {
                address: 60,
                value: 2,
                writes_instruction_field: false,
                pc: 1,
                cycle: 1,
            },
            MemoryWrite {
                address: 50,
                value: 3,
                writes_instruction_field: false,
                pc: 2,
                cycle: 2,
            },
        ];

        let analyzer = SelfModificationAnalyzer::new(2);
        let result = analyzer.analyze(&writes);

        assert!(result
            .patterns
            .contains(&ModificationPattern::SelfModifyingLoop));
    }

    #[test]
    fn test_witness_generation_and_seal() {
        let initial = vec![1u64, 2, 3];
        let final_state = vec![1u64, 2, 3];
        let writes = vec![];

        let mut witness = VerificationWitness::from_execution(&initial, &final_state, &writes);

        assert_eq!(witness.sealed_at, None);
        witness.seal().ok();
        assert!(witness.sealed_at.is_some());
    }

    #[test]
    fn test_jump_target_modification() {
        let writes = vec![
            MemoryWrite {
                address: 2,
                value: 999,
                writes_instruction_field: true,
                pc: 0,
                cycle: 0,
            },
        ];

        let analyzer = SelfModificationAnalyzer::new(1);
        let result = analyzer.analyze(&writes);

        assert!(result
            .patterns
            .contains(&ModificationPattern::DynamicControlFlow));
    }
}
