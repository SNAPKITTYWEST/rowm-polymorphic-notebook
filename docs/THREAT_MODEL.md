# ROWM Threat Model & Security Analysis

**Version:** 1.0.0  
**Status:** Complete (verified 2026-07-27)  
**Authors:** Ahmad Ali Parr, Jessica SNAPKITTYWEST

---

## 1. Trust Boundaries

### Trusted Components (Assumed Secure)

1. **Prolog/Datalog engine** — Logic evaluation, rule grounding
2. **Rust runtime** — Memory safety, WORM enforcement
3. **Cryptographic libraries** — Blake3, Ed25519 (from established crates)
4. **Kernel/OS** — Process isolation, filesystem, system clock

### Untrusted Components (Validated Before Use)

1. **External source code** — All user-provided code in any language
2. **Notebook metadata** — Cell definitions, execution counts, outputs
3. **Network input** — Remote code, serialized instructions, proofs
4. **User-provided proofs** — Terms claimed to be proofs (external tools verify)
5. **Previously sealed artifacts** — Must re-verify linkage

---

## 2. In-Scope Threats

### Threat T1: Unauthorized Code Execution

**Attack Vector:** Agent without permission executes code on protected runtime

**Mitigation:**
- All dispatch routes through `dispatch_gated/5` (sealed entry point)
- Prolog checks: agent_active, trust_level ≠ tier_2, capability_issued, capability_active, not_revoked, not_expired, permission_granted, runtime_active
- No bypass: code cannot query `capability_active/2` or `dispatch_permitted/5` directly

**Residual Risk:** LOW  
- Requires compromise of Prolog engine or Rust runtime boundary  
- Requires forging Ed25519 signature OR recovering private key

**Test Case:** `test_unauthorized_agent_dispatch` in test_queries.pl

---

### Threat T2: Capability Bypass via Expiration Boundary

**Attack Vector:** Agent with expired capability uses it by manipulating time

**Mitigation:**
- Expiration boundary is EXCLUSIVE: `Timestamp < ExpiresAt` (not ≤)
- Capability ExpiresAt=T is inactive at time T
- get_time/1 uses system clock (not user-settable in normal mode)
- Timestamp checked in hardened `dispatch_gated/5`

**Residual Risk:** MEDIUM  
- If system clock can be manipulated (e.g., via NTP attack), boundary is bypassed  
- If Prolog engine can be forced to use stale time value, boundary is bypassed

**Mitigation Upgrade:** Pin to monotonic clock; disallow backward clock adjustments

**Test Case:** `test_expired_capability_rejection` in test_queries.pl (uses fixed time)

---

### Threat T3: Capability Revocation Evasion

**Attack Vector:** Agent retains reference to revoked capability; uses it before revocation is detected

**Mitigation:**
- Revocation recorded as fact: `capability_revoked(CapID, Reason)`
- Every dispatch checks: `\+ capability_revoked(CapID, _)` (negation-as-failure)
- Revocation is WORM-sealed (immutable once written)
- No capability can be "un-revoked" (monotonic)

**Residual Risk:** LOW  
- Requires Prolog engine compromise (to retract revocation fact)  
- Requires racing between revocation and dispatch in nanosecond window

**Test Case:** `test_revoked_capability_immediate_rejection` in test_queries.pl

---

### Threat T4: Proof Obligation Circumvention

**Attack Vector:** Agent forces transition to RELEASED without discharging all 4 proof obligations

**Mitigation:**
- Release gate 1: `all_proofs_satisfied(true)` queries proof_satisfied/2 for all 4 obligations
- Each proof obligation checked before release transition:
  1. InvariantPreservation — loop invariants maintained
  2. SemanticPreservation — source ≡ compiled semantics
  3. LoopInvariantMaintenance — bounds and termination
  4. ReceiptChainIntegrity — monotonic sequence + hash linkage
- Failed proof blocks transition (state stays RECEIPTED)

**Residual Risk:** LOW  
- Requires compromise of proof validator or Prolog engine  
- Requires forging proof artifact from external verifier

**Test Case:** `test_release_blocked_on_failed_proof` in test_queries.pl

---

### Threat T5: Receipt Chain Tampering

**Attack Vector:** Attacker modifies a receipt or inserts a receipt out of sequence

**Mitigation:**
- Receipt is signed with Ed25519 (once issued, immutable)
- Signature covers canonical receipt bytes + previous_receipt_hash
- Chain integrity verified via `receipt_chain_valid/1` predicate
- Monotonic sequencing enforced: `Receipt_N.sequence < Receipt_N+1.sequence`
- Out-of-order insertion detected by hash linkage mismatch

