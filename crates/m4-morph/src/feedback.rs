//! State Feedback Loop — Cell N output → Cell N+1 definitions

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Feedback buffer for inter-cell state propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackBuffer {
    definitions: VecDeque<String>,
    outputs: VecDeque<String>,
}

impl FeedbackBuffer {
    pub fn new() -> Self {
        Self {
            definitions: VecDeque::new(),
            outputs: VecDeque::new(),
        }
    }

    /// Push a definition for next cell (from previous cell output)
    pub fn push_definition(&mut self, def: String) {
        self.definitions.push_back(def);
        // Limit history to 50 definitions
        while self.definitions.len() > 50 {
            self.definitions.pop_front();
        }
    }

    /// Push output from current cell
    pub fn push_output(&mut self, output: String) {
        self.outputs.push_back(output);
        // Limit history to 100 outputs
        while self.outputs.len() > 100 {
            self.outputs.pop_front();
        }
    }

    /// Get all pending definitions as M4 code
    pub fn get_definitions_code(&self) -> String {
        self.definitions
            .iter()
            .map(|d| format!("define(`{}', `{}')\n", d.split('=').next().unwrap_or("_"), d))
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get last N outputs
    pub fn get_recent_outputs(&self, n: usize) -> Vec<String> {
        self.outputs
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect()
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.outputs.clear();
    }

    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

impl Default for FeedbackBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_buffer_creation() {
        let buf = FeedbackBuffer::new();
        assert_eq!(buf.definition_count(), 0);
        assert_eq!(buf.output_count(), 0);
    }

    #[test]
    fn test_push_definition() {
        let mut buf = FeedbackBuffer::new();
        buf.push_definition("CELL_1_RESULT=100".into());
        assert_eq!(buf.definition_count(), 1);
    }

    #[test]
    fn test_definitions_code() {
        let mut buf = FeedbackBuffer::new();
        buf.push_definition("VAR=42".into());
        let code = buf.get_definitions_code();
        assert!(code.contains("VAR"));
    }

    #[test]
    fn test_recent_outputs() {
        let mut buf = FeedbackBuffer::new();
        buf.push_output("output1".into());
        buf.push_output("output2".into());
        buf.push_output("output3".into());

        let recent = buf.get_recent_outputs(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], "output3");
    }

    #[test]
    fn test_buffer_limits() {
        let mut buf = FeedbackBuffer::new();
        for i in 0..60 {
            buf.push_definition(format!("VAR{}={}", i, i * 10));
        }
        // Should keep only last 50
        assert_eq!(buf.definition_count(), 50);
    }
}
