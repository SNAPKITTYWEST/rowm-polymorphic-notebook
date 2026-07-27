//! SUBLEQ-specific pattern recognition

use serde::{Deserialize, Serialize};

/// Common SUBLEQ patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubleqPattern {
    /// Clear M[target] to 0: SUBLEQ target, target, next
    Clear {
        target: usize,
        next_addr: usize,
    },
    /// Copy M[src] to M[dst]
    Copy {
        src: usize,
        dst: usize,
        end_addr: usize,
    },
    /// Add M[a] + M[b] → M[dst]
    Add {
        a: usize,
        b: usize,
        dst: usize,
        end_addr: usize,
    },
    /// Loop: while M[counter] > 0, do body
    Loop {
        header: usize,
        counter: usize,
        bound: usize,
        body_start: usize,
        body_end: usize,
    },
}

/// Pattern matcher for SUBLEQ bytecode
pub struct PatternMatcher;

impl PatternMatcher {
    pub fn new() -> Self {
        Self
    }

    /// Scan bytecode for known patterns
    pub fn scan(&self, bytecode: &[i64]) -> Vec<SubleqPattern> {
        let mut patterns = Vec::new();
        let mut i = 0;

        while i + 2 < bytecode.len() {
            let a = bytecode[i] as usize;
            let b = bytecode[i + 1] as usize;
            let c = bytecode[i + 2] as usize;

            // Pattern: SUBLEQ x, x, next (clear)
            if a == b {
                patterns.push(SubleqPattern::Clear {
                    target: a,
                    next_addr: i + 3,
                });
            }

            // Pattern: backward branch (loop)
            if c < i {
                patterns.push(SubleqPattern::Loop {
                    header: c,
                    counter: a,
                    bound: b,
                    body_start: c,
                    body_end: i,
                });
            }

            i += 3;
        }

        patterns
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matcher() {
        let bytecode = vec![
            0, 0, 3,    // Clear M[0]: SUBLEQ 0, 0, 3
            1, 2, 0,    // Loop back
        ];

        let matcher = PatternMatcher::new();
        let patterns = matcher.scan(&bytecode);
        assert!(patterns.len() > 0);
    }

    #[test]
    fn test_clear_pattern() {
        let bytecode = vec![5, 5, 3]; // SUBLEQ 5, 5, 3 (clear M[5])
        let matcher = PatternMatcher::new();
        let patterns = matcher.scan(&bytecode);
        assert!(patterns.iter().any(|p| matches!(p, SubleqPattern::Clear { .. })));
    }
}
