//! Symbolic execution — tracking values as symbolic expressions

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Symbolic value — an expression, not a concrete number
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolicValue {
    /// Concrete integer
    Const(i64),
    /// Variable reference (register or memory address)
    Mem(usize),
    /// Register reference
    Reg(u8),
    /// Binary operation: op(left, right)
    BinOp {
        op: String,
        left: Box<SymbolicValue>,
        right: Box<SymbolicValue>,
    },
    /// Unary operation: op(val)
    UnOp {
        op: String,
        val: Box<SymbolicValue>,
    },
    /// Conditional: if cond then true_val else false_val
    Ite {
        cond: Box<SymbolicValue>,
        true_val: Box<SymbolicValue>,
        false_val: Box<SymbolicValue>,
    },
}

impl SymbolicValue {
    pub fn add(left: Self, right: Self) -> Self {
        Self::BinOp {
            op: "add".into(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn sub(left: Self, right: Self) -> Self {
        Self::BinOp {
            op: "sub".into(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn mul(left: Self, right: Self) -> Self {
        Self::BinOp {
            op: "mul".into(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn le(left: Self, right: Self) -> Self {
        Self::BinOp {
            op: "le".into(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn eq(left: Self, right: Self) -> Self {
        Self::BinOp {
            op: "eq".into(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn neg(val: Self) -> Self {
        Self::UnOp {
            op: "neg".into(),
            val: Box::new(val),
        }
    }

    pub fn to_smt_lib(&self) -> String {
        match self {
            SymbolicValue::Const(n) => n.to_string(),
            SymbolicValue::Mem(addr) => format!("(mem {})", addr),
            SymbolicValue::Reg(r) => format!("(reg {})", r),
            SymbolicValue::BinOp { op, left, right } => {
                format!("({} {} {})", op, left.to_smt_lib(), right.to_smt_lib())
            }
            SymbolicValue::UnOp { op, val } => {
                format!("({} {})", op, val.to_smt_lib())
            }
            SymbolicValue::Ite { cond, true_val, false_val } => {
                format!(
                    "(ite {} {} {})",
                    cond.to_smt_lib(),
                    true_val.to_smt_lib(),
                    false_val.to_smt_lib()
                )
            }
        }
    }
}

/// Symbolic execution state — memory and variable values as symbolic expressions
#[derive(Debug, Clone)]
pub struct SymbolicState {
    values: HashMap<usize, SymbolicValue>,
}

impl SymbolicState {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.values.clear();
    }

    pub fn get_or_create(&mut self, addr: usize) -> SymbolicValue {
        self.values
            .entry(addr)
            .or_insert_with(|| SymbolicValue::Mem(addr))
            .clone()
    }

    pub fn assign(&mut self, addr: usize, val: SymbolicValue) {
        self.values.insert(addr, val);
    }

    pub fn get(&self, addr: usize) -> Option<&SymbolicValue> {
        self.values.get(&addr)
    }

    pub fn all_values(&self) -> impl Iterator<Item = (&usize, &SymbolicValue)> {
        self.values.iter()
    }
}

impl Default for SymbolicState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbolic_const() {
        let val = SymbolicValue::Const(42);
        assert_eq!(val.to_smt_lib(), "42");
    }

    #[test]
    fn test_symbolic_binop() {
        let expr = SymbolicValue::add(
            SymbolicValue::Const(1),
            SymbolicValue::Const(2),
        );
        assert!(expr.to_smt_lib().contains("add"));
    }

    #[test]
    fn test_symbolic_state() {
        let mut state = SymbolicState::new();
        let val = state.get_or_create(0);
        assert_eq!(val, SymbolicValue::Mem(0));

        state.assign(0, SymbolicValue::Const(42));
        assert_eq!(state.get(0), Some(&SymbolicValue::Const(42)));
    }
}
