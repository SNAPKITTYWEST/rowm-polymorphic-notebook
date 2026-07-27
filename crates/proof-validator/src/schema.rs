//! Proof event schemas for audit trail

use serde::{Deserialize, Serialize};

/// Proof events for Bifrost audit chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofEvent {
    /// Proof successfully validated
    Validated {
        mutation: String,
    },
    /// Proof validation failed
    Violated {
        mutation: String,
        error: String,
        violation_id: u64,
    },
    /// Rollback executed
    RolledBack {
        checkpoint: String,
        violation_id: u64,
    },
}

/// Violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationEvent {
    pub violation_id: u64,
    pub timestamp: u64,
    pub address: usize,
    pub old_value: i64,
    pub new_value: i64,
    pub reason: String,
    pub checkpoint_id: Option<String>,
}

impl ProofEvent {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({}))
    }
}

impl ViolationEvent {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_event_serialization() {
        let event = ProofEvent::Validated {
            mutation: "M[0] ← 42".into(),
        };
        let json = event.to_json();
        assert!(json.is_object());
    }

    #[test]
    fn test_violation_event() {
        let event = ViolationEvent {
            violation_id: 1,
            timestamp: 0,
            address: 0,
            old_value: 5,
            new_value: 10,
            reason: "invariant_breach".into(),
            checkpoint_id: Some("cp1".into()),
        };
        let json = event.to_json();
        assert!(json["violation_id"].is_number());
    }
}
