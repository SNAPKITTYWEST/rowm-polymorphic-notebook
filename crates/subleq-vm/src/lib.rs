//! SUBLEQ Virtual Machine — Sovereign Execution Substrate
//!
//! Implements the One-Instruction Set Computer (OISC) with self-modification
//! hooks, mutation logging, and WORM checkpointing for rollback.

pub mod vm;
pub mod memory;
pub mod checkpoint;
pub mod telemetry;

pub use vm::{SubleqVM, Instruction, ExecMode};
pub use memory::{Memory, Address};
pub use checkpoint::{Checkpoint, CheckpointManager};
pub use telemetry::{TelemetryEmitter, MutationDelta};
