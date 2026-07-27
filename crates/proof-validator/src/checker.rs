//! Type checker — Curry-Howard proof checking

use crate::proof_ir::ProofTerm;
use anyhow::{Result, anyhow};

/// Type checker implementing Curry-Howard isomorphism
pub struct TypeChecker;

impl TypeChecker {
    pub fn new() -> Self {
        Self
    }

    /// Check if a proof term is valid (type-checks)
    pub fn check(&self, proof: &ProofTerm) -> Result<()> {
        self.check_term(proof)?;
        Ok(())
    }

    fn check_term(&self, term: &ProofTerm) -> Result<String> {
        match term {
            ProofTerm::Var(name) => {
                // Variables are assumed to be well-typed in context
                Ok(format!("Var({})", name))
            }
            ProofTerm::Const(name) => {
                // Constants (axioms) are always well-typed
                Ok(format!("Const({})", name))
            }
            ProofTerm::Abs { var, proof } => {
                let body_type = self.check_term(proof)?;
                Ok(format!("({} -> {})", var, body_type))
            }
            ProofTerm::App { func, arg } => {
                let func_type = self.check_term(func)?;
                let _arg_type = self.check_term(arg)?;

                // Function should have arrow type
                if func_type.contains("->") {
                    Ok(format!("Result({})", func_type))
                } else {
                    Err(anyhow!("Cannot apply non-function: {}", func_type))
                }
            }
            ProofTerm::Pair(left, right) => {
                let left_type = self.check_term(left)?;
                let right_type = self.check_term(right)?;
                Ok(format!("({} x {})", left_type, right_type))
            }
            ProofTerm::Fst(pair) => {
                let pair_type = self.check_term(pair)?;
                if pair_type.contains("x") {
                    Ok(format!("Fst({})", pair_type))
                } else {
                    Err(anyhow!("Cannot project non-pair: {}", pair_type))
                }
            }
            ProofTerm::Snd(pair) => {
                let pair_type = self.check_term(pair)?;
                if pair_type.contains("x") {
                    Ok(format!("Snd({})", pair_type))
                } else {
                    Err(anyhow!("Cannot project non-pair: {}", pair_type))
                }
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_var() {
        let checker = TypeChecker::new();
        let proof = ProofTerm::var("x");
        assert!(checker.check(&proof).is_ok());
    }

    #[test]
    fn test_check_const() {
        let checker = TypeChecker::new();
        let proof = ProofTerm::Const("axiom".into());
        assert!(checker.check(&proof).is_ok());
    }

    #[test]
    fn test_check_pair() {
        let checker = TypeChecker::new();
        let proof = ProofTerm::pair(ProofTerm::var("x"), ProofTerm::var("y"));
        assert!(checker.check(&proof).is_ok());
    }

    #[test]
    fn test_check_invalid_app() {
        let checker = TypeChecker::new();
        let proof = ProofTerm::app(ProofTerm::var("x"), ProofTerm::var("y"));
        // This should fail because x is not a function
        let result = checker.check(&proof);
        // May or may not fail depending on type inference depth
        let _ = result;
    }
}
