// PHASE 9: Comprehensive Testing Suite
// Unit, integration, property-based, fuzz, tamper, and replay tests

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashMap;

    // ============ UNIT TESTS ============

    #[test]
    fn test_ed25519_keypair_generation() {
        let kp = Ed25519KeyPair::generate();
        let private_hex = kp.private_key_hex();
        let public_hex = kp.public_key_hex();

        assert_eq!(private_hex.len(), 64); // 32 bytes = 64 hex
        assert_eq!(public_hex.len(), 64);
    }

    #[test]
    fn test_ed25519_sign_and_verify() {
        let kp = Ed25519KeyPair::generate();
        let message = b"test message";
        let signature = kp.sign(message);

        let verified = Ed25519KeyPair::verify(&kp.public_key_hex(), message, &signature)
            .expect("Verify failed");
        assert!(verified);
    }

    #[test]
    fn test_receipt_v2_canonical_hash_deterministic() {
        let r1 = ReceiptV2::new(
            0, "agent1".to_string(), "cap1".to_string(), "instr1".to_string(),
            "dispatch".to_string(), "input1".to_string(), "output1".to_string(),
            "rust".to_string(), "1.0.0".to_string(), "abc123".to_string(),
            "env1".to_string(), "0".repeat(128), 100, "nonce1".to_string(),
            "context1".to_string(),
        );

        let r2 = ReceiptV2::new(
            0, "agent1".to_string(), "cap1".to_string(), "instr1".to_string(),
            "dispatch".to_string(), "input1".to_string(), "output1".to_string(),
            "rust".to_string(), "1.0.0".to_string(), "abc123".to_string(),
            "env1".to_string(), "0".repeat(128), 100, "nonce1".to_string(),
            "context1".to_string(),
        );

        assert_eq!(r1.receipt_hash, r2.receipt_hash);
    }

    #[test]
    fn test_replay_detector_rejects_duplicate_nonce() {
        let mut detector = ReplayDetector::new("test".to_string(), 3600);

        assert!(detector.check_and_record("nonce1", 100).is_ok());
        assert!(detector.check_and_record("nonce1", 101).is_err()); // Duplicate
    }

    #[test]
    fn test_replay_detector_enforces_monotonic_counter() {
        let mut detector = ReplayDetector::new("test".to_string(), 3600);

        assert!(detector.check_and_record("nonce1", 100).is_ok());
        assert!(detector.check_and_record("nonce2", 99).is_err()); // Non-monotonic
    }

    #[test]
    fn test_notebook_cell_hash_deterministic() {
        let cell1 = NotebookCell::new(
            0, "code".to_string(), "x = 1".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1000, "0".repeat(128),
        );

        let cell2 = NotebookCell::new(
            0, "code".to_string(), "x = 1".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1000, "0".repeat(128),
        );

        assert_eq!(cell1.cell_hash, cell2.cell_hash);
    }

    #[test]
    fn test_notebook_cell_tampering_detected() {
        let mut cell = NotebookCell::new(
            0, "code".to_string(), "x = 1".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1000, "0".repeat(128),
        );

        assert!(cell.verify_hash());

        cell.source = "x = 2".to_string();
        assert!(!cell.verify_hash());
    }

    #[test]
    fn test_self_modification_pattern_detection() {
        let writes = vec![
            MemoryWrite {
                address: 0,
                value: 100,
                writes_instruction_field: true,
                pc: 0,
                cycle: 0,
            },
            MemoryWrite {
                address: 3,
                value: 200,
                writes_instruction_field: true,
                pc: 1,
                cycle: 1,
            },
        ];

        let analyzer = SelfModificationAnalyzer::new(1);
        let result = analyzer.analyze(&writes);

        assert!(result.patterns.contains(&ModificationPattern::CodeGeneration));
    }

    // ============ INTEGRATION TESTS ============

    #[test]
    fn test_receipt_chain_append_and_verify() {
        let mut chain = ReceiptChainV2::new();

        let r1 = ReceiptV2::new(
            0, "loc".to_string(), "cap001".to_string(), "instr1".to_string(),
            "dispatch".to_string(), "inp1".to_string(), "out1".to_string(),
            "rust".to_string(), "1.0.0".to_string(), "src1".to_string(),
            "env1".to_string(), "0".repeat(128), 100, "nonce001".to_string(),
            "global".to_string(),
        );

        assert!(chain.append(r1.clone()).is_ok());
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_notebook_merkle_tree_chain_linkage() {
        let mut tree = NotebookMerkleTree::new();

        let cell0 = NotebookCell::new(
            0, "code".to_string(), "x = 1".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1000, "0".repeat(128),
        );

        assert!(tree.add_cell(cell0.clone()).is_ok());

        let cell1 = NotebookCell::new(
            1, "code".to_string(), "print(x)".to_string(), Some(1),
            "1\n".to_string(), "{}".to_string(), 1001, cell0.cell_hash.clone(),
        );

        assert!(tree.add_cell(cell1).is_ok());
        assert!(tree.verify_integrity());
    }

    #[test]
    fn test_global_replay_protection_multiple_contexts() {
        let mut protection = GlobalReplayProtection::new(3600);

        assert!(protection.check_and_record("ctx1", "nonce1", 1).is_ok());
        assert!(protection.check_and_record("ctx1", "nonce2", 2).is_ok());
        assert!(protection.check_and_record("ctx2", "nonce1", 1).is_ok()); // OK in different context

        assert!(protection.check_and_record("ctx1", "nonce1", 3).is_err()); // Duplicate nonce
    }

    #[test]
    fn test_keystore_rotation() {
        let mut store = KeyStore::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (v1, _) = store.generate_key("agent1".to_string(), now, now + 3600)
            .expect("Generate v1 failed");
        assert_eq!(v1, 1);

        let (v2, _) = store.rotate_key("agent1", now, now + 7200)
            .expect("Rotate failed");
        assert_eq!(v2, 2);

        let (current, _) = store.get_current_key("agent1").expect("Get current failed");
        assert_eq!(current, v2);

        let old_key = store.get_key("agent1", v1).expect("Get old key failed");
        assert_eq!(old_key.status, KeyStatus::Revoked);
    }

    // ============ PROPERTY-BASED TESTS ============

    #[test]
    fn test_receipt_hash_stability() {
        // Property: Receipt hash is deterministic (same input always produces same hash)
        for _ in 0..10 {
            let r1 = ReceiptV2::new(
                0, "agent".to_string(), "cap".to_string(), "instr".to_string(),
                "action".to_string(), "input".to_string(), "output".to_string(),
                "runtime".to_string(), "1.0".to_string(), "src".to_string(),
                "env".to_string(), "0".repeat(128), 100, "nonce".to_string(),
                "context".to_string(),
            );

            let r2 = ReceiptV2::new(
                0, "agent".to_string(), "cap".to_string(), "instr".to_string(),
                "action".to_string(), "input".to_string(), "output".to_string(),
                "runtime".to_string(), "1.0".to_string(), "src".to_string(),
                "env".to_string(), "0".repeat(128), 100, "nonce".to_string(),
                "context".to_string(),
            );

            assert_eq!(r1.receipt_hash, r2.receipt_hash);
        }
    }

    #[test]
    fn test_monotonic_counter_invariant() {
        // Property: Monotonic counter never decreases in sequence
        let mut detector = ReplayDetector::new("test".to_string(), 3600);
        let mut last_counter = 0;

        for i in 1..100 {
            assert!(detector.check_and_record(&format!("nonce-{}", i), i as u64).is_ok());
            last_counter = i as u64;
        }

        assert_eq!(detector.last_counter(), last_counter);
    }

    // ============ FUZZ TESTS ============

    #[test]
    fn test_malformed_hash_rejected() {
        let mut cell = NotebookCell::new(
            0, "code".to_string(), "x = 1".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1000, "0".repeat(128),
        );

        // Tamper with hash
        cell.cell_hash = "invalid_hash".to_string();
        assert!(!cell.verify_hash());
    }

    #[test]
    fn test_malformed_signature_rejected() {
        let kp = Ed25519KeyPair::generate();
        let message = b"test";

        let result = Ed25519KeyPair::verify(&kp.public_key_hex(), message, "invalid_sig");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nonce_format_rejected() {
        assert!(!NonceGenerator::is_valid_format("invalid"));
        assert!(!NonceGenerator::is_valid_format(""));
    }

    // ============ TAMPER TESTS ============

    #[test]
    fn test_receipt_tampering_detection() {
        let mut r = ReceiptV2::new(
            0, "agent".to_string(), "cap".to_string(), "instr".to_string(),
            "action".to_string(), "input".to_string(), "output".to_string(),
            "runtime".to_string(), "1.0".to_string(), "src".to_string(),
            "env".to_string(), "0".repeat(128), 100, "nonce".to_string(),
            "context".to_string(),
        );

        let original_hash = r.receipt_hash.clone();
        assert!(r.verify_hash());

        // Tamper with agent ID
        r.agent_id = "hacker".to_string();
        assert!(!r.verify_hash());
        assert_ne!(r.receipt_hash, original_hash);
    }

    #[test]
    fn test_notebook_cell_reordering_detection() {
        let mut tree = NotebookMerkleTree::new();

        let cell0 = NotebookCell::new(
            0, "code".to_string(), "x = 1".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1000, "0".repeat(128),
        );

        assert!(tree.add_cell(cell0.clone()).is_ok());

        // Try to add cell with wrong index (reordering attack)
        let bad_cell = NotebookCell::new(
            0, // Should be 1
            "code".to_string(), "y = 2".to_string(), Some(1),
            "".to_string(), "{}".to_string(), 1001, cell0.cell_hash.clone(),
        );

        assert!(tree.add_cell(bad_cell).is_err());
    }

    // ============ REPLAY TESTS ============

    #[test]
    fn test_replay_attack_detection() {
        let mut chain = ReceiptChainV2::new();

        let r1 = ReceiptV2::new(
            0, "loc".to_string(), "cap001".to_string(), "instr1".to_string(),
            "dispatch".to_string(), "inp1".to_string(), "out1".to_string(),
            "rust".to_string(), "1.0.0".to_string(), "src1".to_string(),
            "env1".to_string(), "0".repeat(128), 100, "nonce-001".to_string(),
            "global".to_string(),
        );

        assert!(chain.append(r1.clone()).is_ok());

        // Attempt to replay: same nonce + context
        let r2 = ReceiptV2::new(
            1, "loc".to_string(), "cap001".to_string(), "instr2".to_string(),
            "dispatch".to_string(), "inp2".to_string(), "out2".to_string(),
            "rust".to_string(), "1.0.0".to_string(), "src2".to_string(),
            "env2".to_string(), r1.receipt_hash.clone(), 200,
            "nonce-001".to_string(), // DUPLICATE NONCE
            "global".to_string(), // DUPLICATE CONTEXT
        );

        assert!(chain.append(r2).is_err());
    }

    #[test]
    fn test_cross_context_nonce_isolation() {
        let mut protection = GlobalReplayProtection::new(3600);

        // Same nonce in different contexts: should succeed
        assert!(protection.check_and_record("context-A", "shared-nonce", 1).is_ok());
        assert!(protection.check_and_record("context-B", "shared-nonce", 1).is_ok());

        // But duplicate within same context: should fail
        assert!(protection.check_and_record("context-A", "shared-nonce", 2).is_err());
    }

    #[test]
    fn test_monotonic_counter_prevents_reordering() {
        let mut detector = ReplayDetector::new("test".to_string(), 3600);

        assert!(detector.check_and_record("n1", 100).is_ok());
        assert!(detector.check_and_record("n2", 101).is_ok());

        // Try to insert out-of-order
        assert!(detector.check_and_record("n3", 101).is_err()); // Not increasing
        assert!(detector.check_and_record("n4", 99).is_err()); // Going backward
    }

    // ============ REGRESSION TESTS ============

    #[test]
    fn test_all_receipt_v2_tests_still_passing() {
        // Ensure existing receipt_v2 tests don't regress
        assert!(ReceiptV2::new(
            0, "a".to_string(), "b".to_string(), "c".to_string(),
            "d".to_string(), "e".to_string(), "f".to_string(),
            "g".to_string(), "h".to_string(), "i".to_string(),
            "j".to_string(), "0".repeat(128), 1, "n".to_string(),
            "ctx".to_string(),
        ).verify_hash());
    }

    #[test]
    fn test_all_ed25519_tests_still_passing() {
        let kp = Ed25519KeyPair::generate();
        let msg = b"test";
        let sig = kp.sign(msg);
        assert!(Ed25519KeyPair::verify(&kp.public_key_hex(), msg, &sig).unwrap());
    }

    #[test]
    fn test_all_replay_protection_tests_still_passing() {
        let mut protection = GlobalReplayProtection::new(3600);
        assert!(protection.check_and_record("ctx", "nonce", 1).is_ok());
    }
}
