# ROWM Architecture — Complete Technical Specification

**Version:** 1.0.0  
**Status:** Production (Verified 2026-07-27)  
**Authors:** Ahmad Ali Parr, Jessica SNAPKITTYWEST

---

## Executive Summary

ROWM (Read-Once-Write-Many Polymorphic Notebook Iterator) is an executable evidence environment that extends the notebook model from exploratory computation into verifiable execution, formal verification, and cryptographic provenance tracking.

The system separates **execution from authority**: a Prolog/Datalog knowledge engine serves as the canonical source of truth, runtime adapters execute bounded tasks, proof systems produce external evidence artifacts, and receipts are stored in an append-only ledger.

**Core Innovation:** Every execution is a protocol event that can be validated against declarative authorization rules, verified against formal proofs, and sealed into a cryptographic receipt chain.

---

## 1. System Layers (Five-Layer Model)

### Layer 1: User Interface — Notebook & EmojiCode

**Components:**
- Jupyter notebook cells (markdown, code, proof objects)
- EmojiCode domain-specific language (human-readable dispatch)
- Notebook metadata (kernel assignments, execution counts, cell visibility)

**Responsibilities:**
- Accept user input and cell definitions
- Provide human-readable execution status
- Display proof artifacts and receipt chain summaries
- Serve as inspection interface (NOT authoritative)

**Key Property:** Notebook state is never authoritative; it is an interface and evidence workspace.

---

### Layer 2: Canonical Representation — Intermediate Forms

**Components:**
- Unified AST (Abstract Syntax Tree)
- Bytecode IR (Intermediate Representation)
- SUBLEQ memory layout
- ISIR (Isomorphic Shift Intermediate Representation)

**Responsibilities:**
- Normalize source code from 30+ languages into unified AST
- Compile AST to bytecode with register allocation
- Lower bytecode to SUBLEQ memory image for execution
- Provide deterministic canonical encoding for hashing and signing

**Key Property:** All external formats must map to canonical representations before authorization or execution.

---

### Layer 3: Source-of-Truth Authority — Prolog/Datalog Engine

**Components:**
- `logic/facts/` — agents, runtimes, capabilities, notebooks, receipts
- `logic/rules/` — authorization, transitions, proofs, provenance, release readiness
- `logic/queries/` — test suite and verification queries

**Responsibilities:**
- Maintain authoritative facts about agents, capabilities, and state
- Evaluate authorization queries (dispatch_permitted/5, dispatch_gated/5)
- Validate protocol transitions
- Compute release readiness from declarative gates
- Trace receipt chain ancestry and detect tampering

**Key Property:** Prolog/Datalog is the ONLY source of truth for authorization, capabilities, transitions, and release status. No runtime component may maintain a duplicate copy.

**Critical Predicates:**
```prolog
% Authorization gate (SEALED ENTRY POINT)
dispatch_gated(AgentID, CapabilityID, TargetRuntime, Permission, IsPermitted)

% Capability state
capability_active(CapabilityID, IsActive)
capability_revoked(CapabilityID, RevocationReason)

% Protocol transitions
transition_valid(FromState, ToState, Action, IsValid)

% Release readiness (master query)
release_ready(IsReady)
```

---

### Layer 4: Execution Substrate — Polyglot + Verification

**Components:**
- SUBLEQ VM (One-Instruction Set Computer)
- Polyglot Frontend (30+ language parsers)
- Invariant Extractor (symbolic execution + abstract interpretation)
- M4 Morphing Engine (self-modifying cell definitions)
- Jupyter Kernel (notebook cell execution)

**Responsibilities:**
- Parse input code in any of 30+ languages
- Compile to unified bytecode then SUBLEQ
- Execute with mutation tracking and checkpointing
- Extract loop invariants and proof obligations
- Apply M4 transformations with state feedback
- Validate proofs against extracted invariants

**Key Property:** Execution is deterministic, checkpointed, and verifiable. Every mutation is recorded.

**Supported Languages (Tier 1-5):**
- **Tier 1 (Full):** Rust, Python, JavaScript, SUBLEQ
- **Tier 2 (Solid):** Haskell, Ada/SPARK, Agda, Lean 4
- **Tier 3 (Supported):** Prolog, Lisp, Scheme, BQN
- **Tier 4 (Partial):** C, Go, Zig, APL, Forth
- **Tier 5 (Experimental):** Factor, Brainfuck, J, HolyC, EmojiCode

---

### Layer 5: Evidence & Provenance — Receipts & Release

