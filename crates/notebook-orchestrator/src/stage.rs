// Execution stages for LOC triad pipeline

use serde::{Deserialize, Serialize};

/// Execution stage in the LOC triad pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Stage {
    Receive,
    Translate,
    Verify,
    Dispatch,
    Execute,
    Encode,
    Seal,
    Complete,
}

impl Stage {
    /// Get the next stage in the pipeline
    pub fn next(self) -> Option<Stage> {
        match self {
            Stage::Receive => Some(Stage::Translate),
            Stage::Translate => Some(Stage::Verify),
            Stage::Verify => Some(Stage::Dispatch),
            Stage::Dispatch => Some(Stage::Execute),
            Stage::Execute => Some(Stage::Encode),
            Stage::Encode => Some(Stage::Seal),
            Stage::Seal => Some(Stage::Complete),
            Stage::Complete => None,
        }
    }

    /// Get the agent responsible for this stage
    pub fn agent(self) -> &'static str {
        match self {
            Stage::Receive => "loc",
            Stage::Translate => "resonance",
            Stage::Verify => "sentinel",
            Stage::Dispatch => "loc",
            Stage::Execute => "forge",
            Stage::Encode => "resonance",
            Stage::Seal => "metatron",
            Stage::Complete => "metatron",
        }
    }

    /// Get the required capability for this stage
    pub fn required_capability(self) -> &'static str {
        match self {
            Stage::Receive => "dispatch",
            Stage::Translate => "compute",
            Stage::Verify => "audit",
            Stage::Dispatch => "dispatch",
            Stage::Execute => "execute",
            Stage::Encode => "synthesize",
            Stage::Seal => "worm_seal",
            Stage::Complete => "finalize",
        }
    }

    /// Get the target runtime for this stage
    pub fn target_runtime(self) -> &'static str {
        match self {
            Stage::Receive => "rust",
            Stage::Translate => "emoji",
            Stage::Verify => "ada",
            Stage::Dispatch => "rust",
            Stage::Execute => "holyc",
            Stage::Encode => "emoji",
            Stage::Seal => "haskell",
            Stage::Complete => "haskell",
        }
    }

    /// Determine if stage is in execution phase (vs metadata/control)
    pub fn is_execution(self) -> bool {
        matches!(self, Stage::Execute | Stage::Verify | Stage::Seal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_progression() {
        assert_eq!(Stage::Receive.next(), Some(Stage::Translate));
        assert_eq!(Stage::Translate.next(), Some(Stage::Verify));
        assert_eq!(Stage::Complete.next(), None);
    }

    #[test]
    fn test_agent_assignment() {
        assert_eq!(Stage::Receive.agent(), "loc");
        assert_eq!(Stage::Verify.agent(), "sentinel");
        assert_eq!(Stage::Execute.agent(), "forge");
        assert_eq!(Stage::Seal.agent(), "metatron");
    }

    #[test]
    fn test_runtime_targets() {
        assert_eq!(Stage::Receive.target_runtime(), "rust");
        assert_eq!(Stage::Execute.target_runtime(), "holyc");
        assert_eq!(Stage::Seal.target_runtime(), "haskell");
    }
}
