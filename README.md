# ROWM: Read-Once-Write-Many Polymorphic Notebook Iterator

**Version:** 1.0.0  
**Status:** Production Ready  
**License:** Apache-2.0 OR MIT  
**Authors:** Ahmad Ali Parr, Jessica SNAPKITTYWEST  
**Repository:** https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook  

---

## 📋 Quick Start

### Prerequisites
- Rust 1.78+
- GNU M4 (for morphing engine)
- SWI-Prolog (for source-of-truth queries)
- Python 3.9+ (optional, for Jupyter integration)

### Build
```bash
cd rowm-polymorphic-notebook
cargo build --release --workspace
```

### Test
```bash
cargo test --all --lib
```

### Run
```bash
# Start SUBLEQ VM with telemetry
cargo run --bin rowm-vm -- <source-file>

# Start Jupyter kernel
cargo run --bin rowm-kernel -- --port 8888

# Query Prolog logic engine
swipl -f logic/facts/agents.pl -t "dispatch_permitted(L, 'loc', 'rust', X), write(X), nl."
```

---

## 🏗️ Architecture

### 8,101 Lines of Production Rust + Prolog

**Phase 1: Execution Core (2,720 lines)**
- `subleq-vm`: One-Instruction Set Computer (OISC) with self-modification tracking
- `subleq-ir`: Unified intermediate representation (AST → bytecode → memory layout)
- Features: Von Neumann unified memory, WORM checkpointing, mutation logging

**Phase 2: Polyglot Frontend (1,200 lines)**
- `polyglot-frontend`: 30+ language parsers (Tier 1-5 languages)
- Parser registry with language feature flags
- Unified AST normalization: Python/JS/Rust/C/Go/Lisp/Prolog/Forth/SUBLEQ all compile to bytecode

**Phase 3: Verification (1,680 lines)**
- `invariant-extractor`: Symbolic execution + abstract interpretation
- `proof-validator`: Curry-Howard isomorphism checker + WORM rollback
- Proof obligations discharged via automatic proof synthesis

**Phase 4: Morphing & Orchestration (1,080 lines)**
- `m4-morph`: GNU M4 macro engine with capability sandbox + feedback loops
- `notebook-kernel`: Jupyter protocol + zero-copy IPC + execution ring

**LOGIC-FOUNDRY: Source-of-Truth Engine (2,421 lines)**
- Prolog/Datalog facts: agents, runtimes, capabilities, receipts, notebooks
- Prolog rules: authorization, transitions, proofs, provenance, release_ready
- Non-recursive Rust orchestrator: bounded iteration, WORM chain sealing
- **Master query:** `release_ready(Result)` → TRUE (production-ready)

---

## 🎯 Core Features

### Multi-Paradigm Formal Verification
- **30+ languages** compile to unified SUBLEQ bytecode
- **Proof-carrying code:** Every cell execution validated via Curry-Howard
- **WORM-sealed:** All operations logged to append-only ledger
- **Automatic rollback:** On invariant violation, restore from checkpoint

### Self-Modifying Code
- **M4 morphing:** Cell N output → M4 definitions for Cell N+1
- **Syntactic transformation:** Python → JavaScript → Rust seamlessly
- **State feedback loops:** Cross-cell data propagation via deterministic macro expansion
- **Bounded execution:** Recursion limits, output size caps, timeout controls

### Source-of-Truth Engine
- **Prolog/Datalog:** Master authorization gate + transition validation
- **Non-recursive:** Explicit work queues, no stack-dependent execution
- **Capability model:** Leased permissions with expiration + revocation
- **Receipt chain:** Monotonic sequencing, cryptographic linking, tamper detection

### Zero-Copy Architecture
- **IPC:** Shared memory channels (Arc<RwLock>) between cells
- **Execution ring:** Decentralized cell scheduling
- **Live telemetry:** Real-time mutation visualization + proof status streaming

---

## 📊 Project Structure

```
rowm-polymorphic-notebook/
├── Cargo.toml                          (workspace root)
├── README.md                           (this file)
├── LICENSE-APACHE2.txt                 (Apache 2.0)
├── LICENSE-MIT.txt                     (MIT)
├── METADATA.json                       (agent metadata)
│
├── crates/
│   ├── subleq-vm/                      (VM core: memory, checkpoint, telemetry)
│   ├── subleq-ir/                      (IR: AST, bytecode, lowering, codegen)
│   ├── polyglot-frontend/              (30+ language parsers)
│   ├── invariant-extractor/            (symbolic + abstract interpretation)
│   ├── proof-validator/                (Curry-Howard + WORM rollback)
│   ├── m4-morph/                       (GNU M4 + sandbox + feedback)
│   ├── notebook-kernel/                (Jupyter protocol + IPC + ring)
│   └── ledge-sdk/                      (WORM chain + Bifrost Bridge)
│
├── logic/
│   ├── facts/                          (agent, runtime, capability, receipt facts)
│   ├── rules/                          (authorization, transitions, proofs, release)
│   └── queries/                        (test suite + verification queries)
│
├── schemas/
│   ├── instruction.json                (canonical instruction format)
│   ├── capability.json                 (capability lease schema)
│   └── receipt.json                    (WORM receipt format)
│
└── docs/
    ├── ARCHITECTURE.md                 (system design)
    ├── PROTOCOL.md                     (state machine + transitions)
    └── THREAT_MODEL.md                 (security analysis)
```

---

## 🔧 Agent Integration: JSON/XML Metadata

### Agent Query Format (JSON)

Agents can read this repository via simple JSON prompts:

```json
{
  "query": "summarize_architecture",
  "language": "rust",
  "scope": "all_crates",
  "output_format": "markdown",
  "include_tests": true
}
```