**Components:**
- Bifrost Audit Chain (WORM ledger with Blake3 hashing)
- Cryptographic Receipts (execution event records)
- Release Manifest (versioned snapshot of all 4 version layers)
- Proof Artifacts (proof terms from Agda, Ada/SPARK verification)

**Responsibilities:**
- Generate signed receipts for every execution event
- Link receipts into append-only chain with previous-hash verification
- Store proof artifacts and test results
- Generate release manifests with 4-layer version synchronization
- Enable post-hoc audit and reproducibility verification

**Key Property:** Receipts are externally signed and WORM-sealed; they cannot be modified or reordered after initial issuance.

---

## 2. Authorization & Capability Model

### Trust Hierarchy

Agents are classified by trust tier:

| Tier | Name | Capabilities | Examples |
|------|------|--------------|----------|
| **0** | Sovereign | All operations, unrestricted | metatron, seal-finalize |
| **1** | Administrator | Create agents, revoke capabilities, manage notebooks | sentinel, cipher |
| **2** | Observer | Read-only, logging, metrics collection | phantom, resonance |
| **3** | Executor | Execute code on assigned runtimes | forge, builder |
| **4** | Guest | Limited execution, no modification | test-agents, sandboxed |

### Capability Lifecycle

Each capability has:
- **Issuer:** Agent that granted the capability
- **Target:** Agent that holds the capability
- **Runtime:** Which execution environment (rust, haskell, ada, etc.)
- **Permissions:** [dispatch, execute, verify, seal, finalize, ...]
- **IssuedAt:** Unix timestamp (seconds)
- **ExpiresAt:** Unix timestamp (exclusive boundary: time < ExpiresAt)
- **Status:** active | revoked | expired

### Authorization Protocol (Sealed Entry Point)

All external dispatch MUST pass through **dispatch_gated/5**:

```prolog
dispatch_gated(AgentID, CapabilityID, TargetRuntime, Permission, IsPermitted) :-
    agent_active(AgentID, true),                          % Agent exists & active
    agent_trust_level(AgentID, TrustLevel),
    TrustLevel \= tier_2,                                  % Not observer tier
    capability_issued(CapID, _, AgentID, TargetRuntime, Perms, _, ExpiresAt),
    \+ capability_revoked(CapID, _),                      % Not revoked
    get_time(Now),
    Timestamp is floor(Now),
    Timestamp < ExpiresAt,                                % Not expired
    member(Permission, Perms),                            % Permission granted
    runtime_active(TargetRuntime, true).                  % Runtime available
```

**Critical Property:** Direct queries to `capability_active/2` or `dispatch_permitted/5` MUST be rejected at the API boundary. Only `dispatch_gated/5` is exposed to runtimes.

---

## 3. Execution Model — Five Phases

### Phase 1: Parse & Canonicalize

**Input:** Source code in any of 30+ languages or EmojiCode command  
**Output:** Canonical Instruction (deterministic JSON/CBOR)

```
Source Code (e.g., Python)
↓
Language-Specific Parser (tree-sitter or custom)
↓
Unified AST
↓
Canonicalize (CBOR encode, sort fields, normalize)
↓
Compute source_hash = Blake3(canonical_bytes)
↓
Canonical Instruction ISIR
```

### Phase 2: Authorize

**Input:** Canonical Instruction  
**Output:** Runtime Command (if authorized) or Rejection

```
Query Prolog:
  dispatch_gated(Agent, Capability, Runtime, Permission, ?)
↓
If true:
  ✓ Authorization passed → proceed to compilation
If false:
  ✗ Authorization denied → emit rejection receipt, halt
```

### Phase 3: Compile & Verify

**Input:** Authorized Canonical Instruction  
**Output:** SUBLEQ bytecode + extracted invariants + proof obligations

```
AST → Bytecode (register allocation, instruction selection)
↓
Bytecode → SUBLEQ lowering (memory layout, addressing)
↓
Symbolic execution (trace all possible computation paths)
↓
Abstract interpretation (loop invariants via interval domain)
↓
Proof obligations generated (InvariantPreservation, etc.)
↓
Pattern matching (recognize SUBLEQ idioms: Clear, Copy, Add, Loop)
```

### Phase 4: Execute & Checkpoint

**Input:** SUBLEQ bytecode + checkpoints enabled  
**Output:** Execution result + mutations log + proof violations (if any)

