//! Predicates — logical assertions used in invariants

use crate::symbolic::SymbolicValue;
use serde::{Deserialize, Serialize};

/// Logical predicate for invariants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    /// True (tautology)
    True,
    /// False (contradiction)
    False,
    /// Equality: left == right
    Eq(SymbolicValue, SymbolicValue),
    /// Less than or equal: left <= right
    Le(SymbolicValue, SymbolicValue),
    /// Less than: left < right
    Lt(SymbolicValue, SymbolicValue),
    /// Greater than or equal: left >= right
    Ge(SymbolicValue, SymbolicValue),
    /// Greater than: left > right
    Gt(SymbolicValue, SymbolicValue),
    /// Not equal: left != right
    Ne(SymbolicValue, SymbolicValue),
    /// Conjunction: p1 AND p2
    And(Box<Predicate>, Box<Predicate>),
    /// Disjunction: p1 OR p2
    Or(Box<Predicate>, Box<Predicate>),
    /// Negation: NOT p
    Not(Box<Predicate>),
    /// Implication: p1 => p2
    Implies(Box<Predicate>, Box<Predicate>),
    /// Loop invariant at a specific address
    LoopInvariant {
        header: usize,
        predicate: Box<Predicate>,
    },
}

impl Predicate {
    pub fn true_() -> Self {
        Predicate::True
    }

    pub fn false_() -> Self {
        Predicate::False
    }

    pub fn and(left: Self, right: Self) -> Self {
        Predicate::And(Box::new(left), Box::new(right))
    }

    pub fn or(left: Self, right: Self) -> Self {
        Predicate::Or(Box::new(left), Box::new(right))
    }

    pub fn not(p: Self) -> Self {
        Predicate::Not(Box::new(p))
    }

    pub fn implies(p1: Self, p2: Self) -> Self {
        Predicate::Implies(Box::new(p1), Box::new(p2))
    }

    /// Convert to SMT-LIB format
    pub fn to_smt_lib(&self) -> String {
        match self {
            Predicate::True => "true".into(),
            Predicate::False => "false".into(),
            Predicate::Eq(left, right) => {
                format!("(= {} {})", left.to_smt_lib(), right.to_smt_lib())
            }
            Predicate::Le(left, right) => {
                format!("(<= {} {})", left.to_smt_lib(), right.to_smt_lib())
            }
            Predicate::Lt(left, right) => {
                format!("(< {} {})", left.to_smt_lib(), right.to_smt_lib())
            }
            Predicate::Ge(left, right) => {
                format!("(>= {} {})", left.to_smt_lib(), right.to_smt_lib())
            }
            Predicate::Gt(left, right) => {
                format!("(> {} {})", left.to_smt_lib(), right.to_smt_lib())
            }
            Predicate::Ne(left, right) => {
                format!("(not (= {} {}))", left.to_smt_lib(), right.to_smt_lib())
            }
            Predicate::And(p1, p2) => {
                format!("(and {} {})", p1.to_smt_lib(), p2.to_smt_lib())
            }
            Predicate::Or(p1, p2) => {
                format!("(or {} {})", p1.to_smt_lib(), p2.to_smt_lib())
            }
            Predicate::Not(p) => {
                format!("(not {})", p.to_smt_lib())
            }
            Predicate::Implies(p1, p2) => {
                format!("(=> {} {})", p1.to_smt_lib(), p2.to_smt_lib())
            }
            Predicate::LoopInvariant { header, predicate } => {
                format!("(loop-inv {} {})", header, predicate.to_smt_lib())
            }
        }
    }

    /// Check if predicate is unsatisfiable (always false)
    pub fn is_unsat(&self, _context: &Predicate) -> bool {
        matches!(self, Predicate::False)
    }

    /// Create from interval domain
    pub fn from_interval(addr: usize, iv: crate::abstract_domain::Interval) -> Self {
        if let Some((min, max)) = iv.get(addr) {
            Predicate::and(
                Predicate::Le(
                    SymbolicValue::Const(min),
                    SymbolicValue::Mem(addr),
                ),
                Predicate::Le(
                    SymbolicValue::Mem(addr),
                    SymbolicValue::Const(max),
                ),
            )
        } else {
            Predicate::True
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicate_smt() {
        let pred = Predicate::Eq(SymbolicValue::Const(1), SymbolicValue::Const(1));
        let smt = pred.to_smt_lib();
        assert!(smt.contains("="));
    }

    #[test]
    fn test_predicate_and() {
        let p1 = Predicate::True;
        let p2 = Predicate::True;
        let conj = Predicate::and(p1, p2);
        let smt = conj.to_smt_lib();
        assert!(smt.contains("and"));
    }

    #[test]
    fn test_predicate_unsat() {
        assert!(Predicate::False.is_unsat(&Predicate::True));
        assert!(!Predicate::True.is_unsat(&Predicate::True));
    }
}
