// WORM-sealed receipt chain implementation

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Single receipt entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub schema_version: String,
    pub sequence_number: u64,
    pub receipt_id: String,
    pub receipt_hash: String,
    pub previous_receipt_hash: String,
    pub agent_id: String,
    pub capability_id: String,
    pub instruction_hash: String,
    pub action: String,
    pub input_hash: String,
    pub output_hash: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub timestamp: u64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Receipt {
    /// Create a new receipt
    pub fn new(
        sequence_number: u64,
        agent_id: String,
        capability_id: String,
        instruction_hash: String,
        action: String,
        input_hash: String,
        output_hash: String,
        runtime_identity: String,
        previous_receipt_hash: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let receipt_id = format!("rcpt_{:07}_{}", sequence_number, agent_id);
        let receipt_hash = Self::compute_hash(
            sequence_number,
            &agent_id,
            &instruction_hash,
            &action,
            &input_hash,
            &output_hash,
            timestamp,
            &previous_receipt_hash,
        );

        Receipt {
            schema_version: "1.0".to_string(),
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
            runtime_version: "1.0.0".to_string(),
            timestamp,
            status: "success".to_string(),
            error_message: None,
        }
    }

    /// Compute SHA-256 hash of receipt
    fn compute_hash(
        seq: u64,
        agent: &str,
        instr_hash: &str,
        action: &str,
        input_hash: &str,
        output_hash: &str,
        timestamp: u64,
        prev_hash: &str,
    ) -> String {
        use sha2::{Sha256, Digest};

        let payload = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}",
            seq, agent, instr_hash, action, input_hash, output_hash, timestamp, prev_hash
        );

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Mark receipt as failed
    pub fn with_error(mut self, error: String) -> Self {
        self.status = "failure".to_string();
        self.error_message = Some(error);
        self
    }

    /// Mark receipt as sealed
    pub fn seal(mut self) -> Self {
        self.status = "sealed".to_string();
        self
    }

    /// Verify receipt integrity
    pub fn verify(&self) -> bool {
        let computed = Self::compute_hash(
            self.sequence_number,
            &self.agent_id,
            &self.instruction_hash,
            &self.action,
            &self.input_hash,
            &self.output_hash,
            self.timestamp,
            &self.previous_receipt_hash,
        );

        computed == self.receipt_hash
    }
}

/// WORM-sealed receipt chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptChain {
    receipts: Vec<Receipt>,
    sealed: bool,
}

impl ReceiptChain {
    /// Create new receipt chain
    pub fn new() -> Self {
        ReceiptChain {
            receipts: Vec::new(),
            sealed: false,
        }
    }

    /// Append receipt to chain
    pub fn append(&mut self, receipt: Receipt) -> Result<(), String> {
        if self.sealed {
            return Err("Receipt chain is sealed".to_string());
        }

        // Verify chain continuity
        if let Some(last) = self.receipts.last() {
            if receipt.sequence_number <= last.sequence_number {
                return Err("Sequence number must be monotonic".to_string());
            }

            if receipt.previous_receipt_hash != last.receipt_hash {
                return Err("Receipt hash chain is broken".to_string());
            }
        } else {
            // First receipt must have zero previous hash
            if receipt.previous_receipt_hash != "0".repeat(64) {
                return Err("First receipt must have zero previous hash".to_string());
            }
        }

        // Verify receipt integrity
        if !receipt.verify() {
            return Err("Receipt hash verification failed".to_string());
        }

        self.receipts.push(receipt);
        Ok(())
    }

    /// Seal the receipt chain (make it read-only)
    pub fn seal(&mut self) -> Result<(), String> {
        if self.receipts.is_empty() {
            return Err("Cannot seal empty chain".to_string());
        }

        self.sealed = true;
        Ok(())
    }

    /// Get chain head
    pub fn head(&self) -> Option<&Receipt> {
        self.receipts.last()
    }

    /// Verify entire chain integrity
    pub fn verify(&self) -> bool {
        if self.receipts.is_empty() {
            return true;
        }

        for (i, receipt) in self.receipts.iter().enumerate() {
            // Verify individual receipt
            if !receipt.verify() {
                return false;
            }

            // Verify chain continuity
            if i > 0 {
                let prev = &self.receipts[i - 1];
                if receipt.sequence_number <= prev.sequence_number {
                    return false;
                }

                if receipt.previous_receipt_hash != prev.receipt_hash {
                    return false;
                }

                if receipt.timestamp < prev.timestamp {
                    return false;
                }
            } else {
                // First receipt
                if receipt.sequence_number != 0 && receipt.previous_receipt_hash != "0".repeat(64) {
                    return false;
                }
            }
        }

        true
    }

    /// Get all receipts
    pub fn all(&self) -> &[Receipt] {
        &self.receipts
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Check if sealed
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }
}

impl Default for ReceiptChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let receipt = Receipt::new(
            0,
            "loc".to_string(),
            "capa_001".to_string(),
            "abc123".to_string(),
            "dispatch".to_string(),
            "input_hash".to_string(),
            "output_hash".to_string(),
            "rust".to_string(),
            "0".repeat(64),
        );

        assert_eq!(receipt.sequence_number, 0);
        assert_eq!(receipt.status, "success");
        assert!(receipt.verify());
    }

    #[test]
    fn test_receipt_chain_append() {
        let mut chain = ReceiptChain::new();

        let receipt1 = Receipt::new(
            0,
            "loc".to_string(),
            "capa_001".to_string(),
            "instr_hash_1".to_string(),
            "dispatch".to_string(),
            "input1".to_string(),
            "output1".to_string(),
            "rust".to_string(),
            "0".repeat(64),
        );

        assert!(chain.append(receipt1.clone()).is_ok());
        assert_eq!(chain.len(), 1);

        let receipt2 = Receipt::new(
            1,
            "sentinel".to_string(),
            "capa_002".to_string(),
            "instr_hash_2".to_string(),
            "verify".to_string(),
            "input2".to_string(),
            "output2".to_string(),
            "ada".to_string(),
            receipt1.receipt_hash,
        );

        assert!(chain.append(receipt2).is_ok());
        assert_eq!(chain.len(), 2);
        assert!(chain.verify());
    }

    #[test]
    fn test_receipt_chain_seal() {
        let mut chain = ReceiptChain::new();

        let receipt = Receipt::new(
            0,
            "loc".to_string(),
            "capa_001".to_string(),
            "abc123".to_string(),
            "dispatch".to_string(),
            "input".to_string(),
            "output".to_string(),
            "rust".to_string(),
            "0".repeat(64),
        );

        assert!(chain.append(receipt).is_ok());
        assert!(chain.seal().is_ok());
        assert!(chain.is_sealed());

        // Cannot append after seal
        let receipt2 = Receipt::new(
            1,
            "forge".to_string(),
            "capa_002".to_string(),
            "def456".to_string(),
            "execute".to_string(),
            "input2".to_string(),
            "output2".to_string(),
            "holyc".to_string(),
            "0".repeat(64),
        );

        assert!(chain.append(receipt2).is_err());
    }
}