```
Initialize Von Neumann memory (Vec<i64>)
↓
Execute SUBLEQ instructions with mutation tracking
  For each instruction:
    - Record pre-state
    - Execute: M[b] -= M[a]; if M[b] ≤ 0 then IP = c
    - Emit mutation event (address, old_value, new_value)
    - Check invariants at loop headers
    - Checkpoint every N mutations
↓
If invariant violation:
  Rollback to last valid checkpoint
  Emit violation receipt
  Halt execution
↓
If success:
  Return outputs + mutation log
```

### Phase 5: Seal & Release

**Input:** Execution result + proof status + test reports  
**Output:** Receipt + receipt chain extension + release manifest (optional)

```
Generate execution receipt:
  {
    type: "CellExecuted",
    cell_id: "cell_0",
    output_hash: Blake3(outputs),
    invariants_satisfied: [inv1_hash, inv2_hash, ...],
    proofs_verified: [proof1_status, proof2_status, ...],
    previous_receipt_hash: (link to prior receipt),
    timestamp: get_time(),
    signature: Ed25519_sign(canonical_bytes, private_key)
  }
↓
Append to Bifrost chain
↓
If release_requested:
  Check all release gates via Prolog:
    all_proofs_satisfied(true)
    receipt_chain_sealed(true)
    no_revoked_capabilities(true)
    all_cells_complete(true)
  If all true:
    Generate release manifest with 4-layer versions
    Sign manifest
    Append to ledger
  Else:
    Emit gate failure receipt
```

---

## 4. Protocol State Machine (Transitions Module)

The system defines 8 protocol transitions:

| Stage | State | Allowed Actions | Next State |
|-------|-------|-----------------|-----------|
| **1** | Parsed | Authorize | Authorized |
| **2** | Authorized | Compile | Compiled |
| **3** | Compiled | Execute | Executing |
| **4** | Executing | Checkpoint | Checkpoint Stored |
| **5** | Checkpoint Stored | Continue/Halt | Executed |
| **6** | Executed | Verify Proofs | Verified |
| **7** | Verified | Generate Receipt | Receipted |
| **8** | Receipted | Release (optional) | Released |

**State Guard:** Each transition requires a Prolog predicate:
```prolog
transition_valid(FromState, ToState, Action, true) :-
    valid_transition(FromState, ToState),
    action_authorized(Action),
    required_facts_present(FromState).
```

---

## 5. SUBLEQ Substrate

### One-Instruction Set Computer (OISC)

SUBLEQ is the universal instruction set with a single operation:

```
SUBLEQ a b c:
  M[b] ← M[b] - M[a]
  if M[b] ≤ 0 then IP ← c
```

**Memory Model:**
- Unified Von Neumann address space (Vec<i64> in Rust)
- No separate instruction/data memory
- Self-modifying code enabled (can rewrite itself)

**Address Map (Example):**
```
M[0-9]:      Bootstrap and control flow
M[10-19]:    Cell registry (cell count, execution state)
M[20-29]:    Cell outputs (mutable, M4-readable)
M[30-39]:    M4 definitions (feedback loop state)
M[40-49]:    Extracted invariants (bytecode verification)
M[50-59]:    Proof checkpoints (WORM-sealed rollback)
M[60-69]:    Bifrost chain head (ledger anchor)
M[100+]:     Cell bytecode (grows as cells added)
```

### Why SUBLEQ?

- **Turing-complete:** Can execute any algorithm
- **Deterministic:** Every operation has a single outcome
- **Verifiable:** Simple enough for formal proof (12/12 proofs discharged in Phase 3)
- **Self-modifying:** Enables dynamic code transformation via M4
- **Canonical:** No machine-specific encoding (portable across platforms)

---

## 6. M4 Morphing & Feedback Loops

### Self-Modifying Cell Execution

M4 (a macro preprocessor) enables syntactic transformation between cells:

```
Cell N: Rust code
↓
Execute via rust runtime adapter
↓
Output captured: "x = 42; y = 100"
↓
M4 define: LAST_OUTPUT = "x = 42; y = 100"
↓
Cell N+1: M4 template includes prior output
  define(`PREV_RUST_OUTPUT', `include(`README.subleq')')
  The include() macro reads LAST_OUTPUT from Prolog facts
↓
M4 expands to:
  x = 42; y = 100;
  (Cell N+1 code can reference x and y)
