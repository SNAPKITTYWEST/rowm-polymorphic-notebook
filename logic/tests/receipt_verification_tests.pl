% PHASE 9: Prolog Verification Tests
% Unit and integration tests for receipt verification and proof obligations

:- use_module(library(plunit)).

:- begin_tests(receipt_verification).

% Test 1: Canonical hash verification
test(receipt_hash_canonical) :-
    receipt_v2(1, 'rcpt-v2-0000000001-loc',
        'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
        'loc', _, _, _, _, _, _, _, 'success'),
    verify_receipt_hash(
        'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
        'rcpt-v2-0000000001-loc'
    ).

% Test 2: Ed25519 signature verification
test(ed25519_signature_verification) :-
    receipt_v2(1, 'rcpt-v2-0000000001-loc', _, 'loc', _, _, _, _, _, 1, 'sig-001-ed25519', _),
    verify_receipt_signature('rcpt-v2-0000000001-loc', 'sig-001-ed25519', 1).

% Test 3: Chain linkage verification (first receipt)
test(chain_linkage_first_receipt) :-
    receipt_chain_link('a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
                       '0000000000000000000000000000000000000000000000000000000000000000'),
    verify_chain_linkage('a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
                         '0000000000000000000000000000000000000000000000000000000000000000').

% Test 4: Chain linkage verification (subsequent receipts)
test(chain_linkage_subsequent) :-
    receipt_chain_link('b8d5c0f3e2g4d9b6f7c3d0e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4',
                       'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3'),
    verify_chain_linkage('b8d5c0f3e2g4d9b6f7c3d0e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4',
                         'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3').

% Test 5: Replay protection - first nonce succeeds
test(replay_protection_first) :-
    nonce_record('nonce-loc-001', 'global', 1, _),
    verify_replay_protection('nonce-loc-001', 'global', 2).

% Test 6: Replay protection - increasing counter
test(replay_protection_monotonic) :-
    nonce_record('nonce-loc-001', 'global', 1, _),
    verify_replay_protection('nonce-loc-001', 'global', 3).

% Test 7: Full receipt verification
test(full_receipt_verification) :-
    verify_receipt_complete('rcpt-v2-0000000001-loc', 'loc', 1, 'nonce-loc-001', 'global', 2).

% Test 8: Release gate - all receipts must be verified
test(release_gate_receipts) :-
    release_ready_receipts.

% Test 9: Proof obligation assignment
test(proof_obligation_z3_assignment) :-
    proof_obligation('inv-001', 'invariant_preservation', 'receipt_chain'),
    proof_tool_assignment('invariant_preservation', 'z3').

% Test 10: Proof obligation assignment - Lean 4
test(proof_obligation_lean4_assignment) :-
    proof_obligation('sem-001', 'semantic_preservation', 'subleq_codegen'),
    proof_tool_assignment('semantic_preservation', 'lean4').

% Test 11: Proof obligation assignment - Ada/SPARK
test(proof_obligation_spark_assignment) :-
    proof_obligation('loop-001', 'loop_invariant', 'receipt_append'),
    proof_tool_assignment('loop_invariant', 'spark').

% Test 12: Proof obligation assignment - Agda
test(proof_obligation_agda_assignment) :-
    proof_obligation('chain-001', 'receipt_chain_integrity', 'v2_chain'),
    proof_tool_assignment('receipt_chain_integrity', 'agda').

% Test 13: All proof obligations assigned
test(all_obligations_assigned) :-
    findall(Type, proof_obligation(_, Type, _), Types),
    list_to_set(Types, UniqueTypes),
    forall(
        member(Type, UniqueTypes),
        proof_tool_assignment(Type, _)
    ).

% Test 14: All obligations discharged (release gate)
test(all_obligations_discharged) :-
    all_obligations_discharged.

% Test 15: Receipt status validity
test(receipt_status_success) :-
    receipt_v2(_, 'rcpt-v2-0000000001-loc', _, _, _, _, _, _, _, _, _, 'success'),
    receipt_status_valid('rcpt-v2-0000000001-loc', true).

% Test 16: Canonical form generation
test(receipt_canonical_form) :-
    verify_receipt_canonical('rcpt-v2-0000000001-loc', Canonical),
    atom(Canonical),
    atom_length(Canonical, Length),
    Length > 0.

% Test 17: Receipt chain statistics
test(receipt_chain_stats) :-
    receipt_chain_stats(Total, Signed, Verified),
    Total > 0,
    Signed > 0,
    Verified > 0.

% Test 18: Nonce context isolation
test(nonce_context_isolation) :-
    nonce_record('nonce-loc-001', 'global', 1, _),
    % Different context would allow same nonce with different counter
    \+ nonce_record('nonce-loc-001', 'different-context', 1, _).

% Test 19: Public key exists for all agents
test(public_keys_for_agents) :-
    findall(Agent, receipt_v2(_, _, _, Agent, _, _, _, _, _, _, _, _), Agents),
    list_to_set(Agents, UniqueAgents),
    forall(
        member(Agent, UniqueAgents),
        ed25519_public_key(Agent, _, _)
    ).

% Test 20: Proof obligation type coverage
test(proof_obligation_type_coverage) :-
    findall(Type, proof_tool_assignment(Type, _), ToolTypes),
    findall(Type, proof_obligation(_, Type, _), ObligationTypes),
    list_to_set(ToolTypes, UniqueToolTypes),
    list_to_set(ObligationTypes, UniqueObligationTypes),
    UniqueToolTypes = UniqueObligationTypes.

:- end_tests(receipt_verification).

% Run all tests
run_tests :-
    run_tests(receipt_verification).
