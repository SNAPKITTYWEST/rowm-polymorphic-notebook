# ROWM Protocol Specification — State Machine & Transitions

**Version:** 1.0.0  
**Status:** Normative (verified 2026-07-27)  
**Authors:** Ahmad Ali Parr, Jessica SNAPKITTYWEST

---

## 1. Protocol Overview

The ROWM protocol defines the formal state machine for cell execution, authorization, verification, and release. All state transitions are guarded by Prolog predicates; no transition may occur without passing source-of-truth validation.

**Core Principle:** Execution is NOT a black-box computation — it is a sequence of validated protocol events, each linked to the prior event via cryptographic receipt chain.

---

## 2. Eight-Stage Execution Pipeline

### Stage 1: PARSED

**Entry:** External instruction (EmojiCode, notebook cell, CLI command)  
**State:** Source representation parsed into AST

```
┌─────────────────────────────────┐
│ PARSED                          │
│                                 │
│ Input: raw source code          │
│ Language: any of 30+            │
│ Kernel: optional (inferred)     │
│                                 │
│ Output: Unified AST             │
│ Hash: Blake3(canonical_source)  │
└─────────────────────────────────┘
  ↓ transition: authorize
  [AUTHORIZED]
```

**Actions:**
- Parse source in language-specific parser
- Emit AST events (function def, assignment, loop, etc.)
- Compute source_hash

**Prolog Guard:**
```prolog
transition_valid(parsed, authorized, authorize, true) :-
    instruction_well_formed(Instr),
    language_supported(Instr.language).
```

---

### Stage 2: AUTHORIZED

**Entry:** Canonical instruction ISIR (parsed state)  
**State:** Authorization decision recorded

```
┌─────────────────────────────────┐
│ AUTHORIZED                      │
│                                 │
│ Agent: (from ISIR)              │
│ Capability: (from ISIR)         │
│ Runtime: (from ISIR)            │
│ Permission: dispatch|execute... │
│                                 │
│ Authorization result: PASS/FAIL │
└─────────────────────────────────┘
  ↓ [if PASS]
  transition: compile
  [COMPILED]
  
  ↓ [if FAIL]
  emit: AuthorizationDeniedReceipt
  [RECEIPTED] → end
```

**Actions:**
- Query Prolog: `dispatch_gated(Agent, Cap, Runtime, Permission, ?)`
- If true: record authorization event, proceed
- If false: emit denial receipt, halt

**Prolog Guard:**
```prolog
transition_valid(authorized, compiled, compile, true) :-
    dispatch_gated(Agent, CapID, Runtime, Perm, true),
    action_authorized(compile, Runtime).
```

---

### Stage 3: COMPILED

**Entry:** Authorized ISIR  
**State:** Bytecode generated, invariants extracted

```
┌─────────────────────────────────┐
│ COMPILED                        │
│                                 │
│ Bytecode: stack-based IR        │
│ Registers: R0-R15 allocated     │
│ SUBLEQ layout: M[0+] mapped     │
│                                 │
│ Invariants extracted:           │
│   - Loop invariants (interval)  │
│   - Proof obligations (4)       │
│ Proof obligations:              │
│   - InvariantPreservation       │
│   - SemanticPreservation        │
│   - LoopInvariantMaintenance    │
│   - ReceiptChainIntegrity       │
│                                 │
│ Status: ready_to_execute        │
└─────────────────────────────────┘
  ↓ transition: execute
  [EXECUTING]
```

**Actions:**
- AST → bytecode with register allocation
- Bytecode → SUBLEQ memory layout
- Symbolic execution trace (all paths)
- Abstract interpretation (loop invariants)
- Extract proof obligations
- Emit compilation receipt

**Prolog Guard:**
```prolog
transition_valid(compiled, executing, execute, true) :-
    bytecode_valid(Bytecode),
    \+ proof_obligation_unsatisfiable(ProofObl),
    invariants_extracted(Invariants).
```

---

### Stage 4: EXECUTING

**Entry:** Compiled SUBLEQ bytecode  
**State:** Virtual machine running with mutation tracking

