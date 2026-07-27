//! Proof Validator — Curry-Howard Isomorphism Checker + WORM Rollback
//!
//! Verifies that self-modification steps preserve extracted invariants.
//! On violation: rollback to last valid WORM checkpoint.

pub mod proof_ir;
pub mod checker;
pub mod validator;
pub mod rollback;
pub mod schema;

pub use proof_ir::{ProofTerm, ProofContext, ProofObligation};
pub use checker::TypeChecker;
pub use validator::ProofValidator;
pub use rollback::RollbackManager;
pub use schema::{ProofEvent, ViolationEvent};