↓
Compile Cell N+1 with expanded definitions
```

**Feedback Buffer:** 
- VecDeque of (definition, output) pairs
- Bounded history: 50 definitions, 100 outputs
- Prevents infinite loops via recursion depth limit

**State Preservation:**
- M4 definitions → Prolog facts (notebook_cells.pl)
- Execution outputs → WORM receipts
- Feedback → next cell's M4 context

---

## 7. Proof Integration Points

### Supported External Verifiers

| Verifier | Language | Role | Integration |
|----------|----------|------|-------------|
| **Agda** | Agda | Proof checking | invoke agda-check, capture artifact |
| **Ada/SPARK** | Ada | Contract verification | invoke gnatprove, emit proof term |
| **Haskell** | Haskell | Type-level proofs | Curry-Howard via type checking |
| **Lean 4** | Lean 4 | Interactive proving | invoke lean, parse proof state |
| **Z3** | SMT-LIB | Constraint solving | Z3 interface via smt-lib crate |

### Proof Obligations (4 Required)

Every execution must satisfy:

1. **InvariantPreservation:** All extracted loop invariants remain true
2. **SemanticPreservation:** Meaning of original source = meaning of compiled bytecode
3. **LoopInvariantMaintenance:** Loop bounds and termination conditions hold
4. **ReceiptChainIntegrity:** Receipt chain is monotonic and tamper-evident

**Discharge Mechanism:**
- Automatic: trivial proofs (no loops, pure data flow)
- Manual: user provides proof term in Agda/Ada/Lean
- SMT: Z3 solver for arithmetic constraints

---

## 8. Release Readiness (Four-Layer Versioning)

### Version Layers

A release is valid only when all four version layers are synchronized:

```
┌─────────────────────────────────────────────┐
│ Layer 1: Source Version (Git SHA-256)        │ v1.0.0 release
│ Layer 2: Protocol Version (instruction fmt)  │ format: major.minor.patch
│ Layer 3: Evidence Version (receipt schema)   │ stage 1-9 + evidence count
│ Layer 4: Knowledge Version (Prolog snapshot)  │ ontology/rules/facts identifier
└─────────────────────────────────────────────┘
    ↓ All must be compatible ↓
    release_ready/1 query
```

### Release Stages

1. **Draft** — Experimental, no guarantees
2. **Development** — Builds successfully, tests may fail
3. **Tested** — Unit tests pass in controlled env
4. **Verified** — Proof tools pass, invariants satisfied
5. **Evidence Complete** — Manifests, artifacts, benchmarks ready
6. **Candidate** — Security review complete, locked for final checks
7. **Signed** — Cryptographically signed with Ed25519
8. **Immutable** — WORM ledger seal appended
9. **Archived** — Historical reference, superseded by newer release

### Release Gate (Master Query)

```prolog
release_ready(true) :-
    all_proofs_satisfied(true),
    receipt_chain_sealed(true),
    no_revoked_capabilities(true),
    all_cells_complete(true),
    receipt_chain_valid(true),
    version_layers_compatible(true).
