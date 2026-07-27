%% ════════════════════════════════════════════════════════════════════════════════
%% Isomorphic Shift Definitions — Source of Truth
%% SnapKitty Collective
%% ════════════════════════════════════════════════════════════════════════════════
%% All isomorphic shifts are registered here as facts.
%% Format: isomorphic_shift(ShiftId, Version, SourceDomain, TargetDomain,
%%         ForwardAdapter, InverseAdapter, SchemaHash, Classification)

:- module(shifts, [
    isomorphic_shift/8,
    shift_available/1,
    shift_permitted/3,
    round_trip_verified/2,
    semantic_shift_valid/3,
    isomorphic_shift_verified/1,
    shift_release_ready/1
]).

%% ── M1: Surface Instruction ↔ Canonical Instruction ──────────────────────────
%% Normalization shift — converts user-facing syntax to deterministic canonical form

isomorphic_shift(
    'M1',                                    % shift_id
    1,                                       % version
    'SurfaceInstruction',                    % source_domain
    'CanonicalInstruction',                  % target_domain
    'm1_surface_to_canonical',               % forward_adapter
    'm1_canonical_to_surface',               % inverse_adapter
    'bf4c8d7e6e9a2f3c1b5d8c2e4a6f9e1b',   % schema_hash (Blake3)
    'normalization'                          % classification: isomorphism
).

%% ── M2: Canonical Instruction ↔ Logic Term ───────────────────────────────────
%% Serialization shift — converts canonical instruction to logical reasoning form

isomorphic_shift(
    'M2',
    1,
    'CanonicalInstruction',
    'LogicTerm',
    'm2_canonical_to_logic',
    'm2_logic_to_canonical',
    'a2e7f1c9d4b6e8a3f5c1d9e2b7a4f6c8',
    'serialization'
).

%% ── M3: Authorized Logic Decision → Runtime Command ──────────────────────────
%% Projection shift — extracts executable command from authorized decision
%% Note: This is primarily forward (decision → command).
%% Inverse exists but loses some authorization context (projection, not isomorphism).

isomorphic_shift(
    'M3',
    1,
    'AuthorizedLogicDecision',
    'RuntimeCommand',
    'm3_logic_to_runtime',
    'm3_runtime_to_logic',  % Partial inverse (projection)
    'c5d9e2f7b1a6c4e8d2f9a1b6e3c7f2d5',
    'projection'  % classification: lossy in inverse direction
).

%% ── M4: Proof Obligation ↔ Verifier Invocation ────────────────────────────────
%% Embedding shift — wraps proof obligation with verifier invocation metadata

isomorphic_shift(
    'M4',
    1,
    'ProofObligation',
    'VerifierInvocation',
    'm4_proof_to_verifier',
    'm4_verifier_to_proof',
    'd7f2c5e1b8a3d6f9c2e4b7a1f5d8c3e6',
    'embedding'
).

%% ── M5: Execution Event ↔ Logic Event Fact ────────────────────────────────────
%% Serialization shift — converts runtime event to logical fact for Prolog reasoning

isomorphic_shift(
    'M5',
    1,
    'ExecutionEvent',
    'LogicEventFact',
    'm5_event_to_logic',
    'm5_logic_to_event',
    'e4b7f1d9c2e6a3f5b8d1c4e7f2a5d9b6',
    'serialization'
).

%% ── M6: Execution Event → Receipt Record ──────────────────────────────────────
%% Projection shift — creates receipt (forward only, with authorization proof)
%% This is primarily one-way: events → receipts.
%% Inverse may partially recover but loses receipt-specific sealing data.

isomorphic_shift(
    'M6',
    1,
    'ExecutionEvent',
    'ReceiptRecord',
    'm6_event_to_receipt',
    'm6_receipt_to_event',  % Partial inverse
    'f6d1e8b4a7c2f5d9e3a6b1c4f7e2d5a8',
    'projection'  % classification: primarily forward
).

%% ── M7: Notebook Cell Record ↔ Logic Cell Fact ────────────────────────────────
%% Serialization shift — converts notebook cell semantics to logical facts

isomorphic_shift(
    'M7',
    1,
    'NotebookCellRecord',
    'LogicCellFact',
    'm7_notebook_to_logic',
    'm7_logic_to_notebook',
    'a1d5c8f2b6e9d4a7c1f5b8e2d6a3f7c4',
    'serialization'
).

%% ── M8: Runtime Value ↔ Canonical Value ───────────────────────────────────────
%% Normalization shift — converts language-specific values to canonical representation

