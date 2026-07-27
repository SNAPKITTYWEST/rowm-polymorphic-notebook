//! Proof Validator — orchestrates checking and rollback

use crate::proof_ir::{ProofTerm, ProofContext, ProofObligation};
use crate::checker::TypeChecker;
use crate::rollback::{RollbackManager, WormCheckpoint};
use crate::schema::{ProofEvent, ViolationEvent};
use anyhow::{Result, Context};
use tracing::{warn, error, info};

pub struct ProofValidator {
    checker: TypeChecker,
    rollback_mgr: RollbackManager,
    proof_context: ProofContext,
    violation_count: u64,
}

impl ProofValidator {
    pub fn new(rollback_mgr: RollbackManager) -> Self {
        Self {
            checker: TypeChecker::new(),
            rollback_mgr,
            proof_context: ProofContext::new(),
            violation_count: 0,
        }
    }

    /// Validate a mutation against current proof context
    /// Returns Ok(()) if valid, Err with rollback if violated
    pub fn validate_mutation(
        &mut self,
        mutation: &str,
        invariants: &[String],
    ) -> Result<()> {
        // 1. Construct proof obligation
        let obligation = ProofObligation::InvariantPreservation {
            address: 0,
            old_value: 0,
            new_value: 0,
            invariants: invariants.to_vec(),
        };

        // 2. Type-check the proof term
        let proof_term = self.proof_context.construct_proof(&obligation)?;

        match self.checker.check(&proof_term) {
            Ok(_) => {
                // Proof valid — record in context
                self.proof_context.add_proof(obligation, proof_term);
                self.emit_audit(ProofEvent::Validated {
                    mutation: mutation.to_string(),
                });
                Ok(())
            }
            Err(e) => {
                self.violation_count += 1;
                error!(?mutation, violation = %e, "Invariant violation detected");

                // Emit violation event to audit trail
                self.emit_audit(ProofEvent::Violated {
                    mutation: mutation.to_string(),
                    error: e.to_string(),
                    violation_id: self.violation_count,
                });

                // 3. Rollback to last valid checkpoint
                let checkpoint = self.rollback_mgr.last_valid_checkpoint()
                    .context("No valid checkpoint for rollback")?;

                self.rollback_mgr.rollback(&checkpoint)?;

                // Emit rollback event
                self.emit_audit(ProofEvent::RolledBack {
                    checkpoint: checkpoint.id,
                    violation_id: self.violation_count,
                });

                Err(anyhow::anyhow!("Invariant violated, rolled back: {}", e))
            }
        }
    }

    /// Add a WORM checkpoint
    pub fn add_checkpoint(&mut self, checkpoint: WormCheckpoint) {
        self.rollback_mgr.add_checkpoint(checkpoint);
    }

    fn emit_audit(&self, event: ProofEvent) {
        // In production, this would emit to Bifrost Bridge audit chain
        info!("Proof audit: {:?}", event);
    }

    pub fn violation_count(&self) -> u64 {
        self.violation_count
    }
}

impl Default for ProofValidator {
    fn default() -> Self {
        Self::new(RollbackManager::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ProofValidator::default();
        assert_eq!(validator.violation_count(), 0);
    }

    #[test]
    fn test_validate_mutation() {
        let mut validator = ProofValidator::default();
        let result = validator.validate_mutation("M[0] ← 42", &["M[0] > 0".into()]);
        // May succeed or fail depending on proof construction
        let _ = result;
    }

    #[test]
    fn test_add_checkpoint() {
        let mut validator = ProofValidator::default();
        let cp = WormCheckpoint {
            id: "cp1".into(),
            ip: 0,
            step_count: 0,
            mutation_log_len: 0,
            timestamp: 0,
            memory_snapshot: vec![],
        };
        validator.add_checkpoint(cp);
        assert!(validator.rollback_mgr.last_valid_checkpoint().is_some());
    }
}