**Residual Risk:** MEDIUM  
- If Ed25519 private key is compromised, receipts can be forged  
- If Prolog engine allows fact retraction, receipts can be deleted
- Timestamp manipulation can reorder receipts within same second

**Mitigation Upgrade:** 
- Use hardware-backed Ed25519 keys (TPM/HSM)
- Append receipts to tamper-evident external ledger (blockchain or WORM storage)
- Use nanosecond timestamps or logical clocks

**Test Case:** `test_receipt_tampering_detected` in test_queries.pl

---

### Threat T6: Invariant Violation & Rollback Abuse

**Attack Vector:** Attacker deliberately violates invariant to trigger rollback, causing state loss or inconsistency

**Mitigation:**
- Invariant violation triggers rollback to LAST VALID CHECKPOINT
- Rollback is WORM-sealed (cannot undo a rollback)
- Violation is recorded in receipt chain (permanent audit trail)
- Repeated violations increment agent's violation counter (eventually triggers revocation)

**Residual Risk:** LOW  
- Attacker can cause localized rollback but cannot escape global audit
- Proof obligations will fail on violated execution path

**Test Case:** `test_invariant_violation_triggers_rollback` in test_queries.pl

---

### Threat T7: Self-Modifying Code Escape

**Attack Vector:** Cell code rewrites bytecode to bypass invariant checks or authorization

**Mitigation:**
- Code invariant is extracted and proved: `∀t ∈ [code_start, code_end]: M[t] == original[t]`
- If code violates this invariant, symbolic execution detects the rewrite
- Rewritten code is NOT re-compiled (would fail signature verification)
- Self-modification is only allowed via M4 feedback (between cells), not within execution

**Residual Risk:** LOW  
- Requires compromise of invariant extractor or symbolic execution engine  
- Requires proof obligation bypass (T4)

**Test Case:** `test_code_self_modification_detected` in test_queries.pl

---

### Threat T8: Cross-Cell State Leakage via M4

**Attack Vector:** Cell N writes secret to M4 definitions; Cell N+1 accesses secret

**Mitigation:**
- M4 definitions are sandboxed: max_expansion_depth=100, max_output_size=1MB
- Definitions are scoped to individual notebook execution (not global)
- Outputs are passed through Prolog facts (auditable, signed)
- Proof obligation checks semantic equivalence (no covert channels)

**Residual Risk:** MEDIUM  
- M4 macro expansion is Turing-complete (can compute anything)  
- If M4 sandbox limits are exceeded, behavior is undefined

**Mitigation Upgrade:** 
- Disable M4 macro expansion for untrusted agents
- Use restricted M4 builtins (no system calls)
- Require proof of M4 output equivalence

**Test Case:** `test_m4_cross_cell_access_denied` (when sandboxed)

---

## 3. Out-of-Scope Threats

### Threat OOS-1: Physical Attacks

**Scope:** Tamper with machine hardware, steal RAM, modify CPU

**Rationale:** System assumes hardware boundary is protected. Mitigated by:
- Deployment in secure data center
- Hardware security modules (HSM) for key storage
- Physical access controls

---

### Threat OOS-2: Compromised Kernel/OS

**Scope:** Kernel patch that violates memory isolation or filesystem integrity

**Rationale:** System assumes kernel is trustworthy. Cannot defend if OS is compromised.

**Mitigation:**
- Use verified kernel (e.g., seL4 with formal proof of isolation)
- Run in VM hypervisor with attestation
- Use signed boot (Secure Boot, measured launch)

---

### Threat OOS-3: Supply Chain Attack on Dependencies

**Scope:** Malicious update to blake3, ed25519-dalek, or swipl packages

**Rationale:** System assumes crates.io and package managers are trustworthy.

**Mitigation:**
- Pin dependency versions to verified commits
- Use cargo vendor to isolate dependencies
- Run internal security audit on critical crates
- Use binary reproducibility to verify builds

---

### Threat OOS-4: Spectre/Meltdown CPU Attacks

**Scope:** Side-channel attacks via CPU cache/speculation

**Rationale:** Prolog/Rust don't have built-in defenses against CPU microarchitecture attacks.

**Mitigation:**
- Use constant-time implementations for cryptography (already in ed25519-dalek)
- Run on CPUs with microcode patches
- Deploy in VM with IBRS enabled

---

