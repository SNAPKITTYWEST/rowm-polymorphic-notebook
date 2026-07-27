% Test Queries: Validation suite for Prolog logic system
% Run with: swipl -f test_queries.pl -t run_tests

:- use_module(authorization).
:- use_module(transitions).
:- use_module(proofs).
:- use_module(provenance).
:- use_module(release).
:- use_module(agents).
:- use_module(capabilities).
:- use_module(runtimes).
:- use_module(receipts).

% Test suite entry point
run_tests :-
    write('=== AUTHORIZATION TESTS ==='), nl,
    test_capability_active,
    test_agent_authorized,
    test_dispatch_permitted,
    nl, write('=== TRANSITION TESTS ==='), nl,
    test_transition_valid,
    test_state_reachable,
    nl, write('=== PROOF TESTS ==='), nl,
    test_proof_satisfied,
    nl, write('=== PROVENANCE TESTS ==='), nl,
    test_receipt_chain_valid,
    nl, write('=== RELEASE READINESS ==='), nl,
    test_release_ready,
    nl, write('=== ALL TESTS PASSED ==='), nl.

% Test: capability_active/2
test_capability_active :-
    write('Testing capability_active/2...'), nl,
    (capability_active('capa_001_loc_rust_exec', true)
        -> write('  PASS: capa_001_loc_rust_exec is active'), nl
        ; write('  FAIL: capa_001_loc_rust_exec should be active'), nl),
    (capability_active('capa_006_phantom_rust_log', false)
        -> write('  PASS: capa_006_phantom_rust_log is inactive (expired)'), nl
        ; write('  FAIL: capa_006_phantom_rust_log should be inactive'), nl).

% Test: agent_authorized/3
test_agent_authorized :-
    write('Testing agent_authorized/3...'), nl,
    (agent_authorized(loc, 'capa_001_loc_rust_exec', rust)
        -> write('  PASS: loc is authorized for capa_001 on rust'), nl
        ; write('  FAIL: loc should be authorized'), nl),
    (agent_authorized(phantom, 'capa_006_phantom_rust_log', rust)
        -> write('  FAIL: phantom (tier_2) should not be authorized'), nl
        ; write('  PASS: phantom (tier_2) is not authorized (correct)'), nl).

% Test: dispatch_permitted/5
test_dispatch_permitted :-
    write('Testing dispatch_permitted/5...'), nl,
    (dispatch_permitted(loc, 'capa_001_loc_rust_exec', rust, dispatch, true)
        -> write('  PASS: loc can dispatch on rust'), nl
        ; write('  FAIL: loc should be able to dispatch'), nl),
    (dispatch_permitted(loc, 'capa_001_loc_rust_exec', rust, invalid_perm, false)
        -> write('  PASS: dispatch rejected for invalid_perm'), nl
        ; write('  FAIL: dispatch should be rejected for invalid_perm'), nl).

% Test: transition_valid/3
test_transition_valid :-
    write('Testing transition_valid/3...'), nl,
    (transition_valid(receive, translate, true)
        -> write('  PASS: receive->translate is valid'), nl
        ; write('  FAIL: receive->translate should be valid'), nl),
    (transition_valid(receive, execute, false)
        -> write('  PASS: receive->execute is invalid (correct)'), nl
        ; write('  FAIL: receive->execute should be invalid'), nl).

% Test: state_reachable/3
test_state_reachable :-
    write('Testing state_reachable/3...'), nl,
    (state_reachable(dispatch, initial, true)
        -> write('  PASS: dispatch is reachable from initial'), nl
        ; write('  FAIL: dispatch should be reachable'), nl),
    (state_reachable(complete, initial, true)
        -> write('  PASS: complete is reachable from initial'), nl
        ; write('  FAIL: complete should be reachable'), nl).

% Test: proof_satisfied/2
test_proof_satisfied :-
    write('Testing proof_satisfied/2...'), nl,
    (proof_satisfied('proof_borrow_step_sound', true)
        -> write('  PASS: proof_borrow_step_sound is satisfied'), nl
        ; write('  FAIL: proof_borrow_step_sound should be satisfied'), nl),
    (proof_satisfied('nonexistent_proof', false)
        -> write('  PASS: nonexistent_proof is not satisfied (correct)'), nl
        ; write('  FAIL: nonexistent_proof should not be satisfied'), nl).

% Test: receipt_chain_valid/1
test_receipt_chain_valid :-
    write('Testing receipt_chain_valid/1...'), nl,
    (receipt_chain_valid(true)
        -> write('  PASS: receipt chain is valid'), nl
        ; write('  FAIL: receipt chain should be valid'), nl).

% Test: release_ready/1
test_release_ready :-
    write('Testing release_ready/1...'), nl,
    (release_ready(true)
        -> write('  PASS: system is release-ready'), nl
        ; write('  FAIL: system should be release-ready'), nl),
    write('  Readiness report:'), nl,
    readiness_report(Report),
    forall(member(check(Name, Result), Report),
        format('    ~w: ~w~n', [Name, Result])).
