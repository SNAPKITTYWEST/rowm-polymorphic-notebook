# Isomorphic Shift — Formal Translation & State Correspondence Layer

**Repository**: SnapKitty Collective  
**Status**: IMPLEMENTATION IN PROGRESS  
**Authority**: Datalog/Prolog source-of-truth  
**Governance Motto**: EVIDENCE OR SILENCE

---

## Mission

The Isomorphic Shift layer provides formally defined, bidirectional transformations connecting:

1. **Notebook Instructions** → Surface-level operation requests (EmojiCode, HolyC, Ada, Python, JavaScript)
2. **Runtime Representations** → Executable states (Rust, Erlang, Lean 4 proof obligations)
3. **Proof Representations** → Formal verification goals (Lean 4, Agda theorems)
4. **Execution Events** → Runtime state transitions logged to WORM
5. **Receipt Records** → Immutable evidence of execution and authorization

Each mapping is:
- **Bidirectional** with verified round-trip laws
- **Semantically preserving** (meaning survives transformation)
- **Authority-preserving** (no unauthorized permission escalation)
- **Invertible** (unless explicitly rejected as unsupported)
- **Classified** by category (isomorphism, embedding, projection, normalization, serialization, etc.)

---

## Core Principle: Round-Trip Laws

For every isomorphic shift M:
- **Forward then Inverse**: `inverse(forward(X)) = canonicalize(X)`
- **Inverse then Forward**: `forward(inverse(Y)) = canonicalize(Y)`
- **Semantic Preservation**: `meaning(X) = meaning(forward(X))`
- **Authority Preservation**: Shifts must NOT increase permissions, proof status, or release status

Lossy transformations are explicitly rejected as non-isomorphic.

---

## Eight Required Mappings (M1–M8)

| ID | Source Domain | Target Domain | Direction | Classification | Status |
|----|---|---|---|---|---|
| **M1** | SurfaceInstruction | CanonicalInstruction | ↔ | Normalization | Design |
| **M2** | CanonicalInstruction | LogicTerm | ↔ | Serialization | Design |
| **M3** | AuthorizedLogicDecision | RuntimeCommand | ↔ | Projection | Design |
| **M4** | ProofObligation | VerifierInvocation | ↔ | Embedding | Design |
| **M5** | ExecutionEvent | LogicEventFact | ↔ | Serialization | Design |
| **M6** | ExecutionEvent | ReceiptRecord | → | Projection | Design |
| **M7** | NotebookCellRecord | LogicCellFact | ↔ | Serialization | Design |
| **M8** | RuntimeSpecificValue | CanonicalValue | ↔ | Normalization | Design |

---

## Canonical Intermediate Representation (ISIR)

All shifts pass through a deterministic canonical form with fields:
- `schema_version: u32` — Versioning
- `shift_id: String` — Unique shift identifier
- `shift_version: u32` — Shift-specific version
- `source_domain: String` — Domain of origin
- `target_domain: String` — Target domain
- `direction: String` — "forward", "inverse", or "bidirectional"
- `value_type: String` — "instruction", "decision", "event", "proof", "receipt", etc.
- `payload: Vec<u8>` — CBOR-encoded canonical representation
- `invariant_set: Vec<String>` — List of preserved invariants
- `required_permission: String` — Authorization level required
- `source_hash: String` — Blake3 hash of source
- `canonical_hash: String` — Blake3 hash of canonical form (deterministic)
- `parent_receipt_hash: Option<String>` — Ancestry chain

**Canonical encoding**: Deterministic CBOR (RFC 7049) with canonical item ordering.

---

## Authority Protocol (12-Step Sequence)

1. **Parse** source → Validate syntax
2. **Validate** source schema → Check structural requirements
3. **Canonicalize** → Produce deterministic ISIR
4. **Identify shift** → Look up M1–M8 mapping
5. **Query Prolog** → Check authorization facts and rules
6. **Validate schema compatibility** → Verify domain/codomain match
7. **Execute bounded transformation** → Apply adapter with timeout
8. **Validate target** → Check output schema
9. **Verify invariants** → Confirm all invariants preserved
10. **Generate evidence** → Produce manifest and receipt
11. **Commit transaction** (if applicable) → Atomic append to ledger
12. **Append receipt** → WORM-seal the result

---

## Datalog/Prolog Integration

**Source-of-truth authority**: All shifts, domains, schemas, authorizations, and permissions are registered as Prolog facts.

**Required files**:
- `logic/shifts.pl` — All isomorphic shifts registered
- `logic/domains.pl` — Domain definitions with types and constraints
- `logic/schemas.pl` — Schema versions and compatibility rules
- `logic/invariants.pl` — Preserved invariants per shift
- `logic/shift_authorization.pl` — Authorization rules
- `logic/shift_validity.pl` — Validity checking rules
- `logic/semantic_equivalence.pl` — Semantic preservation rules
- `logic/shift_release.pl` — Release-readiness rules

**Prohibition**: Isomorphic Shift MUST NOT maintain independent copies of:
- Capabilities or revocations
- Proof verification status
- Release status
- Authorization decisions

All queries flow through Prolog.

---

## Implementation Stack