## 4. Cryptographic Assumptions

### Assumption C1: Blake3 Collision Resistance

**Claim:** Two different inputs X, Y never produce same Blake3 hash (within 256-bit space)

**Usage:** Receipt hashing, capability hashing, checkpoint ancestry verification

**Risk:** If collision found, receipt chain integrity compromised

**Mitigation:** 
- Use full 256-bit hash (not truncated)
- Verify against Blake3 team's formal analysis
- Monitor cryptanalysis literature for attacks

---

### Assumption C2: Ed25519 Signature Unforgeability

**Claim:** Attacker without private key cannot produce valid Ed25519 signature

**Usage:** Receipt signing, release manifest signing, agent identity

**Risk:** If forged, attacker can fake any receipt or impersonate agent

**Mitigation:**
- Protect private keys in HSM or secure enclave
- Rotate keys periodically
- Publish public keys in tamper-evident registry
- Use key pinning for critical agents

**Current Gap:** HMAC-SHA256 used instead of Ed25519 (symmetric, not asymmetric)  
→ Cannot verify signatures without secret key  
→ Third-party audit impossible

---

### Assumption C3: SHA2 Second Preimage Resistance

**Claim:** Attacker cannot find X' ≠ X such that SHA256(X') = SHA256(X)

**Usage:** Source code hashing, bytecode hashing

**Risk:** If preimage found, two different code paths could have same hash

**Mitigation:**
- Use full 256-bit hash (not truncated)
- Transition to Blake3 where feasible (already using in receipts)

---

### Assumption C4: CBOR Deterministic Encoding

**Claim:** Two semantically identical values always encode to identical CBOR bytes

**Usage:** Canonical representation for hashing and signing

**Risk:** If non-deterministic, same payload hashes differently at different times

**Mitigation:**
- Define canonical byte order (little-endian, ascending key order)
- Validate CBOR library implements RFC 7049 deterministic encoding
- Test equivalence: canonical(X) = canonical(canonicalize(X))

---

## 5. Known Vulnerabilities & Limitations

### Vulnerability V1: Timestamp-Based Nondeterminism

**Issue:** Receipt timestamps make reproducible verification impossible

**Evidence:** Receipt includes `timestamp: 1719432000`, which differs on each execution

**Impact:** Two executions of same code produce different receipt hashes

**Mitigation:**
- Inject canonical time at sealing time (not at execution time)
- Use logical clock or block height (e.g., Git commit count)
- For testing: mock get_time/1

**Status:** Known, documented, accepted for now  
**Upgrade Path:** Canonical time injection via test harness

---

### Vulnerability V2: No Cross-System Replay Protection

**Issue:** Receipt from System A can be replayed in System B

**Evidence:** Release.pl gate 11 checks "no_active_dependencies" but doesn't verify context

**Impact:** Attacker can use same receipt in multiple contexts

**Mitigation:**
- Add context-specific binding: receipt includes system_id or deployment_context
- Require capabilities to be per-system (not global)
- Validate receipt is for current deployment

**Status:** Known, documented, acknowledged  
**Upgrade Path:** Per-system capability scoping

---

### Vulnerability V3: Notebook Mutation After Seal

**Issue:** Jupyter .ipynb file can be edited after cell execution recorded

**Evidence:** Notebook cells are mutable JSON; no signature covers them

**Impact:** Attacker modifies notebook cell after it executed, creating false history

**Mitigation:**
- Don't trust notebook file as source of truth
- Use Prolog facts as authoritative record of cell state
- Optionally: GPG-sign .ipynb file after each execution
- For production: use immutable notebook storage (e.g., IPFS, blockchain)

**Status:** Known, documented, mitigation available  
**Upgrade Path:** Signed notebook artifacts

---

### Vulnerability V4: HMAC Instead of Ed25519

**Issue:** Receipts are HMAC-signed (symmetric), not Ed25519-signed (asymmetric)

**Evidence:** Scripts use `hmac_sha256(data, WORM_SECRET)`, not Ed25519

**Impact:** Cannot verify receipts without secret key; third-party audit impossible; not legally admissible

**Compliance Impact:**
- SOX §302: Officer certification requires independent verification ✗
- GDPR Article 5(2): Accountability requires third-party audit capability ✗
- ISO 27001 A.12.4: Event logging integrity requires asymmetric signatures ✗

**Mitigation:**
- Implement Ed25519 signing for all receipts
- Keep private key in HSM (not in code)
- Publish public key for verification
- Rotate keys annually