```
┌─────────────────────────────────┐
│ EXECUTING                       │
│                                 │
│ Von Neumann memory: Vec<i64>    │
│ Instruction pointer: IP         │
│ Mutations: tracked & logged     │
│                                 │
│ For each SUBLEQ step:           │
│   M[b] -= M[a]                  │
│   if M[b] <= 0 then IP = c      │
│   emit MutationEvent             │
│   check_invariants()            │
│   periodic_checkpoint()         │
│                                 │
│ Exception handling:             │
│   - Infinite loop? (timeout)    │
│   - Invariant violation?        │
│     → rollback to checkpoint    │
└─────────────────────────────────┘
  ↓ [success or timeout]
  transition: checkpoint
  [CHECKPOINT_STORED]
  
  ↓ [invariant violation]
  emit: InvariantViolationReceipt
  rollback(checkpoint_id)
  [RECEIPTED] → end
```

**Actions:**
- Initialize memory with cell code
- Execute SUBLEQ instructions
- Emit mutation events (address, old_val, new_val)
- Check invariants at loop entry points
- Create checkpoints every N mutations
- Detect timeouts (default 30s)

**Prolog Guard:**
```prolog
transition_valid(executing, checkpoint_stored, checkpoint, true) :-
    execution_terminated(normal),
    checkpoint_valid(CheckpointID).
    
transition_valid(executing, receipted, halt_on_violation, true) :-
    invariant_violated(InvariantID),
    rollback_succeeded(CheckpointID).
```

---

### Stage 5: CHECKPOINT_STORED

**Entry:** Execution halted (normal or via rollback)  
**State:** Checkpoint is WORM-sealed (write-once)

```
┌─────────────────────────────────┐
│ CHECKPOINT_STORED               │
│                                 │
│ Checkpoint record:              │
│   - cell_id: string             │
│   - output_hash: Blake3         │
│   - invariants: [hash, ...]     │
│   - timestamp: Unix timestamp   │
│   - predecessor_hash: parent    │
│   - status: valid | violated    │
│                                 │
│ WORM sealed: no further writes  │
└─────────────────────────────────┘
  ↓ transition: verify
  [VERIFIED]
```

**Actions:**
- Record checkpoint to WORM storage
- Compute checkpoint_hash = Blake3(record)
- Link to previous checkpoint (DAG structure)
- Mark as immutable

**Prolog Guard:**
```prolog
transition_valid(checkpoint_stored, verified, verify, true) :-
    checkpoint_worm_sealed(CheckpointID),
    checkpoint_hash_valid(CheckpointID).
```

---

### Stage 6: VERIFIED

**Entry:** Checkpoint sealed  
**State:** Proof obligations validated

```
┌─────────────────────────────────┐
│ VERIFIED                        │
│                                 │
│ Proof validation:               │
│   - InvariantPreservation       │
│     Verified: ALL loop invs     │
│     maintained                  │
│   - SemanticPreservation        │
│     Verified: source meaning =  │
│     compiled meaning            │
│   - LoopInvariantMaintenance    │
│     Verified: bounds & exit     │
│   - ReceiptChainIntegrity       │
│     Verified: monotonic seq     │
│                                 │
│ Status: all_proofs_pass OR      │
│         some_proofs_manual      │
└─────────────────────────────────┘
  ↓ transition: seal
  [RECEIPTED]
```

**Actions:**
- Run automated proof (Z3 for arithmetic, simple patterns)
- Query external verifiers (Agda, Ada/SPARK) if requested
- Emit proof status receipt
- Link to prior receipt

**Prolog Guard:**
```prolog
transition_valid(verified, receipted, seal, true) :-
    all_proofs_satisfied(true),
    proof_status_recorded(ProofStatus),
    previous_receipt_linked(PriorReceiptHash).
```

---

### Stage 7: RECEIPTED

**Entry:** Proof verified (or manual override)  
**State:** Receipt is signed and chained