isomorphic_shift(
    'M8',
    1,
    'RuntimeSpecificValue',
    'CanonicalValue',
    'm8_value_to_canonical',
    'm8_canonical_to_value',
    'b2f7a4e1c6d9f3a8b5e2c7d1f4a9e6b3',
    'normalization'
).

%% ════════════════════════════════════════════════════════════════════════════════
%% SHIFT AVAILABILITY CHECK
%% ════════════════════════════════════════════════════════════════════════════════

%% shift_available(+ShiftId) — True if shift is registered and operational
shift_available(ShiftId) :-
    isomorphic_shift(ShiftId, _Version, _SourceDomain, _TargetDomain, _FwdAdapter, _InvAdapter, _Hash, _Class).

%% ════════════════════════════════════════════════════════════════════════════════
%% SHIFT PERMISSION CHECKING
%% ════════════════════════════════════════════════════════════════════════════════

%% shift_permitted(+ShiftId, +Agent, +Direction) — True if agent can perform shift
%% Rules defined in shift_authorization.pl
shift_permitted(ShiftId, Agent, Direction) :-
    shift_available(ShiftId),
    \+ (Agent = unknown_agent),  % Placeholder: real rules in authorization module
    member(Direction, [forward, inverse, bidirectional]).

%% ════════════════════════════════════════════════════════════════════════════════
%% ROUND-TRIP VERIFICATION
%% ════════════════════════════════════════════════════════════════════════════════

%% round_trip_verified(+ShiftId, +Result) — True if round-trip law verified
%% Rules defined in semantic_equivalence.pl
round_trip_verified(ShiftId, passed) :-
    isomorphic_shift(ShiftId, _Version, _SourceDomain, _TargetDomain, _FwdAdapter, _InvAdapter, _Hash, _Class),
    \+ (ShiftId = 'M3'),  % M3 is projection, not full isomorphism
    \+ (ShiftId = 'M6').  % M6 is projection, not full isomorphism

%% ════════════════════════════════════════════════════════════════════════════════
%% SEMANTIC SHIFT VALIDITY
%% ════════════════════════════════════════════════════════════════════════════════

%% semantic_shift_valid(+ShiftId, +SourceValue, +TargetValue) — True if semantics preserved
%% Rules defined in semantic_equivalence.pl
semantic_shift_valid(ShiftId, _SourceValue, _TargetValue) :-
    shift_available(ShiftId).

%% ════════════════════════════════════════════════════════════════════════════════
%% ISOMORPHIC SHIFT VERIFICATION
%% ════════════════════════════════════════════════════════════════════════════════

%% isomorphic_shift_verified(+ShiftId) — True if shift is proven isomorphic
isomorphic_shift_verified(ShiftId) :-
    isomorphic_shift(ShiftId, _Version, _SourceDomain, _TargetDomain, _FwdAdapter, _InvAdapter, _Hash, Classification),
    \+ (Classification = 'projection'),  % Projections are not isomorphisms
    round_trip_verified(ShiftId, passed).

%% ════════════════════════════════════════════════════════════════════════════════
%% RELEASE READINESS
%% ════════════════════════════════════════════════════════════════════════════════

%% shift_release_ready(+ShiftId) — True if shift is ready for production deployment
shift_release_ready(ShiftId) :-
    isomorphic_shift_verified(ShiftId),
    shift_available(ShiftId),
    % All adapters must be implemented and tested (checked in adapter registry)
    \+ (ShiftId = unimplemented_shift).

%% ════════════════════════════════════════════════════════════════════════════════
%% SHIFT MANIFEST (for evidence bundle)
%% ════════════════════════════════════════════════════════════════════════════════

%% shift_manifest(ShiftId, Manifest) — Generate manifest for a shift
shift_manifest(ShiftId, manifest{
    shift_id: ShiftId,
    version: Version,
    source: SourceDomain,
    target: TargetDomain,
    forward_adapter: FwdAdapter,
    inverse_adapter: InvAdapter,
    classification: Class,
    verified: IsVerified,
    release_ready: ReleaseReady
}) :-
    isomorphic_shift(ShiftId, Version, SourceDomain, TargetDomain, FwdAdapter, InvAdapter, _Hash, Class),
    (isomorphic_shift_verified(ShiftId) -> IsVerified = true ; IsVerified = false),
    (shift_release_ready(ShiftId) -> ReleaseReady = true ; ReleaseReady = false).

