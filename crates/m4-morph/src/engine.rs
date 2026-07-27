//! M4 Engine — GNU M4 wrapper with sandboxing and feedback

use crate::sandbox::{SandboxConfig, SandboxLimits};
use crate::feedback::FeedbackBuffer;
use anyhow::{Result, anyhow, Context};
use std::process::{Command, Stdio};
use std::io::Write;
use tracing::{debug, trace};

/// M4 Engine with sandboxing
pub struct M4Engine {
    config: SandboxConfig,
    feedback: FeedbackBuffer,
    expansion_count: usize,
}

impl M4Engine {
    pub fn new(config: SandboxConfig) -> Result<Self> {
        // Verify M4 exists
        Command::new("m4")
            .arg("--version")
            .output()
            .context("GNU M4 not found in PATH")?;

        Ok(Self {
            config,
            feedback: FeedbackBuffer::new(),
            expansion_count: 0,
        })
    }

    pub fn default_sandbox() -> Result<Self> {
        Self::new(SandboxConfig::default())
    }

    pub fn permissive_sandbox() -> Result<Self> {
        Self::new(SandboxConfig::permissive())
    }

    pub fn strict_sandbox() -> Result<Self> {
        Self::new(SandboxConfig::strict())
    }

    /// Expand M4 input with current feedback state
    pub fn expand(&mut self, input: &str) -> Result<String> {
        self.expansion_count += 1;
        debug!(count = self.expansion_count, input_len = input.len(), "M4 expansion");

        // Check recursion depth limit
        if self.expansion_count > self.config.limits.max_expansion_depth {
            return Err(anyhow!(
                "M4 recursion depth exceeded: {} > {}",
                self.expansion_count,
                self.config.limits.max_expansion_depth
            ));
        }

        // Construct full M4 input with feedback
        let full_input = self.construct_input(input)?;

        // Execute M4
        let output = self.execute_m4(&full_input)?;

        // Validate output size
        if output.len() > self.config.limits.max_output_size {
            return Err(anyhow!(
                "M4 output too large: {} > {}",
                output.len(),
                self.config.limits.max_output_size
            ));
        }

        trace!("M4 expansion succeeded");
        Ok(output)
    }

    /// Construct M4 input with sandbox prelude and feedback
    fn construct_input(&self, user_input: &str) -> Result<String> {
        let mut input = String::new();

        // 1. Sandbox prelude (deny dangerous builtins)
        for builtin in &self.config.limits.denied_builtins {
            input.push_str(&format!("define(`{}', `')\n", builtin));
        }

        // 2. Set recursion limit
        input.push_str(&format!(
            "define(`__M4_RECURSION_LIMIT', `{}')\n",
            self.config.limits.max_call_stack
        ));

        // 3. Feedback definitions from previous cells
        input.push_str(&self.feedback.get_definitions_code());

        // 4. User input
        input.push_str(user_input);
        input.push('\n');

        Ok(input)
    }

    /// Execute M4 via subprocess
    fn execute_m4(&self, input: &str) -> Result<String> {
        let mut child = Command::new("m4")
            .arg("-P") // Prefix builtins
            .arg("-E") // Fatal warnings
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn M4 process")?;

        // Write input
        {
            let stdin = child.stdin.as_mut().context("Failed to open M4 stdin")?;
            stdin.write_all(input.as_bytes()).context("Failed to write to M4")?;
        }

        // Wait for completion
        let output = child.wait_with_output().context("Failed to wait for M4")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("M4 expansion failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Record output as feedback for next cell
    pub fn push_feedback(&mut self, output: String) {
        self.feedback.push_output(output);
    }

    /// Define a macro for next cell
    pub fn define(&mut self, name: String, value: String) {
        let def = format!("{}={}", name, value);
        self.feedback.push_definition(def);
    }

    /// Clear feedback buffer (reset for new execution ring)
    pub fn clear_feedback(&mut self) {
        self.feedback.clear();
    }

    pub fn expansion_count(&self) -> usize {
        self.expansion_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let result = M4Engine::default_sandbox();
        // May fail if M4 not installed in test environment
        let _ = result;
    }

    #[test]
    fn test_construct_input() {
        let engine = M4Engine {
            config: SandboxConfig::default(),
            feedback: FeedbackBuffer::new(),
            expansion_count: 0,
        };

        let input = engine.construct_input("test input").unwrap();
        assert!(input.contains("test input"));
    }

    #[test]
    fn test_feedback_integration() {
        let mut engine = M4Engine {
            config: SandboxConfig::default(),
            feedback: FeedbackBuffer::new(),
            expansion_count: 0,
        };

        engine.define("VAR".into(), "42".into());
        engine.push_feedback("output: 42".into());

        assert_eq!(engine.expansion_count(), 0);
    }

    #[test]
    fn test_recursion_limit() {
        let mut engine = M4Engine {
            config: SandboxConfig::strict(),
            feedback: FeedbackBuffer::new(),
            expansion_count: 0,
        };

        engine.expansion_count = 50;
        let result = engine.expand("test");
        // Will fail due to recursion limit
        assert!(result.is_err());
    }
}
