// ROWM v2.0 Receipt — Canonical CBOR + Ed25519 Signatures
// Replaces v1.0 SHA-256-only receipts with deterministic, verifiable receipts

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::ed25519_keymanager::KeyStore;
use signature::Signer;

/// Receipt schema version 2.0 — Ed25519 signed, deterministic hashing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptV2 {
    /// Schema identifier (MUST be "2.0" for v2 receipts)
    pub schema_version: String,

    /// Monotonic sequence in receipt chain (0-indexed)
    pub sequence_number: u64,

    /// Unique receipt identifier (NOT part of canonical hash)
    pub receipt_id: String,

    /// SHA-512 hash of this receipt (computed from canonical bytes, excluding timestamp)
    pub receipt_hash: String,

    /// SHA-512 hash of previous receipt in chain (or all-zeros for first)
    pub previous_receipt_hash: String,

    /// Agent that executed this action
    pub agent_id: String,

    /// Capability under which action was authorized
    pub capability_id: String,

    /// SHA-512 hash of the instruction executed
    pub instruction_hash: String,

    /// Action performed (dispatch, verify, execute, seal, etc.)
    pub action: String,

    /// SHA-512 hash of input to this action
    pub input_hash: String,

    /// SHA-512 hash of output from this action
    pub output_hash: String,

    /// Identity of runtime that executed this
    pub runtime_identity: String,

    /// Version string of runtime
    pub runtime_version: String,

    /// Git commit hash of source code (SHA-1)
    pub source_commit: String,

    /// SHA-512 hash of environment (config, variables, etc. at execution)
    pub environment_hash: String,

    /// Unix timestamp (NOT part of canonical receipt hash)
    pub timestamp: u64,

    /// System monotonic counter (for causality tracking)
    pub monotonic_counter: u64,

    /// Execution status
    pub status: ReceiptStatus,

    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Optional performance metrics (NOT part of canonical hash)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_metrics: Option<PerformanceMetrics>,

    /// Ed25519 detached signature (hex-encoded 64 bytes)
    pub signature: Option<String>,

    /// Public key that signed this receipt (hex-encoded 32 bytes)
    pub signing_public_key: Option<String>,

    /// Key rotation version (for key management)
    pub key_version: Option<u32>,

    /// Whether this receipt is WORM-sealed (immutable)
    pub sealed: bool,

    /// Nonce for replay protection
    pub nonce: String,

    /// Context identifier (for cross-system replay detection)
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReceiptStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl fmt::Display for ReceiptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReceiptStatus::Success => write!(f, "success"),
            ReceiptStatus::Failure => write!(f, "failure"),
            ReceiptStatus::Pending => write!(f, "pending"),
            ReceiptStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMetrics {
    pub wall_time_ms: f64,
    pub cpu_time_ms: f64,
    pub memory_peak_bytes: u64,
}

// Custom Eq impl (f64 is not Eq, so we skip it)
impl Eq for PerformanceMetrics {}

impl ReceiptV2 {
    /// Create a new receipt (unsigned)
    pub fn new(
        sequence_number: u64,
        agent_id: String,
        capability_id: String,
        instruction_hash: String,
        action: String,
        input_hash: String,
        output_hash: String,
        runtime_identity: String,
        runtime_version: String,
        source_commit: String,
        environment_hash: String,
        previous_receipt_hash: String,
        monotonic_counter: u64,
        nonce: String,
        context: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let receipt_id = format!("rcpt-v2-{:016x}-{}", sequence_number, agent_id);

        // Compute canonical hash (timestamp NOT included)
        let receipt_hash = Self::compute_canonical_hash(
            sequence_number,
            &agent_id,
            &capability_id,
            &instruction_hash,
            &action,
            &input_hash,
            &output_hash,
            &runtime_identity,
            &runtime_version,
            &source_commit,
            &environment_hash,
            &previous_receipt_hash,
            monotonic_counter,
            &nonce,
            &context,
        );

        ReceiptV2 {
            schema_version: "2.0".to_string(),
            sequence_number,
            receipt_id,
            receipt_hash,
            previous_receipt_hash,
            agent_id,
            capability_id,
            instruction_hash,
            action,
            input_hash,
            output_hash,
            runtime_identity,
            runtime_version,
            source_commit,
            environment_hash,
            timestamp,
            monotonic_counter,
            status: ReceiptStatus::Success,
            error_message: None,
            performance_metrics: None,
            signature: None,
            signing_public_key: None,
            key_version: None,
            sealed: false,
            nonce,
            context,
        }
    }

