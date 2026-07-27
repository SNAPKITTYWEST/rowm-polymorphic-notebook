//! Abstract interpretation domains

use std::collections::HashMap;

/// Abstract domain for numeric intervals
#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    ranges: HashMap<usize, (i64, i64)>, // addr -> (min, max)
}

impl Interval {
    pub fn new() -> Self {
        Self {
            ranges: HashMap::new(),
        }
    }

    /// Top element: all values possible
    pub fn top() -> Self {
        Self::new()
    }

    /// Bottom element: no values possible
    pub fn bottom() -> Self {
        Self::new() // Empty ranges
    }

    pub fn set(&mut self, addr: usize, range: (i64, i64)) {
        self.ranges.insert(addr, range);
    }

    pub fn get(&self, addr: usize) -> Option<(i64, i64)> {
        self.ranges.get(&addr).copied()
    }

    /// Add two intervals: [a1,b1] + [a2,b2] = [a1+a2, b1+b2]
    pub fn add(&self, a1: (i64, i64), a2: (i64, i64)) -> (i64, i64) {
        (a1.0.saturating_add(a2.0), a1.1.saturating_add(a2.1))
    }

    /// Subtract intervals: [a1,b1] - [a2,b2] = [a1-b2, b1-a2]
    pub fn sub(&self, a1: (i64, i64), a2: (i64, i64)) -> (i64, i64) {
        (a1.0.saturating_sub(a2.1), a1.1.saturating_sub(a2.0))
    }

    /// Widening: join two intervals with limit
    pub fn widen(&self, other: &Interval) -> Interval {
        let mut result = self.clone();

        for (addr, (min1, max1)) in &self.ranges {
            if let Some((min2, max2)) = other.ranges.get(addr) {
                let min = (*min1).min(*min2);
                let max = (*max1).max(*max2);
                result.set(*addr, (min, max));
            }
        }

        result
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::new()
    }
}

/// Abstract domain trait
pub trait AbstractDomain: Clone {
    /// Join two states
    fn join(&self, other: &Self) -> Self;

    /// Widen at loop headers
    fn widen_at(&self, addr: usize, state: &crate::symbolic::SymbolicState) -> crate::predicates::Predicate;

    /// Narrow inside loops
    fn narrow(&self, other: &Self) -> Self;
}

impl AbstractDomain for Interval {
    fn join(&self, other: &Interval) -> Self {
        self.widen(other)
    }

    fn widen_at(&self, _addr: usize, _state: &crate::symbolic::SymbolicState) -> crate::predicates::Predicate {
        // For now, return tautology
        crate::predicates::Predicate::True
    }

    fn narrow(&self, _other: &Interval) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_creation() {
        let iv = Interval::new();
        assert!(iv.get(0).is_none());
    }

    #[test]
    fn test_interval_set_get() {
        let mut iv = Interval::new();
        iv.set(0, (0, 10));
        assert_eq!(iv.get(0), Some((0, 10)));
    }

    #[test]
    fn test_interval_arithmetic() {
        let iv = Interval::new();
        let result = iv.add((1, 5), (2, 3));
        assert_eq!(result, (3, 8));
    }

    #[test]
    fn test_interval_widen() {
        let mut iv1 = Interval::new();
        iv1.set(0, (0, 10));

        let mut iv2 = Interval::new();
        iv2.set(0, (5, 20));

        let widened = iv1.widen(&iv2);
        assert_eq!(widened.get(0), Some((0, 20)));
    }
}
