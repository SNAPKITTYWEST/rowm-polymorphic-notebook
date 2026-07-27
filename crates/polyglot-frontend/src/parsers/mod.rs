//! Language-specific parser implementations

pub mod python;
pub mod javascript;
pub mod rust;
pub mod custom;

pub use python::PythonParser;
pub use javascript::JavaScriptParser;
pub use rust::RustParser;
pub use custom::{SubleqParser, LispParser, PrologParser, ForthParser};
