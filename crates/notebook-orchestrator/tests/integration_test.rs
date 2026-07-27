// Integration tests for orchestrator

#[cfg(test)]
mod orchestrator_tests {
    use notebook_orchestrator::{Instruction, Orchestrator, Stage};
    use std::collections::HashMap;

    #[test]
    fn test_full_pipeline_execution() {
        let mut orch = Orchestrator::new();

        let mut args = HashMap::new();
        args.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let instr = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args,
            "capa_001".to_string(),
        );

        // Enqueue and execute
        assert!(orch.enqueue(instr).is_ok());
        assert!(orch.execute().is_ok());

        // Verify integrity
        assert!(orch.verify());

        // Verify receipt chain
        let chain = orch.receipt_chain();
        assert!(chain.is_sealed());
        assert!(chain.len() >= 8); // All stages

        // Verify chain head
        if let Some(head) = chain.head() {
            assert_eq!(head.status, "sealed");
        }
    }

    #[test]
    fn test_stage_progression() {
        assert_eq!(Stage::Receive.next(), Some(Stage::Translate));
        assert_eq!(Stage::Translate.next(), Some(Stage::Verify));
        assert_eq!(Stage::Verify.next(), Some(Stage::Dispatch));
        assert_eq!(Stage::Dispatch.next(), Some(Stage::Execute));
        assert_eq!(Stage::Execute.next(), Some(Stage::Encode));
        assert_eq!(Stage::Encode.next(), Some(Stage::Seal));
        assert_eq!(Stage::Seal.next(), Some(Stage::Complete));
        assert_eq!(Stage::Complete.next(), None);
    }

    #[test]
    fn test_receipt_chain_integrity() {
        let mut orch = Orchestrator::new();

        let mut args = HashMap::new();
        args.insert("test".to_string(), serde_json::json!("value"));

        let instr = Instruction::new(
            "🦀".to_string(),
            "rust".to_string(),
            "Build".to_string(),
            args,
            "capa_002".to_string(),
        );

        orch.enqueue(instr).unwrap();
        orch.execute().unwrap();

        let chain = orch.receipt_chain();

        // Verify all receipts in chain
        for receipt in chain.all() {
            assert!(receipt.verify(), "Receipt {} failed verification", receipt.receipt_id);
            assert_eq!(receipt.schema_version, "1.0");
        }
    }

    #[test]
    fn test_instruction_hash_determinism() {
        let mut args1 = HashMap::new();
        args1.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let instr1 = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args1,
            "capa_001".to_string(),
        );

        let mut args2 = HashMap::new();
        args2.insert("fn".to_string(), serde_json::json!("FreqAnchor1618"));

        let instr2 = Instruction::new(
            "⚡".to_string(),
            "holyc".to_string(),
            "Execute".to_string(),
            args2,
            "capa_001".to_string(),
        );

        // Same semantic instruction should have same hash
        assert_eq!(instr1.instruction_hash, instr2.instruction_hash);
    }

    #[test]
    fn test_multiple_instructions() {
        let mut orch = Orchestrator::new();

        for i in 0..3 {
            let mut args = HashMap::new();
            args.insert("index".to_string(), serde_json::json!(i));

            let instr = Instruction::new(
                "🦀".to_string(),
                "rust".to_string(),
                "Process".to_string(),
                args,
                format!("capa_{:03}", i),
            );

            assert!(orch.enqueue(instr).is_ok());
        }

        assert!(orch.execute().is_ok());
        assert!(orch.verify());
    }
}
