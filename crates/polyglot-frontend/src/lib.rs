//! Polyglot Frontend — 30+ language parsers
//!
//! All languages parse to unified AST through language-specific parsers
//! implementing the Parser trait.

pub mod registry;
pub mod parsers;
pub mod language;

pub use registry::{LanguageRegistry, Parser};
pub use language::{Language, LanguageTier};