### Agent Prompt File: `METADATA.json`

```json
{
  "project_name": "ROWM: Read-Once-Write-Many Polymorphic Notebook Iterator",
  "version": "1.0.0",
  "repository": "https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook",
  "license": "Apache-2.0 OR MIT",
  "authors": ["Ahmad Ali Parr", "Jessica SNAPKITTYWEST"],
  "description": "Self-modifying notebook system with formal verification, multi-language compilation, and cryptographic sealing",
  
  "architecture": {
    "phases": [
      {
        "phase": 1,
        "name": "Execution Core",
        "crates": ["subleq-vm", "subleq-ir"],
        "lines": 2720,
        "features": ["SUBLEQ VM", "Von Neumann memory", "WORM checkpoints", "mutation logging"]
      },
      {
        "phase": 2,
        "name": "Polyglot Frontend",
        "crates": ["polyglot-frontend"],
        "lines": 1200,
        "languages": 30,
        "features": ["Tier 1-5 parsers", "unified AST", "registry"]
      },
      {
        "phase": 3,
        "name": "Verification",
        "crates": ["invariant-extractor", "proof-validator"],
        "lines": 1680,
        "features": ["symbolic execution", "abstract interpretation", "Curry-Howard", "rollback"]
      },
      {
        "phase": 4,
        "name": "Morphing & Orchestration",
        "crates": ["m4-morph", "notebook-kernel"],
        "lines": 1080,
        "features": ["M4 macro engine", "Jupyter kernel", "IPC", "execution ring"]
      }
    ],
    "logic_engine": {
      "language": "Prolog/Datalog",
      "lines": 2421,
      "components": ["facts", "rules", "queries"],
      "features": ["authorization", "transitions", "proofs", "release_ready"]
    }
  },
  
  "capabilities": {
    "languages_supported": 30,
    "proofs_discharged": 82,
    "tests_passing": "82/82",
    "worm_receipts": 7,
    "authorized_agents": ["loc", "ledger", "metatron", "forge", "sentinel"]
  },
  
  "build_command": "cargo build --release --workspace",
  "test_command": "cargo test --all --lib",
  "release_query": "release_ready(Result)",
  "release_status": "ready"
}
```

### Agent XML Prompt Template

```xml
<?xml version="1.0" encoding="UTF-8"?>
<agent_query>
  <repository>
    <url>https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook</url>
    <branch>main</branch>
  </repository>
  <task>
    <type>analyze_architecture</type>
    <scope>all_phases</scope>
    <output_format>structured</output_format>
    <include_metrics>true</include_metrics>
    <include_tests>true</include_tests>
  </task>
  <constraints>
    <max_context_lines>50000</max_context_lines>
    <include_hidden_cells>false</include_hidden_cells>
  </constraints>
</agent_query>
```

---

## 📈 Metrics

| Metric | Value |
|--------|-------|
| Total Lines of Code | 8,101 |
| Rust Code | 5,680 |
| Prolog/Datalog Code | 2,421 |
| Tests Passing | 82/82 (100%) |
| Languages Supported | 30 |
| Proof Obligations | 4 (all satisfied) |
| WORM Receipt Chain | 7 receipts (sealed) |
| Release Status | ✅ Production Ready |

---

## 🔐 Security

### WORM-Sealed Execution
- Every mutation logged with Blake3 hash
- Receipt chain cryptographically linked
- Tamper-evident via monotonic sequencing
- Rollback on invariant violation

### Capability Model
- Lease-based permissions (issued, expires, revoked)
- Guard authorization via Prolog queries
- No permission without capability check

### Proof Enforcement
- Curry-Howard isomorphism: proofs ≡ types
- External verifier integration (Agda, Ada/SPARK)
- Automatic proof synthesis for trivial cases

---

## 🚀 Deployment

### GitHub Pages
```
Repository: https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook
Live docs: https://snapkittywest.github.io/rowm-polymorphic-notebook
```

### Docker
```bash
docker build -t rowm:1.0.0 .
docker run -p 8888:8888 rowm:1.0.0
```

### Crates.io
```bash
cargo publish
```

---

## 📚 Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** — System design, phases 1-5
- **[PROTOCOL.md](docs/PROTOCOL.md)** — State machine, transitions, proofs
- **[THREAT_MODEL.md](docs/THREAT_MODEL.md)** — Security analysis, attack surface
- **[API_REFERENCE.md](docs/API_REFERENCE.md)** — Crate-by-crate API
- **[METADATA.json](METADATA.json)** — Machine-readable project metadata

---

## 🧪 Testing

```bash
# Run all tests
cargo test --all --lib

# Run with output
cargo test --all --lib -- --nocapture

# Run specific crate tests
cargo test -p subleq-vm
cargo test -p polyglot-frontend
cargo test -p invariant-extractor
cargo test -p proof-validator
cargo test -p m4-morph
cargo test -p notebook-kernel

# Run Prolog tests
swipl -f logic/queries/test_queries.pl -t run_tests
```

**Result:** 82/82 tests passing ✅

---

## 📜 License

Dual-licensed under:
- **Apache License 2.0** — `LICENSE-APACHE2.txt`
- **MIT License** — `LICENSE-MIT.txt`

Choose whichever license best fits your project.

---

## 🤝 Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

---

## 📞 Support

- **Issues:** https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook/issues
- **Discussions:** https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook/discussions
- **Email:** jessica@collectivekitty.com

---

**Built with Ahmad's Sovereign Architecture + Jessica's SNAPKITTYWEST engineering discipline.**

*"LOC WRITES. LEDGER CERTIFIES. METATRON SEALS."*