### Rust Adapters (`adapters/`)
- `m1_surface_to_canonical.rs` — Surface instruction normalization
- `m2_canonical_to_logic.rs` — Canonical to logic term serialization
- `m3_logic_to_runtime.rs` — Authorized logic decision to runtime command
- `m4_proof_to_verifier.rs` — Proof obligation to verifier invocation
- `m5_event_to_logic.rs` — Execution event to logic event fact
- `m6_event_to_receipt.rs` — Execution event to receipt record
- `m7_notebook_to_logic.rs` — Notebook cell to logic cell fact
- `m8_value_normalization.rs` — Runtime value to canonical value

Each adapter implements:
```rust
pub fn forward(source: &Source) -> Result<Target, ShiftError>
pub fn inverse(target: &Target) -> Result<Source, ShiftError>
pub fn verify_round_trip(source: &Source) -> Result<bool, ShiftError>
```

### Logic Engine (`logic/`)
- **Facts**: Shift definitions, domain definitions, schema versions
- **Rules**: Authorization, validity checking, semantic equivalence
- **Queries**: shift_available/1, shift_permitted/3, round_trip_verified/2
- **Tests**: Logic-level tests for rule correctness

### Tests (`tests/`)
- **RoundTrip**: Forward→Inverse→canonicalize, Inverse→Forward→canonicalize
- **SemanticPreservation**: Meaning, types, authority survive
- **Authority**: No permission escalation, no proof status change
- **Failure**: Malformed rejected, unsupported versions rejected, lossy mappings rejected
- **Logic**: Unregistered shifts denied, unauthorized agents denied, contradictions detected

---

## Transaction Model

### States
- `prepared` — Source parsed, schema validated
- `validated` — Canonical form generated, shift identified
- `authorized` — Prolog authorization successful
- `transformed` — Adapter executed successfully
- `invariants_checked` — All invariants verified
- `committed` — Transaction written to ledger (if applicable)
- `receipted` — Receipt WORM-sealed
- `rejected` — Operation denied (authorization, validation, or semantic failure)
- `failed` — Adapter timeout, malformed target, or other runtime error

### Atomicity
- Commit all or nothing
- Failed stages → automatic rollback

### Idempotency
- Same transaction ID + canonical input → returns existing result from ledger

---

## Evidence Bundle

All completed shifts produce:
1. **Manifest** — Metadata about the shift (source, target, timestamp, adapters used)
2. **Receipt** — WORM-sealed proof of execution
3. **Invariant Proof** — Evidence that all invariants were preserved
4. **Authorization Log** — Prolog query results proving authorization
5. **Test Results** — Round-trip verification evidence

---

## Restrictions

✗ NO additional sub-agents  
✗ NO destructive repository operations  
✗ NO deletion of existing Isomorphic Shift artifacts  
✗ NO Git history rewriting  
✗ NO replacing functioning implementations without evidence  
✗ NO self-authorized runtime mutations  
✗ NO isomorphism claims without verified inverse behavior  
✗ NO completion reports based on scaffolding

---

## Directories

```
isomorphic-shift/
├── README.md (this file)
├── schemas/
│   ├── domains.schema.json
│   ├── canonical.schema.json
│   ├── shift.schema.json
│   └── receipt.schema.json
├── logic/
│   ├── shifts.pl
│   ├── domains.pl
│   ├── schemas.pl
│   ├── invariants.pl
│   ├── shift_authorization.pl
│   ├── shift_validity.pl
│   ├── semantic_equivalence.pl
│   ├── shift_release.pl
│   └── tests/
├── adapters/
│   ├── lib.rs
│   ├── m1_surface_to_canonical.rs
│   ├── m2_canonical_to_logic.rs
│   ├── m3_logic_to_runtime.rs
│   ├── m4_proof_to_verifier.rs
│   ├── m5_event_to_logic.rs
│   ├── m6_event_to_receipt.rs
│   ├── m7_notebook_to_logic.rs
│   ├── m8_value_normalization.rs
│   └── tests/
├── proofs/
│   ├── round_trip_laws.lean
│   ├── semantic_preservation.lean
│   └── authority_preservation.lean
├── tests/
│   ├── round_trip_tests.rs
│   ├── semantic_preservation_tests.rs
│   ├── authority_tests.rs
│   ├── failure_tests.rs
│   └── logic_integration_tests.rs
├── evidence/
│   ├── manifests/
│   ├── receipts/
│   └── invariant_proofs/
└── docs/
    ├── architecture.md
    ├── mapping-catalog.md
    ├── invariants.md
    ├── source-of-truth-integration.md
    ├── threat-model.md
    └── limitations.md
```

---

## Next Steps

1. Define domain types and constraints (domains.pl, domains.schema.json)
2. Implement M1–M8 adapters with round-trip verification
3. Register shifts in Prolog (shifts.pl)
4. Implement authorization rules (shift_authorization.pl)
5. Build and test adapters
6. Integrate with existing notebook/ledger systems
7. Generate evidence bundle

---

## References

- **LOC Agent**: DEVFLOW-FINANCE/snapkitty-core/src/agents/loc_agent.rs (Triad execution)
- **Sovereign Kernel**: bob-orchestrator/prolog/sovereign_kernel.pl (Authorization logic)
- **Bifrost Transform**: DEVFLOW-FINANCE/collectivekitty/lib/bifrost/transform.ts (Event transformation pattern)
- **EmojiCode Mapping**: EmojiCode → HolyC/Ada/Rust/Haskell (loc_agent.rs emoji_map)
- **Notebook**: DEVFLOW-FINANCE/sovereign_notebook.ipynb (Triad pipeline trace)

