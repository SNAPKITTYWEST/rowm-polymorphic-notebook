//! Invariant Extractor — Abstract Interpretation over SUBLEQ Memory States
//!
//! Derives loop invariants, pre/post conditions via symbolic execution
//! and abstract domain analysis. Outputs predicates in SMT-LIB format.

pub mod symbolic;
pub mod abstract_domain;
pub mod extractor;
pub mod predicates;
pub mod subleq_analysis;

pub use symbolic::{SymbolicState, SymbolicValue};
pub use abstract_domain::{AbstractDomain, Interval};
pub use extractor::InvariantExtractor;
pub use predicates::Predicate;
pub use subleq_analysis::{SubleqPattern, PatternMatcher};