```
┌─────────────────────────────────┐
│ RECEIPTED                       │
│                                 │
│ Receipt record (WORM):          │
│   type: CellExecuted            │
│   cell_id: string               │
│   output_hash: Blake3           │
│   invariants_satisfied: [...]   │
│   proofs_verified: [...]        │
│   previous_receipt_hash: link   │
│   timestamp: now()              │
│   signature: Ed25519(bytes)     │
│                                 │
│ Chain link: receipt_hash ←→     │
│   prior receipt via hash        │
└─────────────────────────────────┘
  ↓ transition: release [optional]
  [RELEASED]
  
  or stay in RECEIPTED
  (release is not mandatory)
```

**Actions:**
- Generate receipt JSON/CBOR
- Compute receipt_hash = Blake3(canonical_receipt_bytes)
- Sign with Ed25519 private key (issuer = Agent)
- Append to Bifrost chain
- Emit receipt-added event to listeners

**Prolog Guard:**
```prolog
transition_valid(receipted, released, release, true) :-
    receipt_chain_valid(true),
    release_gates_passed(AllGates, true),
    version_layers_compatible(true).

% Alternative: stay in receipted indefinitely
transition_valid(receipted, receipted, noop, true) :-
    \+ release_requested.
```

---

### Stage 8: RELEASED

**Entry:** All release gates passed  
**State:** Release manifest generated and signed

```
┌─────────────────────────────────┐
│ RELEASED                        │
│                                 │
│ Release manifest:               │
│   - source_version: Git SHA     │
│   - protocol_version: 1.0.0     │
│   - evidence_version: stage+cnt │
│   - knowledge_version: Prolog ID│
│   - git_commit: HEAD sha        │
│   - receipt_chain_head: hash    │
│   - signer: Agent identity      │
│   - signature: Ed25519 sig      │
│   - previous_release_hash: link │
│                                 │
│ Status: immutable (archived)    │
└─────────────────────────────────┘
  ↓ Final state
  (no further transitions)
  
  Or Archive:
  ↓ transition: archive
  [ARCHIVED]
```

**Actions:**
- Query all release gates via Prolog
- Verify 4-layer versions sync
- Generate release manifest
- Sign manifest with Agent's Ed25519 key
- Append manifest receipt to ledger
- Tag in Git (if applicable)
- Mark cells as complete

**Prolog Guard:**
```prolog
transition_valid(receipted, released, release, true) :-
    readiness_check('proofs_satisfied', true),
    readiness_check('receipt_chain_sealed', true),
    readiness_check('no_revoked_capabilities', true),
    readiness_check('all_cells_complete', true),
    readiness_check('receipt_chain_integrity', true),
    version_layers_compatible(SourceVer, ProtocolVer, EvidenceVer, KnowledgeVer).

transition_valid(released, archived, archive, true) :-
    release_manifest_sealed(ManifestHash).
```

---

## 3. Authorization Protocol (Sealed Entry Point)

**All external dispatch MUST pass through dispatch_gated/5.**

### dispatch_gated/5 Predicate

```prolog
dispatch_gated(AgentID, CapabilityID, TargetRuntime, Permission, IsPermitted) :-
    % Step 1: Verify agent exists and is active
    agent_active(AgentID, true),
    
    % Step 2: Verify agent trust tier is not observer (tier_2)
    agent_trust_level(AgentID, TrustLevel),
    TrustLevel \= tier_2,
    
    % Step 3: Verify capability is held by agent for this runtime
    capability_issued(CapabilityID, _IssuerID, AgentID, TargetRuntime, Permissions, _IssuedAt, ExpiresAt),
    
    % Step 4: Verify capability is active (not revoked, not expired)
    \+ capability_revoked(CapabilityID, _),
    get_time(Now),
    Timestamp is floor(Now),
    Timestamp < ExpiresAt,  % CRITICAL: exclusive boundary
    
    % Step 5: Verify permission is in capability grant
    member(Permission, Permissions),
    
    % Step 6: Verify target runtime is active
    runtime_active(TargetRuntime, true).

dispatch_gated(_, _, _, _, false).  % Default: deny
```

### Critical Enforcement

**PROHIBITED DIRECT QUERIES:**
- ~~`capability_active/2`~~ (internal only)
- ~~`dispatch_permitted/5`~~ (internal only)
- ~~`agent_authorized/3`~~ (internal only)

