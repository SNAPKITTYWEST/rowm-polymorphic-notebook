//! Invariant Extractor — main orchestrator

use crate::symbolic::{SymbolicState, SymbolicValue};
use crate::abstract_domain::{AbstractDomain, Interval};
use crate::predicates::Predicate;
use crate::subleq_analysis::{SubleqPattern, PatternMatcher};
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

pub struct InvariantExtractor {
    extracted_predicates: Vec<Predicate>,
    symbolic_state: SymbolicState,
    abstract_domain: Box<dyn AbstractDomain>,
    pattern_matcher: PatternMatcher,
    loop_headers: HashSet<usize>,
    invariant_cache: HashMap<usize, Vec<Predicate>>,
}

impl InvariantExtractor {
    pub fn new() -> Self {
        Self {
            extracted_predicates: Vec::new(),
            symbolic_state: SymbolicState::new(),
            abstract_domain: Box::new(Interval::new()),
            pattern_matcher: PatternMatcher::new(),
            loop_headers: HashSet::new(),
            invariant_cache: HashMap::new(),
        }
    }

    /// Analyze a SUBLEQ memory block and extract invariants
    pub fn analyze_block(&mut self, bytecode: &[i64], entry_ip: usize) -> Result<&Vec<Predicate>> {
        info!(entry_ip, len = bytecode.len(), "Starting invariant extraction");

        // 1. Identify control flow structure (loop headers, basic blocks)
        self.discover_loops(bytecode, entry_ip)?;

        // 2. Symbolic execution from entry point
        self.symbolic_state.reset();
        self.execute_symbolic(bytecode, entry_ip)?;

        // 3. Abstract interpretation over memory states
        self.abstract_interpret(bytecode, entry_ip)?;

        // 4. Pattern matching for known SUBLEQ idioms
        self.match_patterns(bytecode)?;

        // 5. Generate SMT-LIB predicates
        self.generate_predicates()?;

        Ok(&self.extracted_predicates)
    }

    /// Discover loop headers via backward branch analysis
    fn discover_loops(&mut self, bytecode: &[i64], entry: usize) -> Result<()> {
        let mut visited = HashSet::new();
        let mut stack = vec![entry];

        while let Some(ip) = stack.pop() {
            if !visited.insert(ip) {
                continue;
            }
            if ip + 2 >= bytecode.len() {
                continue;
            }

            let c = bytecode[ip + 2] as usize;

            // Forward edge
            if ip + 3 < bytecode.len() {
                stack.push(ip + 3);
            }

            // Backward edge → loop header
            if c < ip {
                self.loop_headers.insert(c);
            }

            if c < bytecode.len() {
                stack.push(c);
            }
        }

        debug!(count = self.loop_headers.len(), "Discovered loop headers");
        Ok(())
    }

    /// Symbolic execution over SUBLEQ instructions
    fn execute_symbolic(&mut self, bytecode: &[i64], mut ip: usize) -> Result<()> {
        const MAX_STEPS: u64 = 10_000;
        let mut steps = 0;

        while ip + 2 < bytecode.len() && steps < MAX_STEPS {
            let a = bytecode[ip] as usize;
            let b = bytecode[ip + 1] as usize;
            let c = bytecode[ip + 2] as usize;

            // Symbolic values
            let sym_a = self.symbolic_state.get_or_create(a);
            let sym_b = self.symbolic_state.get_or_create(b);

            // M[b] ← M[b] - M[a]
            let sym_res = SymbolicValue::sub(sym_b, sym_a);
            self.symbolic_state.assign(b, sym_res.clone());

            // Record invariant at loop headers
            if self.loop_headers.contains(&ip) {
                let inv = self.abstract_domain.widen_at(ip, &self.symbolic_state);
                self.invariant_cache.entry(ip).or_default().push(inv);
            }

            // Choose path
            ip = if self.should_branch(&sym_res) { c } else { ip + 3 };
            steps += 1;
        }

        Ok(())
    }

    /// Abstract interpretation with widening
    fn abstract_interpret(&mut self, bytecode: &[i64], entry: usize) -> Result<()> {
        let mut worklist = vec![entry];
        let mut abstract_states: HashMap<usize, Interval> = HashMap::new();
        abstract_states.insert(entry, Interval::top());

        while let Some(ip) = worklist.pop() {
            if ip + 2 >= bytecode.len() {
                continue;
            }

            let current = abstract_states[&ip].clone();
            let c = bytecode[ip + 2] as usize;

            // Next IP candidates
            let next_ips = vec![ip + 3, c];

            for next_ip in next_ips {
                if next_ip >= bytecode.len() {
                    continue;
                }

                let widened = if self.loop_headers.contains(&next_ip) {
                    current.widen(abstract_states.get(&next_ip).unwrap_or(&Interval::bottom()))
                } else {
                    current.clone()
                };

                if abstract_states.get(&next_ip) != Some(&widened) {
                    abstract_states.insert(next_ip, widened);
                    worklist.push(next_ip);
                }
            }
        }

        // Convert abstract states to predicates
        for (ip, state) in abstract_states {
            if let Some(range) = state.get(0) {
                self.extracted_predicates.push(Predicate::from_interval(ip, state));
            }
        }

        Ok(())
    }

    /// Match known SUBLEQ patterns
    fn match_patterns(&mut self, bytecode: &[i64]) -> Result<()> {
        for pattern in self.pattern_matcher.scan(bytecode) {
            match pattern {
                SubleqPattern::Clear { target, .. } => {
                    self.extracted_predicates.push(Predicate::Eq(
                        SymbolicValue::Mem(target),
                        SymbolicValue::Const(0),
                    ));
                }
                SubleqPattern::Loop { header, counter, .. } => {
                    self.extracted_predicates.push(Predicate::LoopInvariant {
                        header,
                        predicate: Box::new(Predicate::Le(
                            SymbolicValue::Const(0),
                            SymbolicValue::Mem(counter),
                        )),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn generate_predicates(&mut self) -> Result<()> {
        // Convert cached loop invariants
        for (header, preds) in self.invariant_cache.drain() {
            for p in preds {
                self.extracted_predicates.push(p);
            }
        }
        Ok(())
    }

    fn should_branch(&self, val: &SymbolicValue) -> bool {
        // If value is <= 0, branch
        matches!(val, SymbolicValue::Const(n) if *n <= 0)
    }
}

impl Default for InvariantExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_discovery() {
        let bytecode = vec![
            0, 1, 3,    // Forward
            1, 2, 0,    // Backward to 0
        ];

        let mut extractor = InvariantExtractor::new();
        extractor.discover_loops(&bytecode, 0).unwrap();
        assert!(extractor.loop_headers.contains(&0));
    }

    #[test]
    fn test_pattern_matching() {
        let bytecode = vec![
            5, 5, 3,    // Clear M[5]
            0, 0, 0,    // Halt
        ];

        let mut extractor = InvariantExtractor::new();
        extractor.match_patterns(&bytecode).unwrap();
        assert!(extractor.extracted_predicates.len() > 0);
    }
}
