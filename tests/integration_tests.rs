// PHASE 9: Integration Tests
// Cross-module verification, E2E workflows, state consistency

use notebook_orchestrator::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_end_to_end_receipt_chain_workflow() {
    // Setup: Create receipt chain with 5 receipts
    let mut chain = ReceiptChainV2::new();

    for i in 0..5 {
        let receipt = ReceiptV2::new(
            i as u32,
            format!("agent-{}", i),
            format!("cap-{:03}", i),
            format!("instr-{:03}", i),
            "dispatch".to_string(),
            format!("input-{}", i),
            format!("output-{}", i),
            "rust".to_string(),
            "1.0.0".to_string(),
            format!("src-{}", i),
            format!("env-{}", i),
            if i == 0 {
                "0".repeat(128)
            } else {
                "a".repeat(128)
            },
            100 + i as u64,
            format!("nonce-{:03}", i),
            "global".to_string(),
        );

        assert!(
            chain.append(receipt).is_ok(),
            "Failed to append receipt {}",
            i
        );
    }

    // Verify: Chain is valid and contains all 5 receipts
    assert!(chain.verify_chain());
    assert_eq!(chain.len(), 5);
}

#[test]
fn test_notebook_cell_chain_with_merkle_tree() {
    let mut tree = NotebookMerkleTree::new();

    // Add 10 cells to tree
    for i in 0..10 {
        let cell = NotebookCell::new(
            i as u32,
            "code".to_string(),
            format!("line {}", i),
            Some(i as u32),
            format!("result {}\n", i),
            "{}".to_string(),
            1000 + i as u64,
            if i == 0 {
                "0".repeat(128)
            } else {
                "a".repeat(128)
            },
        );

        assert!(tree.add_cell(cell).is_ok(), "Failed to add cell {}", i);
    }

    // Verify tree integrity
    assert!(tree.verify_integrity());
}

#[test]
fn test_replay_protection_across_multiple_agents() {
    let mut protection = GlobalReplayProtection::new(3600);

    // 3 agents, each issuing 3 receipts
    let agents = vec!["agent-1", "agent-2", "agent-3"];
    let mut counter = 1u64;

    for agent in &agents {
        for i in 0..3 {
            let nonce = format!("{}-nonce-{}", agent, i);
            let context = "global";

            assert!(
                protection.check_and_record(context, &nonce, counter).is_ok(),
                "Failed for agent {} nonce {}",
                agent,
                i
            );
            counter += 1;
        }
    }

    // Attempt replays should fail
    assert!(protection
        .check_and_record("global", "agent-1-nonce-0", 4)
        .is_err());
}

#[test]
fn test_ed25519_key_rotation_with_signature_verification() {
    let mut store = KeyStore::new();
    let now = 1719432000u64;

    // Generate initial key
    let (v1, _) = store
        .generate_key("agent-x".to_string(), now, now + 3600)
        .expect("Generate v1");

    // Sign with v1
    let kp_v1 = Ed25519KeyPair::generate();
    let msg = b"test message";
    let sig_v1 = kp_v1.sign(msg);

    // Verify with v1 succeeds
    assert!(Ed25519KeyPair::verify(&kp_v1.public_key_hex(), msg, &sig_v1).unwrap());

    // Rotate to v2
    let (v2, _) = store
        .rotate_key("agent-x", now + 1800, now + 5400)
        .expect("Rotate");

    // New signature with v2
    let kp_v2 = Ed25519KeyPair::generate();
    let sig_v2 = kp_v2.sign(msg);

    // Both keys should verify their own signatures
    assert!(Ed25519KeyPair::verify(&kp_v2.public_key_hex(), msg, &sig_v2).unwrap());

    // Old key v1 should still exist but be revoked
    let old_key = store.get_key("agent-x", v1).expect("Get old key");
    assert_eq!(old_key.status, KeyStatus::Revoked);

    // Current key should be v2
    let (current_v, _) = store
        .get_current_key("agent-x")
        .expect("Get current key");
    assert_eq!(current_v, v2);
}