**ALLOWED QUERIES:**
- `dispatch_gated/5` (sealed entry point)
- `release_ready/1` (release readiness)
- `readiness_check/2` (diagnostic)

---

## 4. Receipt Chain Structure

### Receipt Record (WORM Ledger)

```json
{
  "receipt_id": "sha256_of_contents",
  "sequence": 42,
  "type": "CellExecuted",
  "cell_id": "cell_0",
  "agent": "forge",
  "timestamp": 1719432000,
  "output_hash": "blake3_hash_of_output",
  "invariants_satisfied": [
    "inv_1_loop_bound",
    "inv_2_array_bounds",
    "inv_3_type_safety"
  ],
  "proofs_verified": [
    "proof_invariant_preservation",
    "proof_semantic_equivalence"
  ],
  "previous_receipt_hash": "hash_of_prior_receipt",
  "signature": "ed25519_signature",
  "chain_position": "N in DAG"
}
```

### Chain Integrity Checks

**Monotonic Sequencing:**
```prolog
receipt_sequence_valid(Receipt1, Receipt2) :-
    Receipt1.sequence < Receipt2.sequence,
    timestamp(Receipt1) < timestamp(Receipt2).
```

**Hash Linkage:**
```prolog
receipt_chain_valid(ReceiptID) :-
    receipt_issued(ReceiptID, _Seq, _Agent, _Cap, _Instr, _Action, _InHash, _OutHash, _Timestamp),
    receipt_issued(PriorReceiptID, PriorSeq, _, _, _, _, _, _, _),
    PriorSeq + 1 = _Seq,
    receipt_hash(PriorReceiptID, PriorHash),
    receipt_data(ReceiptID, Data),
    Data.previous_receipt_hash == PriorHash.
```

**Tamper Detection:**
```prolog
receipt_tampered(ReceiptID) :-
    receipt_issued(ReceiptID, _, _, _, _, _, InHash, OutHash, _),
    stored_in_hash = hash(stored_data),
    (stored_in_hash \= InHash ; stored_out_hash \= OutHash).
```

---

## 5. Capability Lifecycle

### States

```
Issued → Active → (Revoked OR Expired)

Issued:
  - created by issuer agent
  - stored in capabilities.pl
  - has future ExpiresAt timestamp

Active:
  - \+ capability_revoked(CapID, _)
  - get_time() < ExpiresAt
  - can be used in dispatch_gated/5

Revoked:
  - capability_revoked(CapID, RevocationReason) fact exists
  - cannot be used, period
  - reason recorded for audit

Expired:
  - get_time() >= ExpiresAt (at boundary: exclusive <)
  - automatically inactive
  - new capability must be issued
```

### Revocation Protocol

```prolog
revoke_capability(CapID, Reason) :-
    % Only sovereign or admin agents can revoke
    agent_trust_level(Revoker, Tier),
    (Tier = tier_0 ; Tier = tier_1),
    % Record revocation as WORM fact
    assertz(capability_revoked(CapID, Reason)),
    % Emit revocation receipt
    emit_receipt(type=CapabilityRevoked, cap_id=CapID, reason=Reason).
```

---

## 6. Release Gates (12-Point Checklist)

Before transitioning to RELEASED, all gates must pass:

```prolog
release_ready(true) :-
    gate_1_all_proofs_satisfied,
    gate_2_receipt_chain_sealed,
    gate_3_no_revoked_capabilities,
    gate_4_all_cells_complete,
    gate_5_receipt_chain_integrity,
    gate_6_no_code_mutation_after_seal,
    gate_7_no_untested_paths,
    gate_8_version_layers_compatible,
    gate_9_manifests_generated,
    gate_10_signatures_valid,
    gate_11_no_active_dependencies,
    gate_12_performance_acceptable.
```

**For production release:** All 12 gates must return true.  
**For staged release:** Progressively check gates; advance stage when satisfied.

---

## 7. Transition Error Handling

### Invalid Transition Attempt

```
If transition_valid(FromState, ToState, Action, false):

1. Emit TransitionDeniedReceipt
   {
     type: "TransitionDenied",
     from_state: FromState,
     to_state: ToState,
     action: Action,
     reason: failed_predicate_name,
     prolog_query_result: false
   }

2. Record in Prolog: transition_failed(FromState, Action, Reason)

3. Optionally: Rollback to prior checkpoint

4. Stay in FromState (no transition occurs)

5. Let user/agent retry or handle error
```