**Status:** Known, unresolved  
**Upgrade Path:** Ed25519 implementation required before production release

---

### Vulnerability V5: Truncated Hashes

**Issue:** Some implementations truncate hashes to 16 bytes (128 bits) instead of full 256

**Evidence:** `signature.slice(0, 16)` in worm-receipts.js

**Impact:** Collision resistance reduced from ~2^128 to ~2^64; attackers can forge signatures with ~2^64 operations

**Mitigation:**
- Use full 256-bit hashes everywhere
- Audit all hash truncation and remove it
- Enforce minimum hash size in schemas

**Status:** Known, unresolved  
**Upgrade Path:** Full-hash enforcement in cryptographic layer

---

### Vulnerability V6: Untested Proof Tool Integration

**Issue:** Agda, Ada/SPARK, Lean 4 invocations are stubs; proof artifacts not validated

**Evidence:** No integration tests for external proof tools

**Impact:** Proof claims are unverified; release gates can pass falsely

**Mitigation:**
- Implement end-to-end tests for each proof tool
- Validate proof artifacts (terms, types, signatures)
- Reject unverified proof status
- Require manual proof review for critical proofs

**Status:** Known, documented  
**Upgrade Path:** Proof tool integration tests (Phase 8+)

---

## 6. Security Recommendations

### Before Production Release

1. **Implement Ed25519 signing** (NOT HMAC) for all receipts
2. **Enforce full-hash usage** (no truncation)
3. **Test proof tool integrations** end-to-end
4. **Add per-system context binding** to receipts (replay protection)
5. **Implement notebook signing** (GPG or Ed25519)
6. **Migrate to canonical time** (logical clock or Git height)
7. **Use HSM for private keys** (not filesystem storage)
8. **Enable SELinux/AppArmor** for process isolation
9. **Audit Prolog engine** for fact retraction vulnerabilities
10. **Pin dependency versions** and verify checksums

### Ongoing Operations

1. **Rotate Ed25519 keys** annually or on compromise suspicion
2. **Monitor Blake3/Ed25519 cryptanalysis** literature
3. **Run regular security audits** (penetration testing)
4. **Log all authorization failures** (detect attacks)
5. **Verify receipt chains** weekly (detect tampering)
6. **Back up WORM ledger** to geographically distributed stores
7. **Incident response plan** for leaked private keys

---

## 7. Compliance Mapping

| Compliance Requirement | Status | Evidence |
|------------------------|--------|----------|
| SOX §302 Independent Verification | ❌ BLOCKED | Need Ed25519 (HMAC blocks verification) |
| GDPR Art. 5(2) Accountability | ❌ BLOCKED | Need audit trail verification |
| HIPAA Audit Controls | ⏳ PARTIAL | Logs present; signing incomplete |
| ISO 27001 A.12.4 Event Logging | ⏳ PARTIAL | Truncated hashes reduce assurance |
| PCI-DSS 10.5 Log Integrity | ❌ BLOCKED | HMAC not accepted; need asymmetric |
| FedRAMP AC-6 Least Privilege | ✅ VERIFIED | dispatch_gated sealed entry point |

**Production Readiness: NOT COMPLIANT** (until Ed25519 implemented and tested)

---

## 8. Incident Response

### Suspected Private Key Compromise

1. Immediately revoke all capabilities issued by compromised agent
2. Rotate Ed25519 keypair (new key, new identity if necessary)
3. Audit all receipts signed by old key (may be forged)
4. Notify all systems that consume receipts from this agent
5. Drain old private key from all storage (HSM, filesystem, memory)
6. Document incident with timestamp and details

### Detected Proof Obligation Failure

1. Halt release process (transition blocked by gate)
2. Investigate failed proof: which obligation failed?
3. Review code for semantic changes or unproved invariants
4. Attempt re-proof with external verifier (Agda/Ada/Lean)
5. If re-proof fails: revert cell execution, understand root cause
6. Document in incident log (WORM-sealed)

### Detected Receipt Chain Tampering

1. Compute which receipts are affected (from gap in sequence)
2. Verify previous hash linkages (detect insertion point)
3. Revert affected cells (or entire notebook)
4. Investigate attacker access (how did they modify Prolog facts?)
5. Incident report to audit team and compliance officer

---

**GOVERNANCE PRINCIPLE: TRANSPARENCY OR MISTRUST.**

All vulnerabilities, mitigations, and residual risks are documented publicly. Users make informed decisions about deployment.

*"EVIDENCE OR SILENCE."*
