% Rules: Authorization and Capability Checking
% Core authorization logic for dispatch decisions

:- module(authorization, [
    capability_active/2,
    dispatch_permitted/5,
    agent_authorized/3,
    capability_satisfies/4
]).

:- use_module(agents).
:- use_module(capabilities).
:- use_module(runtimes).

% capability_active(CapabilityID, IsActive)
% A capability is active if issued, not revoked, and not expired
% Expiration boundary: Timestamp < ExpiresAt (exclusive, not inclusive)
capability_active(CapID, true) :-
    capability_issued(CapID, _, _, _, _, _IssuedAt, ExpiresAt),
    \+ capability_revoked(CapID, _),
    get_time(Now),
    Timestamp is floor(Now),
    Timestamp < ExpiresAt.

capability_active(_, false).

% agent_authorized(AgentID, CapabilityID, TargetRuntime)
% An agent is authorized if it holds a valid capability for the target runtime
agent_authorized(AgentID, CapID, TargetRuntime) :-
    agent_active(AgentID, true),
    agent_trust_level(AgentID, TrustLevel),
    TrustLevel \= tier_2,  % tier_2 agents (observers) cannot dispatch
    capability_issued(CapID, _IssuerID, AgentID, TargetRuntime, _Perms, _IssuedAt, _ExpiresAt),
    capability_active(CapID, true).

% capability_satisfies(CapID, Permission, TargetRuntime, Satisfies)
% Check if a capability grants a specific permission on a specific runtime
capability_satisfies(CapID, Permission, TargetRuntime, true) :-
    capability_issued(CapID, _, _, TargetRuntime, Permissions, _, _),
    member(Permission, Permissions),
    capability_active(CapID, true).

capability_satisfies(_, _, _, false).

% dispatch_permitted(AgentID, CapabilityID, TargetRuntime, Permission, IsPermitted)
% Complete authorization check: agent + capability + runtime + permission
dispatch_permitted(AgentID, CapID, TargetRuntime, Permission, true) :-
    agent_authorized(AgentID, CapID, TargetRuntime),
    capability_satisfies(CapID, Permission, TargetRuntime, true),
    runtime_active(TargetRuntime, true).

dispatch_permitted(_, _, _, _, false).

% Helper: get_time/1 — Unix timestamp (system time for real enforcement)
% For deterministic testing, this can be overridden via test-specific assertions
% In production, this is the actual system clock
:- use_module(library(system)).