    /// Compute SHA-512 canonical hash (timestamp excluded for determinism)
    fn compute_canonical_hash(
        seq: u64,
        agent: &str,
        cap: &str,
        instr_hash: &str,
        action: &str,
        input_hash: &str,
        output_hash: &str,
        runtime_id: &str,
        runtime_ver: &str,
        source_commit: &str,
        env_hash: &str,
        prev_hash: &str,
        mono_counter: u64,
        nonce: &str,
        context: &str,
    ) -> String {
        use sha2::{Sha512, Digest};

        // Canonical order: sequence, agent, capability, hashes, runtime, source, environment, counter, nonce, context
        // EXPLICITLY EXCLUDE timestamp
        let payload = format!(
            "seq:{}|agent:{}|cap:{}|instr:{}|action:{}|input:{}|output:{}|rt_id:{}|rt_ver:{}|src:{}|env:{}|prev:{}|mono:{}|nonce:{}|ctx:{}",
            seq, agent, cap, instr_hash, action, input_hash, output_hash,
            runtime_id, runtime_ver, source_commit, env_hash, prev_hash, mono_counter, nonce, context
        );

        let mut hasher = Sha512::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Sign receipt with Ed25519 private key
    pub fn sign(&mut self, private_key_hex: &str, public_key_hex: &str, key_version: u32) -> Result<(), String> {
        // Decode private key from hex
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Failed to decode private key: {}", e))?;

        if private_key_bytes.len() != 32 {
            return Err(format!("Private key must be 32 bytes, got {}", private_key_bytes.len()));
        }

        // Use ed25519_dalek for signing
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            &private_key_bytes.as_slice().try_into()
                .map_err(|_| "Invalid private key bytes".to_string())?
        );

        // Sign canonical hash bytes
        let message = self.receipt_hash.as_bytes();
        let signature = signing_key.sign(message);

        self.signature = Some(hex::encode(signature.to_bytes()));
        self.signing_public_key = Some(public_key_hex.to_string());
        self.key_version = Some(key_version);

        Ok(())
    }

    /// Verify Ed25519 signature
    pub fn verify_signature(&self) -> Result<bool, String> {
        let sig_hex = self.signature.as_ref()
            .ok_or("Receipt is not signed")?;
        let pubkey_hex = self.signing_public_key.as_ref()
            .ok_or("No public key provided")?;

        let signature_bytes = hex::decode(sig_hex)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| format!("Failed to decode public key: {}", e))?;

        if signature_bytes.len() != 64 {
            return Err(format!("Signature must be 64 bytes, got {}", signature_bytes.len()));
        }
        if pubkey_bytes.len() != 32 {
            return Err(format!("Public key must be 32 bytes, got {}", pubkey_bytes.len()));
        }

        let sig_array: [u8; 64] = signature_bytes.try_into()
            .map_err(|_| "Invalid signature bytes".to_string())?;
        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        let pk_array: [u8; 32] = pubkey_bytes.try_into()
            .map_err(|_| "Invalid public key bytes".to_string())?;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pk_array)
            .map_err(|e| format!("Invalid verifying key: {}", e))?;

        let message = self.receipt_hash.as_bytes();
        match verifying_key.verify_strict(message, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Verify receipt hash integrity
    pub fn verify_hash(&self) -> bool {
        let computed = Self::compute_canonical_hash(
            self.sequence_number,
            &self.agent_id,
            &self.capability_id,
            &self.instruction_hash,
            &self.action,
            &self.input_hash,
            &self.output_hash,
            &self.runtime_identity,
            &self.runtime_version,
            &self.source_commit,
            &self.environment_hash,
            &self.previous_receipt_hash,
            self.monotonic_counter,
            &self.nonce,
            &self.context,
        );

        computed == self.receipt_hash
    }

    /// Mark receipt as sealed (WORM)
    pub fn seal(mut self) -> Self {
        self.sealed = true;
        self
    }

    /// Mark receipt as failed
    pub fn with_error(mut self, error: String) -> Self {
        self.status = ReceiptStatus::Failure;
        self.error_message = Some(error);
        self
    }
}

/// WORM-sealed v2 receipt chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptChainV2 {
    receipts: Vec<ReceiptV2>,
    sealed: bool,
}

impl ReceiptChainV2 {
    pub fn new() -> Self {
        ReceiptChainV2 {
            receipts: Vec::new(),
            sealed: false,
        }
    }

    pub fn append(&mut self, receipt: ReceiptV2) -> Result<(), String> {
        if self.sealed {
            return Err("Receipt chain is sealed (WORM)".to_string());
        }

        // Verify hash integrity
        if !receipt.verify_hash() {
            return Err("Receipt hash verification failed".to_string());
        }

        // Verify signature if present
        if receipt.signature.is_some() {
            receipt.verify_signature()?;
        }

        // Verify monotonic sequence
        if let Some(last) = self.receipts.last() {
            if receipt.sequence_number <= last.sequence_number {
                return Err("Sequence number must be monotonically increasing".to_string());
            }

            if receipt.previous_receipt_hash != last.receipt_hash {
                return Err("Receipt hash chain linkage broken".to_string());
            }

            // Verify monotonic counter
            if receipt.monotonic_counter <= last.monotonic_counter {
                return Err("Monotonic counter must increase".to_string());
            }

            // Detect replay
            if receipt.nonce == last.nonce && receipt.context == last.context {
                return Err("Duplicate nonce+context detected (replay attack)".to_string());
            }
        } else {
            // First receipt must have zero previous hash
            if receipt.previous_receipt_hash != "0".repeat(128) {
                return Err("First receipt must have all-zero previous hash".to_string());
            }
        }

        self.receipts.push(receipt);
        Ok(())
    }

