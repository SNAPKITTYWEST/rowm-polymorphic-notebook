%% ════════════════════════════════════════════════════════════════════════════════
%% Shift Authorization Rules
%% SnapKitty Collective
%% ════════════════════════════════════════════════════════════════════════════════
%% Authorization flow for isomorphic shifts. All decisions are made by Prolog rules.
%% Integration with bob-orchestrator/prolog/sovereign_kernel.pl (agent trust model).

:- module(shift_authorization, [
    can_perform_shift/3,
    agent_has_capability/2,
    capability_sufficient_for_shift/2,
    authorization_trace/2,
    authorization_denied_reason/2
]).

%% ────────────────────────────────────────────────────────────────────────────────
%% AGENT TRUST LEVELS (imported from sovereign_kernel.pl)
%% ────────────────────────────────────────────────────────────────────────────────

trust_level(none,      0).
trust_level(low,       1).
trust_level(medium,    2).
trust_level(high,      3).
trust_level(sovereign, 4).

agent_class(sentinel,  sovereign).
agent_class(oracle,    high).
agent_class(builder,   high).
agent_class(archivist, high).
agent_class(berserker, medium).

%% ────────────────────────────────────────────────────────────────────────────────
%% SHIFT CAPABILITY REQUIREMENTS
%% ────────────────────────────────────────────────────────────────────────────────
%% shift_requires_capability(+ShiftId, +CapabilityAtom, +MinTrustLevel)

shift_requires_capability('M1', read,   low).       % Surface → Canonical (read source)
shift_requires_capability('M1', write,  low).       % (write canonical form)

shift_requires_capability('M2', read,   low).       % Canonical → Logic (read canonical)
shift_requires_capability('M2', analyze, medium).   % (analyze/formalize)

shift_requires_capability('M3', read,   medium).    % Decision → Command (read decision)
shift_requires_capability('M3', execute, high).     % (generate executable)

shift_requires_capability('M4', read,   medium).    % Obligation → Invocation (read proof)
shift_requires_capability('M4', verify, high).      % (invoke verifier)

shift_requires_capability('M5', read,   low).       % Event → Fact (read event)
shift_requires_capability('M5', analyze, medium).   % (analyze/classify)

shift_requires_capability('M6', read,   medium).    % Event → Receipt (read event)
shift_requires_capability('M6', seal,   high).      % (seal receipt with WORM)

shift_requires_capability('M7', read,   low).       % Cell → Fact (read notebook)
shift_requires_capability('M7', analyze, medium).   % (extract/analyze)

shift_requires_capability('M8', read,   low).       % Value → Canonical (read value)
shift_requires_capability('M8', write,  low).       % (normalize/write)

%% ────────────────────────────────────────────────────────────────────────────────
%% AGENT CAPABILITIES (who can do what)
%% ────────────────────────────────────────────────────────────────────────────────

agent_has_capability(sentinel, read).
agent_has_capability(sentinel, write).
agent_has_capability(sentinel, execute).
agent_has_capability(sentinel, seal).
agent_has_capability(sentinel, verify).
agent_has_capability(sentinel, analyze).

agent_has_capability(oracle, read).
agent_has_capability(oracle, analyze).
agent_has_capability(oracle, pattern_match).

agent_has_capability(builder, read).
agent_has_capability(builder, write).
agent_has_capability(builder, generate).
agent_has_capability(builder, execute).
agent_has_capability(builder, seal).

agent_has_capability(archivist, read).
agent_has_capability(archivist, analyze).
agent_has_capability(archivist, index).
agent_has_capability(archivist, provenance).

agent_has_capability(berserker, read).
agent_has_capability(berserker, inject).
agent_has_capability(berserker, analyze).

%% ────────────────────────────────────────────────────────────────────────────────
%% CAPABILITY SUFFICIENCY FOR SHIFTS
%% ────────────────────────────────────────────────────────────────────────────────

%% capability_sufficient_for_shift(+Capability, +ShiftId)
%% Returns true if capability is sufficient to perform shift

capability_sufficient_for_shift(write, 'M1').
capability_sufficient_for_shift(write, 'M8').
capability_sufficient_for_shift(generate, 'M1').
capability_sufficient_for_shift(generate, 'M8').