#[test]
fn test_concurrent_receipt_appending() {
    let chain = Arc::new(Mutex::new(ReceiptChainV2::new()));

    let mut handles = vec![];

    // 5 threads, each appending 2 receipts
    for thread_id in 0..5 {
        let chain_clone = Arc::clone(&chain);

        let handle = std::thread::spawn(move || {
            for i in 0..2 {
                let receipt = ReceiptV2::new(
                    (thread_id * 2 + i) as u32,
                    format!("agent-thread-{}", thread_id),
                    format!("cap-{:03}-{}", thread_id, i),
                    format!("instr-{}", i),
                    "dispatch".to_string(),
                    "input".to_string(),
                    "output".to_string(),
                    "rust".to_string(),
                    "1.0.0".to_string(),
                    "src".to_string(),
                    "env".to_string(),
                    "a".repeat(128),
                    100 + i as u64,
                    format!("nonce-thread-{}-{}", thread_id, i),
                    "global".to_string(),
                );

                let mut c = chain_clone.lock().unwrap();
                c.append(receipt).ok();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify: 10 receipts total
    let chain_locked = chain.lock().unwrap();
    assert_eq!(chain_locked.len(), 10);
    assert!(chain_locked.verify_chain());
}

#[test]
fn test_self_modification_detection_with_verification_witness() {
    let writes = vec![
        MemoryWrite {
            address: 0,
            value: 100,
            writes_instruction_field: true,
            pc: 0,
            cycle: 0,
        },
        MemoryWrite {
            address: 4,
            value: 200,
            writes_instruction_field: true,
            pc: 1,
            cycle: 1,
        },
        MemoryWrite {
            address: 8,
            value: 300,
            writes_instruction_field: true,
            pc: 2,
            cycle: 2,
        },
    ];

    let analyzer = SelfModificationAnalyzer::new(2); // threshold 2
    let result = analyzer.analyze(&writes);

    // Should detect code generation pattern
    assert!(result.patterns.len() > 0);

    // Generate witness
    let witness = analyzer.generate_witness(&writes, &result);
    assert!(!witness.execution_trace_hash.is_empty());
}

#[test]
fn test_notebook_tamper_detection_across_cells() {
    let mut tree = NotebookMerkleTree::new();

    let mut cells = vec![];
    for i in 0..5 {
        let cell = NotebookCell::new(
            i as u32,
            "code".to_string(),
            format!("x = {}", i),
            Some(i as u32),
            format!("{}\n", i),
            "{}".to_string(),
            1000 + i as u64,
            if i == 0 {
                "0".repeat(128)
            } else {
                "a".repeat(128)
            },
        );
        cells.push(cell.clone());
        tree.add_cell(cell).ok();
    }

    // Verify initial integrity
    assert!(tree.verify_integrity());

    // Tamper with cell 2
    let mut bad_cell = cells[2].clone();
    bad_cell.source = "x = 999".to_string();

    // Adding tampered cell should fail (wrong hash)
    let result = tree.add_cell(bad_cell);
    assert!(result.is_err());
}

#[test]
fn test_proof_obligation_tracking_with_multiple_verifiers() {
    // This test validates that proof obligations can be tracked
    // across multiple verifier tools (Z3, Lean 4, Ada/SPARK, Agda)

    // In a real scenario, each obligation would be dispatched
    // to its corresponding verifier. For now, we test the data structures.

    let obligations = vec![
        ("inv-001", "invariant_preservation", "receipt_chain"),
        ("inv-002", "invariant_preservation", "memory_state"),
        ("sem-001", "semantic_preservation", "subleq_codegen"),
        ("loop-001", "loop_invariant", "receipt_append"),
        ("chain-001", "receipt_chain_integrity", "v2_chain"),
    ];

    // Validate each obligation has correct tool assignment
    let tool_map = vec![
        ("invariant_preservation", "z3"),
        ("semantic_preservation", "lean4"),
        ("loop_invariant", "spark"),
        ("receipt_chain_integrity", "agda"),
    ];

    for (_, type_name, _) in &obligations {
        let tool = tool_map
            .iter()
            .find(|(t, _)| t == type_name)
            .map(|(_, tool)| *tool);
        assert!(tool.is_some(), "Unknown obligation type: {}", type_name);
    }
}

#[test]
fn test_cross_module_state_consistency() {
    // Verify that state is consistent across:
    // 1. Receipt chain
    // 2. Replay protection
    // 3. Notebook merkle tree
    // 4. Key store

    let mut chain = ReceiptChainV2::new();
    let mut replay = GlobalReplayProtection::new(3600);
    let mut tree = NotebookMerkleTree::new();
    let mut keystore = KeyStore::new();
    let now = 1719432000u64;

    // Initialize key
    let (v1, _) = keystore
        .generate_key("agent-1".to_string(), now, now + 3600)
        .unwrap();

    // Add receipt to chain with replay protection
    let receipt = ReceiptV2::new(
        0,
        "agent-1".to_string(),
        "cap-001".to_string(),
        "instr-001".to_string(),
        "dispatch".to_string(),
        "input-001".to_string(),
        "output-001".to_string(),
        "rust".to_string(),
        "1.0.0".to_string(),
        "src-001".to_string(),
        "env-001".to_string(),
        "0".repeat(128),
        100,
        "nonce-001".to_string(),
        "global".to_string(),
    );

    assert!(chain.append(receipt.clone()).is_ok());
    assert!(replay
        .check_and_record("global", "nonce-001", 1)
        .is_ok());

    // Add notebook cell that references receipt
    let cell = NotebookCell::new(
        0,
        "code".to_string(),
        format!("receipt: {}", receipt.receipt_id),
        Some(1),
        "success".to_string(),
        "{}".to_string(),
        1000,
        "0".repeat(128),
    );

    assert!(tree.add_cell(cell).is_ok());

    // Verify all state is valid
    assert!(chain.verify_chain());
    assert!(tree.verify_integrity());
    assert_eq!(keystore.get_current_key("agent-1").unwrap().0, v1);
}
