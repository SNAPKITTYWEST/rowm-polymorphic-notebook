//! Sandboxing configuration for M4

use serde::{Deserialize, Serialize};

/// M4 sandbox limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxLimits {
    pub max_expansion_depth: usize,
    pub max_output_size: usize,
    pub max_call_stack: usize,
    pub allowed_builtins: Vec<String>,
    pub denied_builtins: Vec<String>,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_expansion_depth: 100,
            max_output_size: 1_048_576, // 1MB
            max_call_stack: 1000,
            allowed_builtins: vec![
                "define".into(),
                "defn".into(),
                "ifdef".into(),
                "ifelse".into(),
                "eval".into(),
                "len".into(),
                "substr".into(),
                "translit".into(),
                "include".into(),
            ],
            denied_builtins: vec![
                "esyscmd".into(),
                "sysval".into(),
                "maketemp".into(),
                "mkstemp".into(),
                "changequote".into(),
                "changecom".into(),
            ],
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub limits: SandboxLimits,
    pub enable_include: bool,
    pub enable_eval: bool,
    pub timeout_ms: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            limits: SandboxLimits::default(),
            enable_include: false,
            enable_eval: true,
            timeout_ms: 5000,
        }
    }
}

impl SandboxConfig {
    pub fn permissive() -> Self {
        let mut config = Self::default();
        config.enable_include = true;
        config.timeout_ms = 30000;
        config
    }

    pub fn strict() -> Self {
        let mut config = Self::default();
        config.limits.max_expansion_depth = 50;
        config.limits.max_output_size = 262_144; // 256KB
        config.timeout_ms = 1000;
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_limits_default() {
        let limits = SandboxLimits::default();
        assert_eq!(limits.max_expansion_depth, 100);
        assert_eq!(limits.max_output_size, 1_048_576);
    }

    #[test]
    fn test_sandbox_config_permissive() {
        let config = SandboxConfig::permissive();
        assert!(config.enable_include);
        assert_eq!(config.timeout_ms, 30000);
    }

    #[test]
    fn test_sandbox_config_strict() {
        let config = SandboxConfig::strict();
        assert_eq!(config.limits.max_expansion_depth, 50);
        assert_eq!(config.limits.max_output_size, 262_144);
    }
}
