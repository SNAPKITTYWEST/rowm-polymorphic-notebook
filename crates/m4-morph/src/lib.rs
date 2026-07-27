//! M4 Macro Engine — Syntactic Morphing with State Feedback Loops
//!
//! Wraps GNU M4 with:
//! - Capability sandboxing (denied builtins, recursion limits)
//! - State feedback (Cell N output → M4 definitions for Cell N+1)
//! - Deterministic expansion

pub mod engine;
pub mod sandbox;
pub mod feedback;

pub use engine::M4Engine;
pub use sandbox::{SandboxLimits, SandboxConfig};
pub use feedback::FeedbackBuffer;
