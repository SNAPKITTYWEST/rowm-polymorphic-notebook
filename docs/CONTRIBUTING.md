# Contributing to ROWM

**Version:** 1.0.0  
**Status:** Open for Contributions  
**Authors:** Ahmad Ali Parr, Jessica SNAPKITTYWEST

---

## Welcome

ROWM is an open-source project seeking contributors in:
- **Formal verification** (Agda, Ada/SPARK, Lean 4 integration)
- **Language support** (new polyglot parsers)
- **Performance optimization** (VM speed, compilation)
- **Security auditing** (threat model review, penetration testing)
- **Documentation** (guides, examples, API docs)
- **Testing** (unit tests, integration tests, property-based tests)

---

## Core Values

1. **Logic Over Assumptions** — Every claim is backed by Prolog facts
2. **Evidence Over Assertions** — No feature ships without passing tests
3. **Reproducibility Over Convenience** — Build and test results must be deterministic
4. **Transparency Over Secrecy** — Threat model and known limitations are public
5. **Verification Over Belief** — Formal proofs preferred over documentation

---

## Getting Started

### Prerequisites

- Rust 1.78+ (install via [rustup.rs](https://rustup.rs))
- GNU M4 (for morphing engine)
- SWI-Prolog 8.x+ (for logic engine)
- Git

### Build

```bash
git clone https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook.git
cd rowm-polymorphic-notebook
cargo build --release --workspace
```

### Run Tests

```bash
# Rust tests
cargo test --all --lib

# Prolog tests
swipl -f logic/facts/*.pl -f logic/rules/*.pl -f logic/queries/test_queries.pl -t run_tests

# Release readiness check
swipl -f logic/facts/*.pl -f logic/rules/*.pl -t "release_ready(R), format('Result: ~w~n', [R])."
```

---

## Development Workflow

### 1. Pick an Issue

Check [GitHub Issues](https://github.com/SNAPKITTYWEST/rowm-polymorphic-notebook/issues) for:
- Bugs with `#audit` label (security/correctness)
- Features with `#feature` label
- Docs with `#documentation` label

### 2. Create a Branch

```bash
git checkout -b fix/issue-name  # for bug fixes
git checkout -b feature/issue-name  # for new features
git checkout -b docs/issue-name  # for documentation
```

### 3. Implement & Test

- Write code following project style (see below)
- Add tests for all new functionality
- Run full test suite: `cargo test --all --lib`
- Verify Prolog logic: `swipl ...` queries

### 4. Commit with Evidence

Include concrete evidence in commit message:

```
fix: Authorization gate now properly rejects tier_2 agents

Fixes #42: dispatch_gated/5 now checks agent_trust_level before returning true.
Boundary condition: Timestamp < ExpiresAt (not <=).

Evidence:
- Test case: test_tier2_agent_dispatch_denied passes
- Regression: test_expired_capability_acceptance now correctly fails
- Prolog validation: readiness_check('no_revoked_capabilities', true) passes

Co-Authored-By: Claude <noreply@anthropic.com>
```

### 5. Create Pull Request

```bash
git push origin feature/issue-name
gh pr create --fill
```

Include in PR description:
- What this fixes (or adds)
- How to test it
- Relevant documentation changes
- Any known limitations

### 6. Review & Merge

- Address code review feedback
- Re-run tests after changes
- Maintain focus on single issue (don't add unrelated fixes)
- Once approved: repo maintainers merge

---

## Code Style

### Rust

- Use `cargo fmt` before committing: `cargo fmt --all`
- Use `cargo clippy` for linting: `cargo clippy --all --lib`
- **No unsafe code** without explicit `// SAFETY: ...` comment explaining why
- Prefer `Result<T>` over panicking for errors
- Max line length: 100 characters (soft limit)

**Example:**

```rust
// Good: explicit error handling
pub fn load_proof(path: &str) -> Result<ProofTerm> {
    let bytes = std::fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|e| anyhow!("Invalid proof: {}", e))
}

// Avoid: panicking
pub fn load_proof_bad(path: &str) -> ProofTerm {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
```

### Prolog

- One fact per line (no multi-line facts)
- Comments above rules explaining intent
- Use descriptive predicate names (not `p/2`, use `authorization/2`)
- No anonymous variables (`_`) in public predicates

**Example:**

```prolog
% Good: clear, documented
% dispatch_gated/5: Sealed authorization entry point
% All external dispatch must pass through this predicate.
dispatch_gated(Agent, Cap, Runtime, Perm, true) :-
    agent_active(Agent, true),
    agent_trust_level(Agent, Tier),
    Tier \= tier_2,
    capability_issued(Cap, _, Agent, Runtime, Perms, _, Expires),
    \+ capability_revoked(Cap, _),
    get_time(Now),
    Now < Expires,
    member(Perm, Perms),
    runtime_active(Runtime, true).

% Avoid: cryptic
p(A, C, R, P, T) :- a(A), tnl(A, TL), TL \= t2, ci(C, _, A, R, PS, _, E), 
    \+ cr(C, _), gt(N), N < E, m(P, PS), ra(R, T).
```

### Documentation (Markdown)

- Use ATX headers (`#`, `##`, not underlines)
- Wrap at 80 characters for readability
- Include code examples with language tags
- Link to related documentation and GitHub issues

---

## Testing

### Unit Tests

Write tests in `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_gated_rejects_revoked_capability() {
        // Arrange
        let agent = "forge";
        let cap = "capa_revoked";
        let runtime = "rust";
        let permission = "execute";

        // Act
        let result = dispatch_gated(agent, cap, runtime, permission, ?);

        // Assert
        assert_eq!(result, false);
    }
}
```

### Integration Tests

Add files to `crates/*/tests/`:

```rust
// tests/integration_test.rs
#[test]
fn test_end_to_end_cell_execution() {
    // Full execution: parse → authorize → compile → execute → verify → receipt
}
```

### Prolog Tests

Add to `logic/queries/test_queries.pl`:

```prolog
test_dispatch_gated_denies_tier2 :-
    \+ dispatch_gated('phantom', 'capa_001', rust, execute, true),
    write('✓ Tier 2 agent correctly denied\n').
```

### Property-Based Tests

Use `proptest` for randomized testing:

```rust
proptest! {
    #[test]
    fn prop_invariant_preserved(seed in 0u64..1000) {
        let mut vm = create_test_vm(seed);
        vm.execute().expect("execution must succeed");
        assert!(check_all_invariants(&vm));
    }
}
```

---

## Merge Criteria

Before a PR can merge, all of the following must pass:

- [ ] **Build:** `cargo build --release --workspace` succeeds
- [ ] **Tests:** `cargo test --all --lib` passes (100%)
- [ ] **Linting:** `cargo clippy` has no warnings
- [ ] **Format:** `cargo fmt` produces no changes
- [ ] **Prolog:** `swipl ... release_ready(true)` passes
- [ ] **Documentation:** Relevant docs updated
- [ ] **Evidence:** Commit message includes test evidence
- [ ] **No Security Regressions:** No removal of authorization checks
- [ ] **No Unstaged Features:** No proto/planned code merged as complete
- [ ] **Reviewed:** At least 1 approving review from maintainer

---

## Adding a New Language to Polyglot Frontend

### Step 1: Implement Parser Trait

```rust
// crates/polyglot-frontend/src/parsers/mylang.rs
pub struct MyLangParser;

impl Parser for MyLangParser {
    fn parse(&self, source: &str) -> Result<Ast> {
        // Use tree-sitter or custom parser
        let tree = tree_sitter_mylang::parse(source)?;
        convert_tree_to_ast(tree)
    }

    fn language(&self) -> Language {
        Language::Mylang
    }
}
```

### Step 2: Add Language Enum

```rust
// crates/polyglot-frontend/src/language.rs
pub enum Language {
    // ... existing languages ...
    Mylang,
}

pub impl Language {
    pub fn tier(&self) -> LanguageTier {
        match self {
            Language::Mylang => LanguageTier::Tier4,  // or appropriate tier
            // ...
        }
    }
}
```

### Step 3: Register in Registry

```rust
// In registry.rs or similar
let mut registry = LanguageRegistry::new();
registry.register(Language::Mylang, Box::new(MyLangParser));
```

### Step 4: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mylang_parse_simple() {
        let source = "x := 10;";
        let ast = MyLangParser.parse(source).unwrap();
        assert_eq!(ast.root.statements.len(), 1);
    }
}
```

### Step 5: Update Documentation

- Add to `docs/API_REFERENCE.md` (polyglot-frontend section)
- Add to README.md language support table
- Update CONTRIBUTING.md if integration is complex

---

## Adding a New Proof Integration

### Step 1: Design Adapter

```rust
// crates/proof-validator/src/adapters/myprover.rs
pub struct MyProverAdapter;

impl ProofVerifier for MyProverAdapter {
    fn verify(&self, obligation: &ProofObligation, proof: &ProofTerm) -> Result<ProofStatus> {
        // Invoke external tool (Agda, Lean, etc.)
        // Return status: Proved | Disproved | Manual | Error
    }
}
```

### Step 2: Integrate with Validator

```rust
// In proof-validator.rs
pub struct ProofValidator {
    verifiers: HashMap<String, Box<dyn ProofVerifier>>,
}

impl ProofValidator {
    pub fn add_verifier(&mut self, name: &str, verifier: Box<dyn ProofVerifier>) {
        self.verifiers.insert(name.to_string(), verifier);
    }
}
```

### Step 3: Test End-to-End

```rust
#[test]
fn test_proof_verification_with_myprover() {
    let obligation = create_test_obligation();
    let proof = invoke_myprover(&obligation)?;
    assert_eq!(proof.status, ProofStatus::Proved);
}
```

---

## Security & Audit Contributions

### Reporting Security Issues

**Do NOT open public issues for security vulnerabilities.**

Email security concerns to: **[security-contact-TBD]**

Include:
- Description of vulnerability
- Proof-of-concept (if applicable)
- Steps to reproduce
- Suggested mitigation

Vulnerability disclosure timeline:
1. Report received
2. Assessment (48 hours)
3. Fix development (1-2 weeks typical)
4. Fix release & public disclosure

### Audit Contributions

If conducting security audit, provide:
- Threat description and CWE reference
- Reproduction steps
- Severity rating (CVSS or descriptive)
- Suggested remediation
- Proof-of-concept code (if applicable)

---

## Documentation Contributions

### Fixing Docs

- Fix typos, unclear sections, broken examples
- Update outdated information
- Add clarifying examples
- Link related documentation

### Adding New Docs

- Get consensus via GitHub issue first (avoid writing docs that won't be merged)
- Include with corresponding code changes
- Follow markdown style (see docs/ for examples)
- Keep examples runnable and tested

---

## Community Guidelines

1. **Be Respectful** — All contributors and maintainers are volunteers
2. **Assume Good Intent** — Technical disagreements are not personal
3. **Focus on Code** — Critique code, not contributors
4. **Share Knowledge** — Help newer contributors learn
5. **No Tolerance for Harassment** — We enforce a Code of Conduct

---

## Questions & Support

- **General questions:** GitHub Discussions
- **Implementation questions:** GitHub Issues
- **Security questions:** Private email (see above)
- **Design feedback:** Pull request comments

---

## Recognition

Contributors are recognized in:
1. Git commit author line (Co-Authored-By)
2. GitHub contributors graph
3. Release notes (for significant contributions)
4. Project README (for sustained contributors)

---

**Thank you for contributing to ROWM!**

*"EVIDENCE OR SILENCE." — Make your contributions count.*