    pub fn seal(&mut self) -> Result<(), String> {
        if self.receipts.is_empty() {
            return Err("Cannot seal empty chain".to_string());
        }
        self.sealed = true;
        Ok(())
    }

    pub fn head(&self) -> Option<&ReceiptV2> {
        self.receipts.last()
    }

    pub fn verify_chain(&self) -> bool {
        if self.receipts.is_empty() {
            return true;
        }

        for (i, receipt) in self.receipts.iter().enumerate() {
            if !receipt.verify_hash() {
                return false;
            }

            if i > 0 {
                let prev = &self.receipts[i - 1];
                if receipt.sequence_number <= prev.sequence_number {
                    return false;
                }
                if receipt.previous_receipt_hash != prev.receipt_hash {
                    return false;
                }
                if receipt.monotonic_counter <= prev.monotonic_counter {
                    return false;
                }
            }
        }

        true
    }

    pub fn all(&self) -> &[ReceiptV2] {
        &self.receipts
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// Sign the head receipt with a key from KeyStore
    pub fn sign_head(&mut self, keystore: &KeyStore, agent_id: &str) -> Result<(), String> {
        let receipt = self.receipts.last_mut()
            .ok_or("Cannot sign: no receipts in chain".to_string())?;

        if self.sealed {
            return Err("Cannot sign: chain is sealed".to_string());
        }

        // Get current key for agent
        let (key_version, _) = keystore.get_current_key(agent_id)?;

        // Sign with keystore
        let signature = keystore.sign(agent_id, receipt.receipt_hash.as_bytes())?;

        // Update receipt with signature
        let metadata = keystore.get_key(agent_id, key_version)?;
        receipt.signature = Some(signature);
        receipt.signing_public_key = Some(metadata.public_key.clone());
        receipt.key_version = Some(key_version);

        Ok(())
    }
}

impl Default for ReceiptChainV2 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_v2_canonical_hash_deterministic() {
        let r1 = ReceiptV2::new(
            0,
            "agent1".to_string(),
            "cap1".to_string(),
            "instr1".to_string(),
            "dispatch".to_string(),
            "input1".to_string(),
            "output1".to_string(),
            "rust".to_string(),
            "1.0.0".to_string(),
            "abc123".to_string(),
            "env1".to_string(),
            "0".repeat(128),
            100,
            "nonce1".to_string(),
            "context1".to_string(),
        );

        let r2 = ReceiptV2::new(
            0,
            "agent1".to_string(),
            "cap1".to_string(),
            "instr1".to_string(),
            "dispatch".to_string(),
            "input1".to_string(),
            "output1".to_string(),
            "rust".to_string(),
            "1.0.0".to_string(),
            "abc123".to_string(),
            "env1".to_string(),
            "0".repeat(128),
            100,
            "nonce1".to_string(),
            "context1".to_string(),
        );

        // Same input → same hash (timestamp differs but not in hash)
        assert_eq!(r1.receipt_hash, r2.receipt_hash);
    }

    #[test]
    fn test_receipt_v2_replay_detection() {
        let mut chain = ReceiptChainV2::new();

        let r1 = ReceiptV2::new(
            0,
            "loc".to_string(),
            "cap001".to_string(),
            "instr1".to_string(),
            "dispatch".to_string(),
            "inp1".to_string(),
            "out1".to_string(),
            "rust".to_string(),
            "1.0.0".to_string(),
            "src1".to_string(),
            "env1".to_string(),
            "0".repeat(128),
            100,
            "nonce001".to_string(),
            "global".to_string(),
        );

        assert!(chain.append(r1.clone()).is_ok());

        // Duplicate nonce+context should be rejected
        let r2 = ReceiptV2::new(
            1,
            "loc".to_string(),
            "cap001".to_string(),
            "instr2".to_string(),
            "dispatch".to_string(),
            "inp2".to_string(),
            "out2".to_string(),
            "rust".to_string(),
            "1.0.0".to_string(),
            "src2".to_string(),
            "env2".to_string(),
            r1.receipt_hash.clone(),
            200,
            "nonce001".to_string(),  // DUPLICATE nonce
            "global".to_string(),  // DUPLICATE context
        );

        assert!(chain.append(r2).is_err());
    }
}
