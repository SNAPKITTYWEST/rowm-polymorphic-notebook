//! Abstract Syntax Tree — Unified representation for all languages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub metadata: ProgramMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    pub source_language: String,
    pub source_file: Option<String>,
    pub timestamp: Option<u64>,
}

/// Statements — top-level definitions and control flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    /// Variable/function definition
    Define {
        name: String,
        value: Box<Expr>,
        ty: Type,
    },
    /// Expression statement (side effects)
    Expr(Box<Expr>),
    /// If-then-else
    If {
        cond: Box<Expr>,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    /// While loop
    While {
        cond: Box<Expr>,
        body: Vec<Stmt>,
    },
    /// For loop
    For {
        var: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Vec<Stmt>,
    },
    /// Return statement
    Return(Option<Box<Expr>>),
    /// Function definition
    Function {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<Stmt>,
    },
}

/// Expressions — values and computations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal values
    Literal(Literal),
    /// Variable reference
    Var(String),
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    /// Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Array/vector indexing
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    /// Array construction
    Array(Vec<Expr>),
    /// Array slice
    Slice {
        expr: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    /// Object/record construction
    Record(HashMap<String, Expr>),
    /// Property access
    Property {
        expr: Box<Expr>,
        prop: String,
    },
    /// Type cast
    Cast {
        expr: Box<Expr>,
        target_type: Type,
    },
    /// Lambda/anonymous function
    Lambda {
        params: Vec<(String, Type)>,
        body: Box<Expr>,
        return_type: Type,
    },
    /// Block expression
    Block(Vec<Stmt>, Option<Box<Expr>>),
    /// Conditional expression (ternary)
    Cond {
        test: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
}

/// Literal values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
    Nil,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    Xor,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,      // -x
    Not,      // !x
    BitNot,   // ~x
    Abs,      // |x|
}

/// Types — static type information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    I64,
    F64,
    Bool,
    String,
    Unit,
    /// Compound types
    Array(Box<Type>),
    Vector(Box<Type>),
    Tuple(Vec<Type>),
    Record(HashMap<String, Type>),
    /// Function type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// Generic/polymorphic
    Generic(String),
    /// Inferred type (to be resolved)
    Inferred,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::I64 | Type::F64)
    }

    pub fn is_integral(&self) -> bool {
        matches!(self, Type::I64 | Type::Bool)
    }

    pub fn width_bits(&self) -> Option<usize> {
        match self {
            Type::I64 => Some(64),
            Type::F64 => Some(64),
            Type::Bool => Some(1),
            _ => None,
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Inferred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Literal(Literal::Int(1))),
            right: Box::new(Expr::Literal(Literal::Int(2))),
        };

        let stmt = Stmt::Expr(Box::new(expr));
        let program = Program {
            stmts: vec![stmt],
            metadata: ProgramMetadata {
                source_language: "test".into(),
                source_file: None,
                timestamp: None,
            },
        };

        assert_eq!(program.stmts.len(), 1);
    }

    #[test]
    fn test_type_properties() {
        assert!(Type::I64.is_numeric());
        assert!(Type::I64.is_integral());
        assert_eq!(Type::I64.width_bits(), Some(64));
    }
}
