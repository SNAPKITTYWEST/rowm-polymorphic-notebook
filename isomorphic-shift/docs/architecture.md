# Isomorphic Shift Architecture — Complete Specification

**Version**: 1.0  
**Status**: DESIGN & FOUNDATION COMPLETE  
**Authority**: Prolog/Datalog source-of-truth  
**Last Updated**: 2026-07-27

---

## Table of Contents

1. [Overview](#overview)
2. [Core Concepts](#core-concepts)
3. [Eight Required Mappings](#eight-required-mappings)
4. [Authority Model](#authority-model)
5. [Transaction Model](#transaction-model)
6. [Error Handling](#error-handling)
7. [Integration Points](#integration-points)
8. [Implementation Roadmap](#implementation-roadmap)

---

## Overview

The Isomorphic Shift layer is a formally-defined translation system that bridges multiple representation domains:

- **Notebook Instructions** (EmojiCode, HolyC, Python, JavaScript, Ada)
- **Canonical Intermediate Representation (ISIR)** (deterministic, CBOR-encoded)
- **Logic Terms** (Prolog/Datalog for reasoning)
- **Authorized Decisions** (with proof of authorization)
- **Runtime Commands** (executable on Rust, Erlang, LLVM, Ada targets)
- **Formal Proofs** (Lean 4, Agda proof obligations)
- **Execution Events** (runtime state changes)
- **Receipts** (WORM-sealed evidence of execution)
- **Notebook Cells** (Jupyter cells with sealed outputs)

Every transformation is **bidirectional** (or explicitly marked lossy), **semantically preserving**, **authority-preserving**, and **invertible**.

---

## Core Concepts

### Round-Trip Law

The fundamental property: For any isomorphic shift M,

```
inverse(forward(X)) = canonicalize(X)
forward(inverse(Y)) = canonicalize(Y)
```

This guarantees that information is not lost in either direction.

### Semantic Preservation

All transformations preserve:
- **Meaning**: The runtime behavior of the transformed value
- **Types**: Numeric types, arrays, unions, capabilities
- **Authority**: Permissions, proof status, release status
- **Structure**: Argument count, ordering, nesting

### Authority Preservation

Isomorphic Shift **MUST NOT**:
- Increase permissions (e.g., read-only → read-write)
- Change proof verification status (e.g., failed → passed)
- Modify release status (e.g., development → production)
- Escalate agent trust level
- Remove revocations

### Classification

Each shift is classified as:
- **Isomorphism**: Fully bidirectional, bijective (round-trip perfect)
- **Partial Isomorphism**: Bidirectional but lossy in one direction (e.g., projection)
- **Embedding**: One-way injection into larger space
- **Projection**: One-way extraction from larger space
- **Normalization**: One-way canonicalization
- **Serialization**: One-way encoding to portable form

---

## Eight Required Mappings

### M1: SurfaceInstruction ↔ CanonicalInstruction

**Direction**: Bidirectional (full isomorphism)  
**Classification**: Normalization  
**Source Domains**: EmojiCode, HolyC, Python, JavaScript, Ada  
**Target**: Deterministic CBOR-encoded canonical form

**Forward Mapping**:
1. Parse surface syntax for language
2. Extract verb and arguments
3. Validate shell safety (no metacharacters)
4. Reorder arguments in canonical order
5. Encode as deterministic CBOR
6. Compute Blake3 hash
7. Wrap in ISIR with metadata

**Inverse Mapping**:
1. Decode CBOR
2. Extract verb and arguments
3. Reconstruct surface syntax for target language
4. Validate reconstructed syntax
5. Verify round-trip: parse reconstructed, compare to original

**Invariants Preserved**:
- verb_identity
- argument_identity
- type_preservation
- authorization_identity

**Example**:
```
Forward:  ⚡ fn:FreqAnchor1618
→ CanonicalInstruction(verb=execute, args=[fn, FreqAnchor1618])

Inverse:  CanonicalInstruction(verb=execute, args=[fn, FreqAnchor1618])
→ ⚡ fn:FreqAnchor1618
```

### M2: CanonicalInstruction ↔ LogicTerm

**Direction**: Bidirectional (full isomorphism)  
**Classification**: Serialization  
**Source**: Canonical instruction (CBOR)  
**Target**: Prolog logic term (ground, fully instantiated)

**Forward Mapping**:
1. Decode CBOR from canonical instruction
2. Extract verb → functor
3. Extract arguments → argument list
4. Convert each argument type to Prolog representation
5. Ground all variables (instantiate with concrete values)
6. Construct logic_term(functor, args)

**Inverse Mapping**:
1. Extract functor → verb
2. Extract args → argument list
3. Convert Prolog types back to canonical types
4. Reconstruct canonical instruction
5. Encode as CBOR

**Invariants Preserved**:
- functor_identity
- argument_identity
- grounding_identity
- type_correspondence

**Example**:
```
Forward:  canonical_instruction(execute, [fn, 1618])
→ logic_term(execute, [fn, 1618])

Inverse:  logic_term(execute, [fn, 1618])
→ canonical_instruction(execute, [fn, 1618])
```

### M3: AuthorizedLogicDecision → RuntimeCommand

**Direction**: Primarily forward (projection)  
**Classification**: Projection (lossy in inverse)  
**Source**: Authorized logic decision (with Prolog proof)  
**Target**: Executable runtime command

**Forward Mapping**:
1. Extract agent and decision from authorized decision
2. Validate agent has sufficient trust for target
3. Query authorization rules: can_perform_shift/3
4. If authorized, extract command semantics
5. Map decision functor to runtime opcode
6. Map arguments to operands
7. Construct runtime_command(target, opcode, operands, context)

**Inverse Mapping** (Partial):
1. Extract opcode → decision functor
2. Extract operands → arguments
3. Reconstruct authorized_decision
4. Note: Authorization context may be lost (projection)

**Invariants Preserved**:
- semantics_preservation
- target_executability
- safety_preservation

**Authorization Check**:
- Agent must have execute permission
- Decision must pass authorization rules
- No permission escalation allowed

**Example**:
```
Forward:  authorized_decision(builder, logic_term(execute, [fn, 1618]), proof_hash, ts)
→ runtime_command(rust, execute_fn, [1618], {...})

Inverse:  runtime_command(rust, execute_fn, [1618], {...})
→ authorized_decision(?, logic_term(execute, [?, ?]), ?, ?)  [PARTIAL]
```

### M4: ProofObligation ↔ VerifierInvocation

**Direction**: Bidirectional (full isomorphism)  
**Classification**: Embedding  
**Source**: Formal proof obligation  
**Target**: Verifier invocation with result

**Forward Mapping**:
1. Extract theorem name and goal from obligation
2. Generate Blake3 hash of (theorem, hypotheses, goal)
3. Invoke verifier (Lean 4 or Agda)
4. Capture verification status (pending/verified/failed/timeout)
5. Wrap in verifier_invocation(proof_id, verifier, status, code, timestamp)

**Inverse Mapping**:
1. Extract proof_id
2. Extract theorem name from proof_code
3. Extract goal from proof_code
4. Reconstruct proof_obligation

**Invariants Preserved**:
- obligation_correspondence
- status_integrity
- proof_code_identity

**Example**:
```
Forward:  proof_obligation(Borrow_Step_Sound, [carry, borrow], goal_formula, bridge_verifier)
→ verifier_invocation(hash_xyz, lean4, verified, code, 1727404800)

Inverse:  verifier_invocation(hash_xyz, lean4, verified, code, 1727404800)
→ proof_obligation(Borrow_Step_Sound, [...], goal_formula, bridge_verifier)
```

### M5: ExecutionEvent ↔ LogicEventFact

**Direction**: Bidirectional (full isomorphism)  
**Classification**: Serialization  
**Source**: Runtime execution event  
**Target**: Prolog logic fact

**Forward Mapping**:
1. Extract event_id from event
2. Extract agent, action, outcome
3. Classify outcome (success/blocked/failed/timeout)
4. Generate WORM seal
5. Construct logic_event_fact(event_id, agent, action, outcome, worm_seal)

**Inverse Mapping**:
1. Extract event_id → find original event
2. Reconstruct execution_event from fact data
3. Verify WORM seal unchanged

**Invariants Preserved**:
- event_correspondence
- outcome_integrity
- worm_immutability

**Example**:
```
Forward:  execution_event(hash_abc, 1727404800, builder, generate, {result: ok}, parent_hash)
→ logic_event_fact(hash_abc, builder, generate, success, worm_seal_xyz)

Inverse:  logic_event_fact(hash_abc, builder, generate, success, worm_seal_xyz)
→ execution_event(hash_abc, 1727404800, builder, generate, {...}, parent_hash)
```

### M6: ExecutionEvent → ReceiptRecord

**Direction**: Primarily forward (projection)  
**Classification**: Projection (lossy in inverse)  
**Source**: Execution event  
**Target**: WORM-sealed receipt record

**Forward Mapping**:
1. Extract event_id from event
2. Generate receipt_id = hash(event_id, timestamp, authorization_proof)
3. Query authorization: was this event authorized?
4. Generate authorization_proof = hash(Prolog query result)
5. Determine verification_status (authorized/provisional/denied)
6. Generate WORM seal
7. Construct receipt_record(receipt_id, event_id, auth_proof, status, worm_seal, timestamp)

**Inverse Mapping** (Partial):
1. Extract event_id from receipt
2. Reconstruct partial execution_event
3. Note: Receipt-specific sealing and authorization proof may not be fully recoverable

**Invariants Preserved**:
- event_ancestry
- authorization_integrity
- worm_immutability
- timestamp_immutability

**Example**:
```
Forward:  execution_event(hash_abc, 1727404800, sentinel, verify, {...}, ...)
→ receipt_record(hash_receipt, hash_abc, proof_hash, authorized, worm_xyz, 1727404800)

Inverse:  receipt_record(hash_receipt, hash_abc, proof_hash, authorized, worm_xyz, 1727404800)
→ execution_event(hash_abc, 1727404800, sentinel, verify, {...}, ...)  [PARTIAL]
```

### M7: NotebookCellRecord ↔ LogicCellFact

**Direction**: Bidirectional (full isomorphism)  
**Classification**: Serialization  
**Source**: Jupyter notebook cell  
**Target**: Prolog logic fact

**Forward Mapping**:
1. Extract cell_id from notebook cell
2. Extract cell_type (code/markdown/raw)
3. Parse cell source to extract instructions
4. Extract proof obligations from cell comments
5. Generate WORM seal
6. Construct logic_cell_fact(cell_id, cell_type, instructions, obligations, worm_seal)

**Inverse Mapping**:
1. Extract cell_id
2. Find original notebook cell
3. Reconstruct cell record from fact

**Invariants Preserved**:
- cell_correspondence
- instruction_extraction_completeness
- worm_immutability

**Example**:
```
Forward:  notebook_cell(hash_c1, code, "⚡ fn:1618", [outputs], 5, metadata)
→ logic_cell_fact(hash_c1, code, [logic_term(execute, [fn, 1618])], [], worm_seal_m)

Inverse:  logic_cell_fact(hash_c1, code, [logic_term(execute, [fn, 1618])], [], worm_seal_m)
→ notebook_cell(hash_c1, code, "⚡ fn:1618", [...], 5, ...)
```

### M8: RuntimeSpecificValue ↔ CanonicalValue

**Direction**: Bidirectional (full isomorphism)  
**Classification**: Normalization  
**Source**: Language-specific value (i32, f64, string, array, capability)  
**Target**: Language-neutral canonical representation

**Forward Mapping**:
1. Inspect runtime value type
2. Map to canonical type:
   - i32, i64, u32, u64 → integer
   - f32, f64, f128 → float (with precision tag)
   - bool → boolean
   - string → string
   - bytes → bytes
   - array → list (with rank/shape)
   - map → map
   - tagged union → tagged_union
   - capability → capability_ref
   - hash/proof → hash_ref
3. Encode value as deterministic CBOR
4. Preserve all type information in type_tag
5. Construct canonical_value(type, cbor_bytes, type_tag)

**Inverse Mapping**:
1. Extract CBOR bytes
2. Decode using type_tag
3. Reconstruct runtime value in source language
4. Verify no type loss (array rank preserved, etc.)

**Invariants Preserved**:
- type_preservation
- value_preservation
- no_precision_loss
- no_capability_escalation

**Example**:
```
Forward:  runtime_value(i64, 1618, {})
→ canonical_value(integer, cbor_hex, i64_tag)

Inverse:  canonical_value(integer, cbor_hex, i64_tag)
→ runtime_value(i64, 1618, {})
```

---

## Authority Model

### Permission Hierarchy

```
none < read < write < execute < seal < verify < admin
```

### Agent Classes

- **sentinel**: sovereign trust (all permissions)
- **oracle**: high trust (read, analyze only)
- **builder**: high trust (read, write, generate, execute, seal)
- **archivist**: high trust (read, analyze, index, provenance)
- **berserker**: medium trust (read, analyze, inject)

### Shift Authorization

Each shift requires specific capabilities:

| Shift | Required Capability | Min Trust |
|-------|-------------------|-----------|
| M1 | read, write | low |
| M2 | read, analyze | medium |
| M3 | execute | high |
| M4 | verify | high |
| M5 | read, analyze | medium |
| M6 | seal | high |
| M7 | read, analyze | medium |
| M8 | read, write | low |

**Rule**: Agent must have required capability AND sufficient trust level.

**Absolute Prohibition**: No shift can escalate permissions or proof status.

---

## Transaction Model

### State Machine

```
   parsed
     ↓
 validated
     ↓
authorized  ← [Prolog query rejected: → rejected]
     ↓
transformed  ← [Adapter error: → failed]
     ↓
invariants_checked  ← [Invariant violated: → rejected]
     ↓
committed  ← [If applicable]
     ↓
receipted
     ↓
  [done]
```

### Atomicity

- **Commit phase**: Write result and receipt atomically
- **Failure**: Automatic rollback
- **Idempotency**: Same (transaction_id, canonical_input) → same result from ledger

### Transaction Record

```json
{
  "transaction_id": "blake3_hash",
  "shift_id": "M1",
  "agent": "builder",
  "source_value": "...",
  "target_value": "...",
  "timestamp": 1727404800,
  "state": "committed",
  "receipt_hash": "blake3_hash",
  "error": null
}
```

---

## Error Handling

### Rejection Codes

| Code | Meaning | Recoverable |
|------|---------|------------|
| MALFORMED_SOURCE | Source doesn't parse | Yes (fix input) |
| UNKNOWN_SHIFT | Shift not registered | No |
| UNSUPPORTED_VERSION | Shift version not supported | Yes (upgrade) |
| INCOMPATIBLE_SCHEMA | Source/target schema mismatch | No |
| AMBIGUOUS_MAPPING | Multiple interpretations | No |
| LOSSY_MAPPING | Claimed isomorphism but lossy | No (use projection) |
| INVALID_INVERSE | Inverse doesn't match forward | No (implementation bug) |
| MISSING_ADAPTER | Adapter not implemented | No |
| TIMEOUT | Transformation took too long | Yes (retry) |
| AUTHORIZATION_DENIED | Agent not authorized | No |
| INVARIANT_VIOLATION | Invariant not preserved | No (implementation bug) |
| SEMANTIC_MISMATCH | Meaning not preserved | No (implementation bug) |

### Failure Response

```json
{
  "error": {
    "code": "AUTHORIZATION_DENIED",
    "message": "Agent 'oracle' cannot perform M3 (execute permission required)",
    "detail": {
      "agent": "oracle",
      "shift": "M3",
      "reason": "ORACLE_READ_ONLY_VIOLATION"
    }
  }
}
```

---

## Integration Points

### Notebook System

- **Input**: Notebook cell source (EmojiCode, Python, etc.)
- **M1**: Convert to canonical instruction
- **M2**: Convert to logic term
- **M7**: Extract cell facts to Prolog
- **Output**: Logic facts queryable in Prolog

### Runtime Execution

- **Input**: Authorized decision from Prolog
- **M3**: Convert to runtime command
- **Execution**: Run on Rust/Erlang/Ada runtime
- **M5**: Record as execution event
- **M6**: Generate receipt
- **Output**: WORM-sealed receipt in ledger

### Formal Verification

- **Input**: Proof obligation from system
- **M4**: Invoke verifier (Lean 4/Agda)
- **Output**: Verification status and proof

### LOC Agent Integration

- **Source**: loc_agent.rs (DEVFLOW-FINANCE/snapkitty-core)
- **Triad Execution**: EmojiCode → M1 → M2 → M3 → Runtime → M5/M6
- **WORM Sealing**: Receipt → M6 → WORM ledger

---

## Implementation Roadmap

### Phase 1: Foundation (Current)
- [x] Domain definitions (domains.schema.json, domains.pl)
- [x] Shift registration (shifts.pl)
- [x] Authorization rules (shift_authorization.pl)
- [x] Semantic equivalence rules (semantic_equivalence.pl)
- [x] Invariant definitions (invariants.pl)
- [x] Release gates (shift_release.pl)

### Phase 2: Adapters
- [ ] M1 adapter: SurfaceInstruction ↔ CanonicalInstruction (Rust)
- [ ] M2 adapter: CanonicalInstruction ↔ LogicTerm (Rust)
- [ ] M3 adapter: AuthorizedLogicDecision → RuntimeCommand (Rust)
- [ ] M4 adapter: ProofObligation ↔ VerifierInvocation (Rust + FFI)
- [ ] M5 adapter: ExecutionEvent ↔ LogicEventFact (Rust)
- [ ] M6 adapter: ExecutionEvent → ReceiptRecord (Rust)
- [ ] M7 adapter: NotebookCellRecord ↔ LogicCellFact (Rust)
- [ ] M8 adapter: RuntimeSpecificValue ↔ CanonicalValue (Rust)

### Phase 3: Testing
- [ ] Round-trip tests for all isomorphic shifts
- [ ] Semantic preservation tests
- [ ] Authority tests (no permission escalation)
- [ ] Failure mode tests
- [ ] Logic integration tests

### Phase 4: Integration
- [ ] Integrate with LOC agent
- [ ] Wire with Prolog reasoner
- [ ] Hook into notebook system
- [ ] WORM ledger integration
- [ ] Production deployment

---

## References

- **LOC Agent**: /DEVFLOW-FINANCE/snapkitty-core/src/agents/loc_agent.rs
- **Sovereign Kernel**: /bob-orchestrator/prolog/sovereign_kernel.pl
- **Notebook**: /DEVFLOW-FINANCE/sovereign_notebook.ipynb
- **Session Handoff**: /SESSION_HANDOFF.md
- **Previous Audits**: /sov-kernel-monster/*.md

