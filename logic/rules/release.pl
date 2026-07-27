% Rules: Release Readiness Determination
% Source-of-truth query for system readiness

:- module(release, [
    release_ready/1,
    readiness_check/2,
    all_proofs_satisfied/1,
    receipt_chain_sealed/1,
    no_revoked_capabilities/1,
    all_cells_complete/1,
    dispatch_gated/5
]).

:- use_module(authorization).
:- use_module(transitions).
:- use_module(proofs).
:- use_module(provenance).
:- use_module(receipts).
:- use_module(agents).
:- use_module(notebook_cells).

% release_ready(IsReady)
% Master query: system is release-ready if all conditions are met

release_ready(true) :-
    all_proofs_satisfied(true),
    receipt_chain_sealed(true),
    no_revoked_capabilities(true),
    all_cells_complete(true),
    receipt_chain_valid(true).

release_ready(false).

% readiness_check(CheckName, PassedBoolean)
% Individual readiness checks

readiness_check('proofs_satisfied', Result) :- all_proofs_satisfied(Result).
readiness_check('receipt_chain_sealed', Result) :- receipt_chain_sealed(Result).
readiness_check('no_revoked_capabilities', Result) :- no_revoked_capabilities(Result).
readiness_check('all_cells_complete', Result) :- all_cells_complete(Result).
readiness_check('receipt_chain_integrity', Result) :- receipt_chain_valid(Result).

% all_proofs_satisfied(IsSatisfied)
% All proof obligations must be verified

all_proofs_satisfied(true) :-
    proof_satisfied('proof_borrow_step_sound', true),
    proof_satisfied('proof_cons_cell_model', true),
    proof_satisfied('proof_dispatch_safe', true),
    proof_satisfied('proof_receipt_chain_integrity', true).

all_proofs_satisfied(false).

% receipt_chain_sealed(IsSealed)
% Receipt chain must have reached its head (final seal)

receipt_chain_sealed(true) :-
    receipt_chain_head(HeadID),
    receipt_issued(HeadID, _Seq, metatron, _Cap, _Instr, finalize, _InHash, _OutHash, _TS),
    receipt_valid(HeadID, true).

receipt_chain_sealed(false).

% no_revoked_capabilities(IsClean)
% Verify that no active agent capabilities have been revoked

no_revoked_capabilities(true) :-
    \+ (agent_active(Agent, true),
        capability_issued(_CapID, _Issuer, Agent, _Runtime, _Perms, _Issued, _Expires),
        capability_revoked(_CapID, _)).

no_revoked_capabilities(false).

% all_cells_complete(IsComplete)
% All cells must be either sealed or passed execution

all_cells_complete(true) :-
    \+ (cell_exists(CellID, _Type, _Kernel, _Visibility, _SourceHash, _OutputHash),
        \+ (cell_sealed(CellID, true) ; cell_metadata(CellID, _Class, _DepCount, passed))).

all_cells_complete(false).

% Detailed readiness report (for logging)

readiness_report(Report) :-
    findall(check(Name, Result), readiness_check(Name, Result), Checks),
    Report = Checks.

% dispatch_gated(AgentID, CapabilityID, TargetRuntime, Permission, IsPermitted)
% SEALED authorization gate: all external dispatch MUST pass through this predicate.
% This is the only entry point for runtime authorization decisions.
% Directly querying capability_active/2 or dispatch_permitted/5 is not allowed.

dispatch_gated(AgentID, CapID, TargetRuntime, Permission, true) :-
    % Step 1: Verify agent exists and is active
    agent_active(AgentID, true),
    % Step 2: Verify agent trust tier is not observer (tier_2)
    agent_trust_level(AgentID, TrustLevel),
    TrustLevel \= tier_2,
    % Step 3: Verify capability is held by agent for this runtime
    capability_issued(CapID, _IssuerID, AgentID, TargetRuntime, _Perms, _IssuedAt, _ExpiresAt),
    % Step 4: Verify capability is active (not revoked, not expired)
    \+ capability_revoked(CapID, _),
    get_time(Now),
    Timestamp is floor(Now),
    ExpiresAt_val is _ExpiresAt,
    Timestamp < ExpiresAt_val,
    % Step 5: Verify permission is in capability
    member(Permission, _Perms),
    % Step 6: Verify target runtime is active
    runtime_active(TargetRuntime, true).

dispatch_gated(_, _, _, _, false).
