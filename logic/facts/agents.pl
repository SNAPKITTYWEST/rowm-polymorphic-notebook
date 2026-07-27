% Facts: Agent Registry and Identity
% LOGIC-FOUNDRY Sovereign Agent Directive

:- module(agents, [
    agent_registered/2,
    agent_active/2,
    agent_capability/3,
    agent_trust_level/2
]).

% agent_registered(AgentID, Metadata)
% Metadata: atom_record(name, role, origin_kernel, trust_tier)

agent_registered(loc,       atom_record('LOC Executor', executor, rust, tier_1)).
agent_registered(forge,     atom_record('Forge Builder', adapter, rust, tier_1)).
agent_registered(sentinel,  atom_record('Sentinel Auditor', verifier, haskell, tier_1)).
agent_registered(cipher,    atom_record('Cipher Crypto', adapter, rust, tier_0)).
agent_registered(metatron,  atom_record('Metatron Seal', validator, haskell, tier_1)).
agent_registered(phantom,   atom_record('Phantom Logger', observer, rust, tier_2)).
agent_registered(resonance, atom_record('Resonance Math', compute, haskell, tier_1)).

% agent_active(AgentID, IsActive)
% An agent is active if it has a valid registration and has not been revoked

agent_active(loc, true).
agent_active(forge, true).
agent_active(sentinel, true).
agent_active(cipher, true).
agent_active(metatron, true).
agent_active(phantom, true).
agent_active(resonance, true).

% agent_capability(AgentID, Capability, SupportedRuntime)
% Maps which capabilities each agent can exercise on which runtimes

agent_capability(loc, dispatch, rust).
agent_capability(loc, dispatch, emoji).
agent_capability(loc, execute, holyc).
agent_capability(loc, orchestrate, ada).

agent_capability(forge, build, rust).
agent_capability(forge, verify, rust).
agent_capability(forge, emit_artifact, rust).

agent_capability(sentinel, audit, haskell).
agent_capability(sentinel, verify, ada).
agent_capability(sentinel, verify, haskell).
agent_capability(sentinel, attest, haskell).

agent_capability(cipher, seal, rust).
agent_capability(cipher, sign, rust).
agent_capability(cipher, revoke, rust).

agent_capability(metatron, finalize, haskell).
agent_capability(metatron, worm_seal, haskell).
agent_capability(metatron, meta_verify, haskell).

agent_capability(phantom, log, rust).
agent_capability(phantom, observe, rust).
agent_capability(phantom, collect_metrics, rust).

agent_capability(resonance, compute, haskell).
agent_capability(resonance, synthesize, haskell).
agent_capability(resonance, verify_math, haskell).

% agent_trust_level(AgentID, TrustTier)
% tier_0: Crypto material (cipher, key-holder)
% tier_1: Trusted core execution (loc, forge, sentinel, metatron, resonance)
% tier_2: Observational (phantom, logging)

agent_trust_level(loc, tier_1).
agent_trust_level(forge, tier_1).
agent_trust_level(sentinel, tier_1).
agent_trust_level(cipher, tier_0).
agent_trust_level(metatron, tier_1).
agent_trust_level(phantom, tier_2).
agent_trust_level(resonance, tier_1).
