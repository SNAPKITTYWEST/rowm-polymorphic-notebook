% Facts: Capability and Lease Registry
% WORM-sealed capability objects

:- module(capabilities, [
    capability_issued/7,
    capability_valid/2,
    capability_revoked/2,
    lease_active/2
]).

:- dynamic(capability_revoked/2).

% capability_issued(CapabilityID, IssuerAgent, TargetAgent, TargetRuntime, Permissions, IssuedAt, ExpiresAt)
% CapabilityID = hash(issuer || target_agent || target_runtime || issued_at)

capability_issued(
    'capa_001_loc_rust_exec',
    cipher,
    loc,
    rust,
    [dispatch, execute, seal],
    1719432000,
    1719518400
).

capability_issued(
    'capa_002_forge_rust_build',
    cipher,
    forge,
    rust,
    [build, verify, emit_artifact],
    1719432000,
    1719518400
).

capability_issued(
    'capa_003_sentinel_ada_verify',
    cipher,
    sentinel,
    ada,
    [verify, audit, attest],
    1719432000,
    1719604800
).

capability_issued(
    'capa_004_resonance_haskell_compute',
    cipher,
    resonance,
    haskell,
    [compute, synthesize, verify_math],
    1719432000,
    1719604800
).

capability_issued(
    'capa_005_metatron_haskell_seal',
    cipher,
    metatron,
    haskell,
    [finalize, worm_seal, meta_verify],
    1719432000,
    1719518400
).

capability_issued(
    'capa_006_phantom_rust_log',
    cipher,
    phantom,
    rust,
    [log, observe, collect_metrics],
    1719432000,
    1719432000
).

% capability_valid(CapabilityID, IsValid)
% A capability is valid if:
%   - It was issued (exists in capability_issued/7)
%   - It has not been revoked (not in capability_revoked/2)
%   - Its expiration time has not passed (ExpiresAt > now)

capability_valid('capa_001_loc_rust_exec', true).
capability_valid('capa_002_forge_rust_build', true).
capability_valid('capa_003_sentinel_ada_verify', true).
capability_valid('capa_004_resonance_haskell_compute', true).
capability_valid('capa_005_metatron_haskell_seal', true).
capability_valid('capa_006_phantom_rust_log', false).  % Expired

% capability_revoked(CapabilityID, RevocationReason)
% Clause-based revocation state (can be extended with real reasons)
% Initially empty — no revocations recorded

% (none yet — all capabilities are active by default unless listed below)
% Add facts here if revocation occurs:
% capability_revoked('capa_NNN_...', 'reason_string').

% lease_active(LeaseID, IsActive)
% Each notebook execution creates a lease tied to a capability

lease_active('notebook-test-001', true).
lease_active('notebook-demo-002', true).
lease_active('notebook-demo-003', true).
