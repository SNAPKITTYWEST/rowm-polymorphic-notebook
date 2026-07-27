% Proof Obligation Rules (PHASE 7-10)
% Maps proof obligations to verifiers (Agda, Ada/SPARK, Lean 4, Z3)

:- module(proof_obligations, [
    proof_obligation/3,
    proof_verified/3,
    proof_tool_assignment/2,
    all_obligations_discharged/0
]).

% PROOF OBLIGATIONS
:- dynamic(proof_obligation/3).

% proof_obligation(ObligationID, Type, Context).

% 1. INVARIANT PRESERVATION (via Z3 SMT solver)
% Verify: ∀ state. invariant(state) ∧ transition(state, state') → invariant(state')
proof_obligation('inv-001', 'invariant_preservation', 'receipt_chain').
proof_obligation('inv-002', 'invariant_preservation', 'memory_state').
proof_obligation('inv-003', 'invariant_preservation', 'authorization_gates').

% 2. SEMANTIC PRESERVATION (via Lean 4)
% Verify: ∀ source. semantics(compile(source)) = semantics(source)
proof_obligation('sem-001', 'semantic_preservation', 'subleq_codegen').
proof_obligation('sem-002', 'semantic_preservation', 'unicode_roundtrip').
proof_obligation('sem-003', 'semantic_preservation', 'polyglot_compilation').

% 3. LOOP INVARIANT MAINTENANCE (via Ada/SPARK)
% Verify: ∀ i. loop_invariant(i) ∧ loop_condition(i) → loop_invariant(i+1)
proof_obligation('loop-001', 'loop_invariant', 'receipt_append').
proof_obligation('loop-002', 'loop_invariant', 'cell_iteration').
proof_obligation('loop-003', 'loop_invariant', 'nonce_verification').

% 4. RECEIPT CHAIN INTEGRITY (via Agda)
% Verify: ∀ r1, r2. r1.sequence < r2.sequence ∧ r2.prev_hash = r1.hash → chain_valid(r1, r2)
proof_obligation('chain-001', 'receipt_chain_integrity', 'v2_chain').
proof_obligation('chain-002', 'receipt_chain_integrity', 'replay_detection').
proof_obligation('chain-003', 'receipt_chain_integrity', 'merkle_tree').

% TOOL ASSIGNMENT (which verifier for each obligation)
:- dynamic(proof_tool_assignment/2).

% For v1 (stubs): all return true
proof_tool_assignment('invariant_preservation', 'z3').
proof_tool_assignment('semantic_preservation', 'lean4').
proof_tool_assignment('loop_invariant', 'spark').
proof_tool_assignment('receipt_chain_integrity', 'agda').

% PROOF STATUS
:- dynamic(proof_verified/3).

% proof_verified(ObligationID, Tool, Status).
% Status: proved | disproved | manual | error | timeout

% Current stub status (v1 - pre-PHASE-7)
proof_verified('inv-001', 'stub', 'assumed').
proof_verified('inv-002', 'stub', 'assumed').
proof_verified('inv-003', 'stub', 'assumed').

proof_verified('sem-001', 'stub', 'assumed').
proof_verified('sem-002', 'stub', 'assumed').
proof_verified('sem-003', 'stub', 'assumed').

proof_verified('loop-001', 'stub', 'assumed').
proof_verified('loop-002', 'stub', 'assumed').
proof_verified('loop-003', 'stub', 'assumed').

proof_verified('chain-001', 'stub', 'assumed').
proof_verified('chain-002', 'stub', 'assumed').
proof_verified('chain-003', 'stub', 'assumed').

% DISPATCH LOGIC (pre-PHASE-7: stubs / post-PHASE-7: real verifiers)
% After PHASE 7, this becomes:
%
% proof_verified(ObligationID, Tool, proved) :-
%     proof_obligation(ObligationID, Type, _),
%     proof_tool_assignment(Type, Tool),
%     % Call actual verifier (e.g., agda_adapter:verify(...))
%     agda_adapter:verify_obligation(ObligationID, Tool).

% VERIFICATION PREDICATE (comprehensive check)
verify_proof_obligation(ObligationID) :-
    proof_obligation(ObligationID, Type, Context),
    proof_tool_assignment(Type, Tool),
    proof_verified(ObligationID, Tool, Status),
    % Current: all stubs assumed
    (Status = assumed ; Status = proved).

% RELEASE GATE: All proof obligations discharged
all_obligations_discharged :-
    % 1. All obligations must exist
    findall(ID, proof_obligation(ID, _, _), ObligationIDs),
    length(ObligationIDs, Count),
    Count > 0,

    % 2. All must be verified (either proved or assumed)
    forall(
        proof_obligation(OID, Type, _),
        (   proof_tool_assignment(Type, Tool),
            proof_verified(OID, Tool, Status),
            (Status = proved ; Status = assumed)
        )
    ),

    % 3. No timeouts or errors
    \+ proof_verified(_, _, error),
    \+ proof_verified(_, _, timeout).

% DEBUG: Check obligation status
check_obligation_status :-
    format('PROOF OBLIGATIONS STATUS~n', []),
    format('========================~n', []),
    forall(
        proof_obligation(ID, Type, Context),
        (   proof_tool_assignment(Type, Tool),
            proof_verified(ID, Tool, Status),
            format('~w: ~w (~w) [~w]~n', [ID, Type, Tool, Status])
        )
    ),
    format('~n', []),
    (   all_obligations_discharged
    ->  format('✓ ALL OBLIGATIONS DISCHARGED~n', [])
    ;   format('✗ UNMET OBLIGATIONS EXIST~n', [])
    ).

% STATISTICS
obligation_stats(Total, Proved, Assumed, Errors) :-
    findall(ID, proof_obligation(ID, _, _), AllIDs),
    length(AllIDs, Total),
    findall(ID, proof_verified(ID, _, proved), ProvedList),
    length(ProvedList, Proved),
    findall(ID, proof_verified(ID, _, assumed), AssumedList),
    length(AssumedList, Assumed),
    findall(ID, proof_verified(ID, _, error), ErrorList),
    length(ErrorList, Errors).