### Automatic Rollback on Invariant Violation

```
If invariant_violated during EXECUTING:

1. Emit InvariantViolationReceipt
   {
     type: "InvariantViolated",
     invariant_id: InvID,
     violated_at: instruction_pointer,
     expected: predicate_formula,
     actual: observed_state
   }

2. Query Prolog: checkpoint_valid(CheckpointID)

3. If valid: restore memory from checkpoint

4. Transition: EXECUTING → CHECKPOINT_STORED → VERIFIED
   (with violation recorded)

5. Emit post-rollback receipt with new state

6. Release readiness check will fail (gate 4)
```

---

## 8. Concurrency & Isolation

### Single-Cell Execution (No Concurrency)

The protocol assumes:
- One cell executes at a time
- No concurrent dispatch_gated calls
- Receipt chain is linearized (no branches)

**Rationale:** Formal verification of concurrent state is intractable; linearization enables proof certification.

### Multi-Notebook Isolation

Each notebook:
- Has independent Prolog instance (or isolated rule set)
- Maintains separate receipt chain
- Cannot interfere with other notebooks
- Can cross-reference via signed manifests

---

## 9. State Diagram

```
                    ┌─────────────────────┐
                    │  PARSED             │
                    │  (AST generated)    │
                    └─────────────────────┘
                            ↓ authorize
                    ┌─────────────────────┐
                ┌──→│  AUTHORIZED         │───────┐
                │   │  (dispatch_gated OK)│       │
                │   └─────────────────────┘       ↓ [DENY]
                │           ↓ compile      [AuthorizationDenied]
                │   ┌─────────────────────┐       ↓
                │   │  COMPILED           │  [RECEIPTED] → END
                │   │  (bytecode ready)   │
                │   └─────────────────────┘
                │           ↓ execute
                │   ┌─────────────────────┐
                │   │  EXECUTING          │
                │   │  (mutations logged) │
                │   │  [invariant check]  │
                │   └─────────────────────┘
        [VIOLATION]  ↓ checkpoint
           ↓         ┌─────────────────────┐
      [ROLLBACK]    │  CHECKPOINT_STORED  │
           │        │  (WORM sealed)      │
           └───────→└─────────────────────┘
                            ↓ verify
                    ┌─────────────────────┐
                    │  VERIFIED           │
                    │  (proofs checked)   │
                    └─────────────────────┘
                            ↓ seal
                    ┌─────────────────────┐
                    │  RECEIPTED          │◄──────┐
                    │  (receipt chained)  │       │
                    └─────────────────────┘       │
                      ↓ release        [NOOP]    │
                      │                (stay)    │
                      ├─────────────────────────→┤
                      ↓
                    ┌─────────────────────┐
                    │  RELEASED           │
                    │  (manifest signed)  │
                    └─────────────────────┘
                            ↓ archive (optional)
                    ┌─────────────────────┐
                    │  ARCHIVED           │
                    │  (historical ref)   │
                    └─────────────────────┘
```

---

## 10. Extensibility

### Adding New Transitions

To add a new transition (e.g., CUSTOM_ACTION):

1. **Define entry state:** `transition_valid(FromState, CustomState, custom_action, true) :- ...`
2. **Implement Prolog guard:** Add rule to transitions.pl
3. **Update state machine:** Add state record in transitions.pl facts
4. **Test:** Add test case to logic/queries/test_queries.pl
5. **Document:** Update this PROTOCOL.md

### Version Compatibility

Protocol 1.0.0 is IMMUTABLE for releases tagged "1.x.y". New protocol features (2.0.0) must:
- Be backward-compatible with 1.x read paths
- OR version-gate old vs. new logic
- Include migration documentation

---

**GOVERNANCE: EVIDENCE OR SILENCE.**

All transitions are logged, verified, and sealed. The protocol is not just specification — it is executable law.

*"LOC WRITES. LEDGER CERTIFIES. METATRON SEALS."*