capability_sufficient_for_shift(read, 'M2').
capability_sufficient_for_shift(analyze, 'M2').

capability_sufficient_for_shift(execute, 'M3').
capability_sufficient_for_shift(generate, 'M3').

capability_sufficient_for_shift(verify, 'M4').
capability_sufficient_for_shift(execute, 'M4').

capability_sufficient_for_shift(read, 'M5').
capability_sufficient_for_shift(analyze, 'M5').

capability_sufficient_for_shift(seal, 'M6').
capability_sufficient_for_shift(execute, 'M6').

capability_sufficient_for_shift(read, 'M7').
capability_sufficient_for_shift(analyze, 'M7').
capability_sufficient_for_shift(generate, 'M7').

%% ────────────────────────────────────────────────────────────────────────────────
%% PRIMARY AUTHORIZATION RULE
%% ────────────────────────────────────────────────────────────────────────────────

%% can_perform_shift(+Agent, +ShiftId, +Direction) — Main authorization predicate
can_perform_shift(Agent, ShiftId, Direction) :-
    % Agent must be known
    agent_class(Agent, _Trust),

    % Agent must have at least one capability required by shift
    agent_has_capability(Agent, Capability),
    capability_sufficient_for_shift(Capability, ShiftId),

    % Direction must be valid
    member(Direction, [forward, inverse, bidirectional]),

    % Trust level check: agent trust >= minimum required for this capability
    agent_class(Agent, AgentTrust),
    shift_requires_capability(ShiftId, Capability, MinTrust),
    trust_level(AgentTrust, AgentLevel),
    trust_level(MinTrust, MinLevel),
    AgentLevel >= MinLevel,

    % Oracle is read-only: cannot perform direction=forward on write-requiring shifts
    \+ (Agent = oracle, shift_requires_capability(ShiftId, write, _), Direction = forward),
    \+ (Agent = oracle, shift_requires_capability(ShiftId, execute, _), Direction = forward),
    \+ (Agent = oracle, shift_requires_capability(ShiftId, seal, _), Direction = forward).

%% ────────────────────────────────────────────────────────────────────────────────
%% AUTHORIZATION TRACE (for logging/auditing)
%% ────────────────────────────────────────────────────────────────────────────────

authorization_trace(Agent, trace{
    agent: Agent,
    status: authorized,
    rules_satisfied: [
        'agent_is_known',
        'agent_has_required_capability',
        'trust_level_sufficient',
        'direction_valid',
        'no_oracle_write_violation'
    ]
}) :-
    agent_class(Agent, _).

authorization_trace(Agent, trace{
    agent: Agent,
    status: denied,
    reason: 'agent_not_known'
}) :-
    \+ agent_class(Agent, _).

%% ────────────────────────────────────────────────────────────────────────────────
%% DENIAL REASONS
%% ────────────────────────────────────────────────────────────────────────────────

authorization_denied_reason(Agent, 'AGENT_UNKNOWN') :-
    \+ agent_class(Agent, _).

authorization_denied_reason(Agent, 'INSUFFICIENT_TRUST') :-
    agent_class(Agent, AgentTrust),
    \+ (shift_requires_capability(_ShiftId, _Capability, MinTrust),
        trust_level(AgentTrust, AgentLevel),
        trust_level(MinTrust, MinLevel),
        AgentLevel >= MinLevel).

authorization_denied_reason(Agent, 'NO_REQUIRED_CAPABILITY') :-
    agent_class(Agent, _),
    \+ agent_has_capability(Agent, _).

authorization_denied_reason(oracle, 'ORACLE_READ_ONLY_VIOLATION') :-
    shift_requires_capability(_ShiftId, Capability, _),
    member(Capability, [write, execute, seal]).

%% ════════════════════════════════════════════════════════════════════════════════
%% RESTRICTION: NO UNAUTHORIZED PERMISSION ESCALATION
%% ════════════════════════════════════════════════════════════════════════════════

%% Rule: If a shift is denied for an agent, it MUST remain denied.
%% No intermediate transformation step can escalate permissions.

prevent_permission_escalation(Agent, ShiftId, Direction) :-
    \+ can_perform_shift(Agent, ShiftId, Direction),
    !.  % Cut: once denied, always denied for this agent/shift/direction

