//! Stack-based bytecode intermediate form
//!
//! All 30+ languages compile to this bytecode.
//! Bytecode then compiles to SUBLEQ memory layout.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bytecode program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    pub instructions: Vec<Instr>,
    pub constants: Vec<Value>,
    pub functions: HashMap<String, FunctionDescriptor>,
    pub metadata: BytecodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeMetadata {
    pub version: u32,
    pub source_language: String,
    pub timestamp: Option<u64>,
}

/// Single bytecode instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instr {
    /// Load constant into register
    LoadConst { reg: Register, idx: usize },
    /// Load variable
    LoadVar { reg: Register, name: String },
    /// Store register to variable
    StoreVar { name: String, reg: Register },
    /// Move between registers
    Move { dst: Register, src: Register },
    /// Binary operation: dst = src1 OP src2
    BinOp {
        dst: Register,
        op: BinOpCode,
        src1: Register,
        src2: Register,
    },
    /// Unary operation: dst = OP src
    UnOp {
        dst: Register,
        op: UnOpCode,
        src: Register,
    },
    /// Array index: dst = src[idx]
    Index {
        dst: Register,
        src: Register,
        idx: Register,
    },
    /// Array store: dst[idx] = src
    IndexStore {
        dst: Register,
        idx: Register,
        src: Register,
    },
    /// Call function
    Call {
        func: String,
        args: Vec<Register>,
        ret: Register,
    },
    /// Jump to label
    Jump(String),
    /// Conditional jump
    JumpIf {
        cond: Register,
        label: String,
    },
    /// Jump if zero
    JumpIfZero {
        cond: Register,
        label: String,
    },
    /// Label (target for jumps)
    Label(String),
    /// Return from function
    Return(Option<Register>),
    /// No-op
    Nop,
}

/// Register identifiers (16 registers, R0-R15)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register(pub u8);

impl Register {
    pub const R0: Register = Register(0);
    pub const R1: Register = Register(1);
    pub const R2: Register = Register(2);
    pub const R3: Register = Register(3);
    pub const R4: Register = Register(4);
    pub const R5: Register = Register(5);
    pub const R6: Register = Register(6);
    pub const R7: Register = Register(7);
    pub const R8: Register = Register(8);
    pub const R9: Register = Register(9);
    pub const R10: Register = Register(10);
    pub const R11: Register = Register(11);
    pub const R12: Register = Register(12);
    pub const R13: Register = Register(13);
    pub const R14: Register = Register(14);
    pub const R15: Register = Register(15);

    pub fn is_valid(&self) -> bool {
        self.0 < 16
    }
}

/// Binary operation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOpCode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
}

/// Unary operation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOpCode {
    Neg,
    Not,
    BitNot,
    Abs,
}

/// Runtime values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

impl Value {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Int(n) => Some(*n != 0),
            _ => None,
        }
    }
}

/// Function descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDescriptor {
    pub name: String,
    pub params: Vec<String>,
    pub entry_point: usize, // Instruction index
    pub locals: usize,       // Number of local variables
}

impl Bytecode {
    pub fn new(source_language: &str) -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            functions: HashMap::new(),
            metadata: BytecodeMetadata {
                version: 1,
                source_language: source_language.to_string(),
                timestamp: None,
            },
        }
    }

    pub fn add_instr(&mut self, instr: Instr) {
        self.instructions.push(instr);
    }

    pub fn add_const(&mut self, val: Value) -> usize {
        let idx = self.constants.len();
        self.constants.push(val);
        idx
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn const_count(&self) -> usize {
        self.constants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_validity() {
        assert!(Register::R0.is_valid());
        assert!(Register::R15.is_valid());
        assert!(!Register(16).is_valid());
    }

    #[test]
    fn test_bytecode_creation() {
        let mut bc = Bytecode::new("test");
        let idx = bc.add_const(Value::Int(42));
        assert_eq!(idx, 0);
        assert_eq!(bc.const_count(), 1);
    }

    #[test]
    fn test_value_conversions() {
        assert_eq!(Value::Int(5).as_i64(), Some(5));
        assert_eq!(Value::Bool(true).as_i64(), Some(1));
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
    }
}
