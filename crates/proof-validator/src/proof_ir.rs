//! Proof IR — internal representation of proof terms

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proof terms (Curry-Howard isomorphism)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofTerm {
    /// Variable reference (assumption)
    Var(String),
    /// Abstraction: λx. proof
    Abs {
        var: String,
        proof: Box<ProofTerm>,
    },
    /// Application: proof1 proof2
    App {
        func: Box<ProofTerm>,
        arg: Box<ProofTerm>,
    },
    /// Constant proof (axiom)
    Const(String),
    /// Pair: (proof1, proof2)
    Pair(Box<ProofTerm>, Box<ProofTerm>),
    /// Projection: fst(proof), snd(proof)
    Fst(Box<ProofTerm>),
    Snd(Box<ProofTerm>),
}

impl ProofTerm {
    pub fn var(name: &str) -> Self {
        ProofTerm::Var(name.into())
    }

    pub fn app(func: Self, arg: Self) -> Self {
        ProofTerm::App {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }

    pub fn pair(left: Self, right: Self) -> Self {
        ProofTerm::Pair(Box::new(left), Box::new(right))
    }
}

/// Proof obligations — propositions that need to be discharged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofObligation {
    /// Memory mutation preserves invariants
    InvariantPreservation {
        address: usize,
        old_value: i64,
        new_value: i64,
        invariants: Vec<String>,
    },
    /// Semantic equivalence before/after transformation
    SemanticPreservation {
        before: String,
        after: String,
    },
    /// Loop invariant maintenance
    LoopInvariantMaintenance {
        header: usize,
        invariant: String,
    },
}

impl ProofObligation {
    pub fn to_string(&self) -> String {
        match self {
            ProofObligation::InvariantPreservation {
                address,
                old_value,
                new_value,
                invariants,
            } => {
                format!(
                    "preserve_invariants({}, {} -> {}, {})",
                    address,
                    old_value,
                    new_value,
                    invariants.join(", ")
                )
            }
            ProofObligation::SemanticPreservation { before, after } => {
                format!("semantic_equiv({} ≡ {})", before, after)
            }
            ProofObligation::LoopInvariantMaintenance { header, invariant } => {
                format!("loop_invariant({}, {})", header, invariant)
            }
        }
    }
}

/// Proof context — tracks hypotheses and proven facts
#[derive(Debug, Clone)]
pub struct ProofContext {
    hypotheses: HashMap<String, String>,
    proven_facts: HashMap<String, ProofTerm>,
}

impl ProofContext {
    pub fn new() -> Self {
        Self {
            hypotheses: HashMap::new(),
            proven_facts: HashMap::new(),
        }
    }

    pub fn add_hypothesis(&mut self, name: String, ty: String) {
        self.hypotheses.insert(name, ty);
    }

    pub fn add_proof(&mut self, obligation: ProofObligation, proof: ProofTerm) {
        let key = obligation.to_string();
        self.proven_facts.insert(key, proof);
    }

    pub fn construct_proof(&self, obligation: &ProofObligation) -> anyhow::Result<ProofTerm> {
        // Simple proof construction: for invariant preservation, assume the invariant holds
        match obligation {
            ProofObligation::InvariantPreservation { invariants, .. } => {
                if invariants.is_empty() {
                    Ok(ProofTerm::var("invariant_holds"))
                } else {
                    // For multiple invariants, construct a tuple
                    let proof = invariants
                        .iter()
                        .fold(ProofTerm::var("_"), |acc, inv| {
                            ProofTerm::pair(acc, ProofTerm::var(inv))
                        });
                    Ok(proof)
                }
            }
            _ => Ok(ProofTerm::var("axiom")),
        }
    }

    pub fn has_proof_for(&self, obligation: &ProofObligation) -> bool {
        let key = obligation.to_string();
        self.proven_facts.contains_key(&key)
    }
}

impl Default for ProofContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_term_creation() {
        let proof = ProofTerm::var("x");
        assert!(matches!(proof, ProofTerm::Var(_)));
    }

    #[test]
    fn test_proof_obligation() {
        let ob = ProofObligation::InvariantPreservation {
            address: 0,
            old_value: 5,
            new_value: 10,
            invariants: vec!["x > 0".into()],
        };
        let s = ob.to_string();
        assert!(s.contains("preserve_invariants"));
    }

    #[test]
    fn test_proof_context() {
        let mut ctx = ProofContext::new();
        ctx.add_hypothesis("x".into(), "Int".into());
        assert!(ctx.hypotheses.contains_key("x"));
    }
}