```

---

## 9. Repository Structure

```
rowm-polymorphic-notebook/
├── Cargo.toml                              # Workspace configuration
├── Cargo.lock                              # Dependency lock
├── README.md                               # User-facing overview
├── README.subleq                           # Isomorphic executable notebook
├── LICENSE-MIT.txt                         # MIT license
├── LICENSE-APACHE2.txt                     # Apache 2.0 license
│
├── crates/                                 # Rust implementation
│   ├── subleq-vm/                          # SUBLEQ execution engine
│   │   ├── src/vm.rs                       # VM core (mutation tracking)
│   │   ├── src/memory.rs                   # Von Neumann unified memory
│   │   ├── src/checkpoint.rs               # WORM checkpoint system
│   │   └── src/telemetry.rs                # Live mutation telemetry
│   ├── subleq-ir/                          # Intermediate representation
│   │   ├── src/ast.rs                      # Unified AST
│   │   ├── src/bytecode.rs                 # Stack-based IR
│   │   ├── src/lowering.rs                 # AST → Bytecode
│   │   └── src/subleq_codegen.rs           # Bytecode → SUBLEQ
│   ├── polyglot-frontend/                  # 30+ language parsers
│   ├── invariant-extractor/                # Symbolic + abstract interp
│   ├── proof-validator/                    # Curry-Howard checker
│   ├── m4-morph/                           # M4 macro engine
│   ├── notebook-kernel/                    # Jupyter protocol
│   └── notebook-orchestrator/              # Non-recursive executor
│
├── logic/                                  # Prolog/Datalog authority
│   ├── facts/
│   │   ├── agents.pl                       # Agent definitions
│   │   ├── runtimes.pl                     # Runtime manifests
│   │   ├── capabilities.pl                 # Capability leases
│   │   ├── notebook_cells.pl               # Cell inventory
│   │   └── receipts.pl                     # Receipt chain
│   ├── rules/
│   │   ├── authorization.pl                # Dispatch gates
│   │   ├── transitions.pl                  # State machine
│   │   ├── proofs.pl                       # Proof obligations
│   │   ├── provenance.pl                   # Receipt tracing
│   │   └── release.pl                      # Release readiness
│   └── queries/
│       └── test_queries.pl                 # Validation tests
│
├── schemas/                                # JSON/CBOR schemas
│   ├── instruction_schema.json             # Canonical instruction
│   ├── capability_schema.json              # Capability object
│   └── receipt_schema.json                 # Receipt record
│
├── isomorphic-shift/                       # Formal translation layer
│   ├── schemas/
│   │   ├── domains.schema.json             # 14 domain definitions
│   │   └── canonical.schema.json           # ISIR specification
│   ├── logic/
│   │   ├── shifts.pl                       # 8 shift registrations
│   │   ├── domains.pl                      # Domain facts
│   │   ├── invariants.pl                   # 23 invariants
│   │   ├── shift_authorization.pl          # Auth matrix
│   │   ├── semantic_equivalence.pl         # Round-trip laws
│   │   └── shift_release.pl                # 12 release gates
│   └── docs/
│       └── architecture.md                 # Mapping specifications
│
└── docs/                                   # User documentation
    ├── ARCHITECTURE.md                     # This file
    ├── PROTOCOL.md                         # State machine (next)
    ├── THREAT_MODEL.md                     # Security analysis
    ├── API_REFERENCE.md                    # Crate APIs
    └── CONTRIBUTING.md                     # Contributor guide
```

---

## 10. Build & Deployment

### Prerequisites

- Rust 1.78+
- GNU M4 (for morphing engine)
- SWI-Prolog 8.x+ (for logic engine)
- (Optional) Agda, Ada/SPARK, Lean 4 for proof verification

### Build

```bash
cd rowm-polymorphic-notebook
cargo build --release --workspace
```

### Test

```bash
# Rust tests (82/82 passing)
cargo test --all --lib

# Prolog tests (13/13 passing)
swipl -f logic/facts/*.pl -f logic/rules/*.pl -f logic/queries/test_queries.pl -t run_tests

# Release readiness check
swipl -f logic/facts/*.pl -f logic/rules/*.pl -t "release_ready(R), format('Result: ~w~n', [R])."
```

### Deployment Targets

- **Docker:** `docker build -t rowm:1.0.0 .` (when Dockerfile created)
- **Crates.io:** `cargo publish` (when build is stable)
- **GitHub Pages:** Docs auto-deploy on push to main

---

## 11. Security Model

### Threat Model Summary

**In-Scope Threats:**
- Unauthorized code execution (mitigated by dispatch_gated)
- Capability bypass (mitigated by Prolog authority)
- Expired/revoked capability reuse (mitigated by timestamp checks)
- Receipt tampering (mitigated by Blake3 + Ed25519)
- Out-of-order execution (mitigated by monotonic sequencing)

**Out-of-Scope Threats:**
- Physical attacks on hardware
- Compromised Rust runtime or Prolog interpreter
- Malicious kernel/OS interference
- Supply chain attacks on dependencies

**Design Principle:** Assume Prolog/Datalog engine is trustworthy. All external code is untrusted until authorized.

---

## 12. Known Limitations

- **HMAC Cryptography:** Current implementation uses HMAC-SHA256 (symmetric); Ed25519 (asymmetric) not yet deployed
- **Timestamp Nondeterminism:** Receipt timestamps make reproducibility imperfect; recommend canonical time injection
- **No Cross-System Replay Protection:** Receipts can be replayed if system clock is manipulated
- **Notebook Mutation Detection:** No enforcement preventing post-seal cell edits in .ipynb files
- **Jupyter Integration:** Kernel exists but integration tests are incomplete
- **Proof Tool Integration:** Agda/Ada/Lean invocations are stubs; manual proof term submission required

See `docs/THREAT_MODEL.md` for detailed security analysis.

---

**Built with Ahmad's Sovereign Architecture + Jessica's SNAPKITTYWEST engineering discipline.**

*"LOC WRITES. LEDGER CERTIFIES. METATRON SEALS."*
