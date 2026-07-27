//! Unified Intermediate Representation
//!
//! - Common AST nodes for all 30+ languages
//! - Stack-based bytecode intermediate form
//! - SUBLEQ memory layout code generation

pub mod ast;
pub mod bytecode;
pub mod lowering;
pub mod subleq_codegen;

pub use ast::{Expr, Stmt, Program, Type};
pub use bytecode::{Bytecode, Opcode, Register};
pub use lowering::Lowerer;
pub use subleq_codegen::SubleqCodegen;
