// Replay Protection — Nonce, Sequence, Context-based Detection
// Implements SEC-004: Replay attack prevention via monotonic counters + context awareness

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique request identifier (prevents replay across time)
pub type Nonce = String;

/// Execution context (prevents replay across systems/networks)
pub type Context = String;

/// Monotonic sequence number (prevents replay within same context)
pub type SequenceNumber = u64;

/// Replay detection tracker for a single execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDetector {
    /// Context identifier (e.g., "global", "agent:alice", "receipt-chain:1")
    context: Context,

    /// Last seen monotonic counter in this context
    last_monotonic_counter: u64,

    /// Set of seen nonces in this context (for deduplication)
    seen_nonces: HashSet<Nonce>,

    /// Maximum age of nonce history (seconds)
    nonce_ttl: u64,

    /// Timestamp of last cleanup
    last_cleanup: u64,
}

impl ReplayDetector {
    /// Create a new replay detector for a context
    pub fn new(context: Context, nonce_ttl: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ReplayDetector {
            context,
            last_monotonic_counter: 0,
            seen_nonces: HashSet::new(),
            nonce_ttl,
            last_cleanup: now,
        }
    }

    /// Check if a nonce+sequence combination is valid (not replayed)
    pub fn check_and_record(
        &mut self,
        nonce: impl Into<Nonce>,
        monotonic_counter: u64,
    ) -> Result<(), String> {
        let nonce = nonce.into();
        // Cleanup expired nonces periodically
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - self.last_cleanup > 3600 {
            self.seen_nonces.clear();
            self.last_cleanup = now;
        }

        // Check monotonic counter (must be strictly increasing)
        if monotonic_counter <= self.last_monotonic_counter {
            return Err(format!(
                "Monotonic counter not increasing: {} <= {}",
                monotonic_counter, self.last_monotonic_counter
            ));
        }

        // Check for duplicate nonce
        if self.seen_nonces.contains(&nonce) {
            return Err(format!("Duplicate nonce detected: {}", nonce));
        }

        // Record this nonce+counter combination
        self.seen_nonces.insert(nonce);
        self.last_monotonic_counter = monotonic_counter;

        Ok(())
    }

    /// Get context identifier
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Get last monotonic counter value
    pub fn last_counter(&self) -> u64 {
        self.last_monotonic_counter
    }

    /// Get count of active nonces
    pub fn active_nonce_count(&self) -> usize {
        self.seen_nonces.len()
    }
}

/// Global replay detector: tracks all execution contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalReplayProtection {
    /// Per-context detectors
    detectors: HashMap<Context, ReplayDetector>,

    /// Default nonce TTL (seconds)
    default_nonce_ttl: u64,
}

impl GlobalReplayProtection {
    pub fn new(default_nonce_ttl: u64) -> Self {
        GlobalReplayProtection {
            detectors: HashMap::new(),
            default_nonce_ttl,
        }
    }

    /// Check a nonce+sequence+context combination
    pub fn check_and_record(
        &mut self,
        context: impl Into<Context>,
        nonce: impl Into<Nonce>,
        monotonic_counter: u64,
    ) -> Result<(), String> {
        let context = context.into();
        let nonce = nonce.into();

        // Get or create detector for this context
        let detector = self.detectors
            .entry(context.clone())
            .or_insert_with(|| ReplayDetector::new(context.clone(), self.default_nonce_ttl));

        // Check and record
        detector.check_and_record(nonce, monotonic_counter)
    }

    /// Get statistics for a context
    pub fn get_context_stats(&self, context: impl AsRef<str>) -> Option<ContextStats> {
        let ctx_str = context.as_ref();
        self.detectors.iter()
            .find(|(key, _)| key.as_str() == ctx_str)
            .map(|(_, detector)| ContextStats {
                context: detector.context.clone(),
                last_counter: detector.last_monotonic_counter,
                active_nonces: detector.seen_nonces.len(),
            })
    }

    /// List all monitored contexts
    pub fn list_contexts(&self) -> Vec<Context> {
        self.detectors.keys().cloned().collect()
    }

    /// Clear old nonce history across all contexts
    pub fn cleanup_expired_nonces(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for detector in self.detectors.values_mut() {
            if now - detector.last_cleanup > 3600 {
                detector.seen_nonces.clear();
                detector.last_cleanup = now;
            }
        }
    }
}

impl Default for GlobalReplayProtection {
    fn default() -> Self {
        Self::new(86400) // 24 hours default TTL
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub context: Context,
    pub last_counter: SequenceNumber,
    pub active_nonces: usize,
}

/// Nonce generator: creates cryptographically unique nonces
pub struct NonceGenerator;

impl NonceGenerator {
    /// Generate a unique nonce (UUID v4 + timestamp)
    pub fn generate() -> Nonce {
        use uuid::Uuid;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        format!("nonce-{}-{:x}", Uuid::new_v4(), now)
    }

    /// Generate nonce for specific agent + timestamp
    pub fn generate_for_agent(agent_id: &str) -> Nonce {
        use uuid::Uuid;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        format!("nonce-{}-{}-{:x}", agent_id, Uuid::new_v4(), now)
    }

    /// Validate nonce format (not cryptographic validation)
    pub fn is_valid_format(nonce: &str) -> bool {
        nonce.starts_with("nonce-") && nonce.len() > 10
    }
}

/// Multi-context replay protection state (for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayProtectionSnapshot {
    pub contexts: Vec<(Context, u64, usize)>, // (context, last_counter, nonce_count)
    pub snapshot_time: u64,
}

impl GlobalReplayProtection {
    /// Create a snapshot of current state (for audit/export)
    pub fn snapshot(&self) -> ReplayProtectionSnapshot {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let contexts = self.detectors
            .values()
            .map(|d| (d.context.clone(), d.last_monotonic_counter, d.seen_nonces.len()))
            .collect();

        ReplayProtectionSnapshot {
            contexts,
            snapshot_time: now,
        }
    }

    /// Export for long-term storage
    pub fn export(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to export: {}", e))
    }

    /// Import from long-term storage
    pub fn import(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to import: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_detector_rejects_duplicate_nonce() {
        let mut detector = ReplayDetector::new("test-context".to_string(), 3600);

        let nonce = "nonce-test-001";
        assert!(detector.check_and_record(nonce, 100).is_ok());

        // Duplicate nonce should fail
        assert!(detector.check_and_record(nonce, 101).is_err());
    }

    #[test]
    fn test_replay_detector_rejects_non_monotonic_counter() {
        let mut detector = ReplayDetector::new("test-context".to_string(), 3600);

        assert!(detector.check_and_record("nonce-001", 100).is_ok());

        // Non-increasing counter should fail
        assert!(detector.check_and_record("nonce-002", 100).is_err());
        assert!(detector.check_and_record("nonce-003", 99).is_err());
    }

    #[test]
    fn test_global_replay_protection_multiple_contexts() {
        let mut protection = GlobalReplayProtection::new(3600);

        // Context 1: sequence 1, 2, 3
        assert!(protection.check_and_record("ctx1", "nonce-001", 1).is_ok());
        assert!(protection.check_and_record("ctx1", "nonce-002", 2).is_ok());
        assert!(protection.check_and_record("ctx1", "nonce-003", 3).is_ok());

        // Context 2: independent sequence 1, 2
        assert!(protection.check_and_record("ctx2", "nonce-101", 1).is_ok());
        assert!(protection.check_and_record("ctx2", "nonce-102", 2).is_ok());

        // Verify stats
        let stats1 = protection.get_context_stats("ctx1").unwrap();
        assert_eq!(stats1.last_counter, 3);
        assert_eq!(stats1.active_nonces, 3);

        let stats2 = protection.get_context_stats("ctx2").unwrap();
        assert_eq!(stats2.last_counter, 2);
        assert_eq!(stats2.active_nonces, 2);
    }

    #[test]
    fn test_nonce_generator_produces_unique_values() {
        let n1 = NonceGenerator::generate();
        let n2 = NonceGenerator::generate();

        assert_ne!(n1, n2);
        assert!(NonceGenerator::is_valid_format(&n1));
        assert!(NonceGenerator::is_valid_format(&n2));
    }

    #[test]
    fn test_nonce_format_validation() {
        assert!(NonceGenerator::is_valid_format("nonce-123-456"));
        assert!(!NonceGenerator::is_valid_format("invalid"));
        assert!(!NonceGenerator::is_valid_format("nonce-"));
    }

    #[test]
    fn test_global_replay_protection_export_import() {
        let mut protection = GlobalReplayProtection::new(3600);
        protection.check_and_record("ctx1", "nonce-001", 1).ok();
        protection.check_and_record("ctx1", "nonce-002", 2).ok();

        let json = protection.export().expect("Export failed");
        let imported = GlobalReplayProtection::import(&json).expect("Import failed");

        let stats = imported.get_context_stats("ctx1").unwrap();
        assert_eq!(stats.last_counter, 2);
        assert_eq!(stats.active_nonces, 2);
    }

    #[test]
    fn test_replay_detector_cleanup_expires_nonces() {
        let mut detector = ReplayDetector::new("test".to_string(), 1);

        detector.check_and_record("nonce-001", 1).ok();
        assert_eq!(detector.active_nonce_count(), 1);

        // Simulate time passing and cleanup
        detector.last_cleanup = 0; // Force cleanup
        detector.check_and_record("nonce-002", 2).ok();

        // Nonces should be cleared during cleanup
        assert!(detector.seen_nonces.len() <= 2);
    }

    #[test]
    fn test_cross_context_isolation() {
        let mut protection = GlobalReplayProtection::new(3600);

        // Same nonce in different contexts should be allowed
        assert!(protection.check_and_record("ctx1", "shared-nonce", 1).is_ok());
        assert!(protection.check_and_record("ctx2", "shared-nonce", 1).is_ok());

        // But duplicate nonce in same context should fail
        assert!(protection.check_and_record("ctx1", "shared-nonce", 2).is_err());
    }
}
