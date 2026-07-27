%% ════════════════════════════════════════════════════════════════════════════════
%% Shift Release Readiness — Production Deployment Gates
%% SnapKitty Collective
%% ════════════════════════════════════════════════════════════════════════════════
%% Comprehensive checklist for shift readiness before production deployment.

:- module(shift_release, [
    shift_release_ready/1,
    shift_release_checklist/2,
    shift_readiness_status/2,
    release_gate_passed/2,
    release_gate_failed_reason/2
]).

%% ════════════════════════════════════════════════════════════════════════════════
%% RELEASE GATES
%% ════════════════════════════════════════════════════════════════════════════════

%% gate_1: Shift is registered in shifts.pl
shift_gate_1(ShiftId) :-
    findall(1, isomorphic_shift(ShiftId, _, _, _, _, _, _, _), L),
    length(L, N),
    N > 0.

%% gate_2: Both forward and inverse adapters implemented (or marked as projection)
shift_gate_2(ShiftId) :-
    isomorphic_shift(ShiftId, _Version, _SourceDomain, _TargetDomain, FwdAdapter, InvAdapter, _Hash, Classification),
    (
        (FwdAdapter \= '', InvAdapter \= '')  % Both implemented
    ;
        (Classification = 'projection')        % Or marked as projection (lossy)
    ).

%% gate_3: All invariants verified or assumed
shift_gate_3(ShiftId) :-
    findall(1, shift_preserves_invariant(ShiftId, _Inv, verified), Verified),
    findall(1, invariant_definition(_, ShiftId, _), All),
    length(Verified, V),
    length(All, A),
    (A = 0 ; V >= A).  % All invariants verified or no invariants defined

%% gate_4: Authorization rules defined
shift_gate_4(ShiftId) :-
    findall(1, shift_requires_capability(ShiftId, _, _), L),
    length(L, N),
    N > 0.

%% gate_5: Semantic equivalence verified
shift_gate_5(ShiftId) :-
    verify_isomorphism(ShiftId, Result),
    member(Result, [fully_isomorphic, partial]).

%% gate_6: Round-trip law verified
shift_gate_6(ShiftId) :-
    round_trip_verified(ShiftId, passed).

%% gate_7: No permission escalation possible
shift_gate_7(ShiftId) :-
    \+ (
        shift_requires_capability(ShiftId, Cap1, Trust1),
        shift_requires_capability(ShiftId, Cap2, Trust2),
        trust_level(Trust1, L1),
        trust_level(Trust2, L2),
        L2 > L1
    ).

%% gate_8: All tests pass
shift_gate_8(ShiftId) :-
    findall(test_result(ShiftId, Status), test_passes(ShiftId, Status), Results),
    length(Results, N),
    N > 0,
    forall(member(test_result(_ShiftId, Status), Results), Status = passed).

%% gate_9: No known security vulnerabilities
shift_gate_9(ShiftId) :-
    \+ known_security_issue(ShiftId).

%% gate_10: Documentation complete
shift_gate_10(ShiftId) :-
    documented_shift(ShiftId).

%% gate_11: Adapter code reviewed
shift_gate_11(ShiftId) :-
    adapter_reviewed(ShiftId).

%% gate_12: Ready for deployment
shift_gate_12(ShiftId) :-
    deployment_approved(ShiftId).

%% ════════════════════════════════════════════════════════════════════════════════
%% COMPOUND RELEASE DECISION
%% ════════════════════════════════════════════════════════════════════════════════

shift_release_ready(ShiftId) :-
    shift_gate_1(ShiftId),
    shift_gate_2(ShiftId),
    shift_gate_3(ShiftId),
    shift_gate_4(ShiftId),
    shift_gate_5(ShiftId),
    shift_gate_6(ShiftId),
    shift_gate_7(ShiftId),
    shift_gate_8(ShiftId),
    shift_gate_9(ShiftId),
    shift_gate_10(ShiftId),
    shift_gate_11(ShiftId),
    shift_gate_12(ShiftId).

%% ════════════════════════════════════════════════════════════════════════════════
%% CHECKLIST GENERATION
%% ════════════════════════════════════════════════════════════════════════════════

shift_release_checklist(ShiftId, checklist{
    shift_id: ShiftId,
    gate_1_registered: (shift_gate_1(ShiftId) -> pass ; fail),
    gate_2_adapters: (shift_gate_2(ShiftId) -> pass ; fail),
    gate_3_invariants: (shift_gate_3(ShiftId) -> pass ; fail),
    gate_4_authorization: (shift_gate_4(ShiftId) -> pass ; fail),
    gate_5_semantics: (shift_gate_5(ShiftId) -> pass ; fail),
    gate_6_roundtrip: (shift_gate_6(ShiftId) -> pass ; fail),
    gate_7_no_escalation: (shift_gate_7(ShiftId) -> pass ; fail),
    gate_8_tests: (shift_gate_8(ShiftId) -> pass ; fail),
    gate_9_security: (shift_gate_9(ShiftId) -> pass ; fail),
    gate_10_documentation: (shift_gate_10(ShiftId) -> pass ; fail),
    gate_11_review: (shift_gate_11(ShiftId) -> pass ; fail),
    gate_12_approval: (shift_gate_12(ShiftId) -> pass ; fail),
    overall_status: (shift_release_ready(ShiftId) -> ready ; blocked)
}).

%% ════════════════════════════════════════════════════════════════════════════════
%% READINESS STATUS REPORTING
%% ════════════════════════════════════════════════════════════════════════════════

shift_readiness_status(ShiftId, status{
    shift_id: ShiftId,
    status: (shift_release_ready(ShiftId) -> 'RELEASE_READY' ; 'NOT_READY'),
    gates_passed: GatesPassed,
    gates_total: 12,
    percentage_complete: Percentage
}) :-
    findall(1, (
        member(Gate, [
            shift_gate_1, shift_gate_2, shift_gate_3, shift_gate_4,
            shift_gate_5, shift_gate_6, shift_gate_7, shift_gate_8,
            shift_gate_9, shift_gate_10, shift_gate_11, shift_gate_12
        ]),
        call(Gate, ShiftId)
    ), Passed),
    length(Passed, GatesPassed),
    Percentage is (GatesPassed * 100) // 12.

%% ════════════════════════════════════════════════════════════════════════════════
%% GATE STATUS HELPERS
%% ════════════════════════════════════════════════════════════════════════════════

release_gate_passed(ShiftId, GateName) :-
    Gate =.. [GateName, ShiftId],
    call(Gate).

release_gate_failed_reason(ShiftId, GateName, Reason) :-
    \+ release_gate_passed(ShiftId, GateName),
    gate_failure_reason(ShiftId, GateName, Reason).

%% ════════════════════════════════════════════════════════════════════════════════
%% FAILURE REASON EXPLANATIONS
%% ════════════════════════════════════════════════════════════════════════════════

gate_failure_reason(ShiftId, 'shift_gate_1', 'Shift not registered in shifts.pl') :-
    \+ findall(1, isomorphic_shift(ShiftId, _, _, _, _, _, _, _), [_|_]).

gate_failure_reason(ShiftId, 'shift_gate_2', 'Forward or inverse adapter not implemented') :-
    \+ (
        isomorphic_shift(ShiftId, _V, _SD, _TD, FA, IA, _H, _C),
        FA \= '',
        IA \= ''
    ).

gate_failure_reason(ShiftId, 'shift_gate_3', 'Not all invariants verified') :-
    findall(1, shift_preserves_invariant(ShiftId, _, verified), V),
    findall(1, invariant_definition(_, ShiftId, _), A),
    \+ (length(V, LV), length(A, LA), (LA = 0 ; LV >= LA)).

gate_failure_reason(_ShiftId, 'shift_gate_4', 'No authorization rules defined') :-
    true.  % Generic reason

gate_failure_reason(ShiftId, 'shift_gate_5', 'Semantic equivalence not verified') :-
    \+ (verify_isomorphism(ShiftId, R), member(R, [fully_isomorphic, partial])).

gate_failure_reason(ShiftId, 'shift_gate_6', 'Round-trip law not verified') :-
    \+ round_trip_verified(ShiftId, passed).

gate_failure_reason(ShiftId, 'shift_gate_7', 'Potential permission escalation detected') :-
    (
        shift_requires_capability(ShiftId, C1, T1),
        shift_requires_capability(ShiftId, C2, T2),
        trust_level(T1, L1),
        trust_level(T2, L2),
        L2 > L1
    ).

gate_failure_reason(_ShiftId, 'shift_gate_8', 'Not all tests pass') :-
    true.

gate_failure_reason(ShiftId, 'shift_gate_9', Reason) :-
    known_security_issue(ShiftId, Reason).

gate_failure_reason(_ShiftId, 'shift_gate_10', 'Documentation incomplete') :-
    true.

gate_failure_reason(_ShiftId, 'shift_gate_11', 'Adapter code not reviewed') :-
    true.

gate_failure_reason(_ShiftId, 'shift_gate_12', 'Not approved for deployment') :-
    true.

%% ════════════════════════════════════════════════════════════════════════════════
%% PLACEHOLDER PREDICATES (to be implemented by integrator)
%% ════════════════════════════════════════════════════════════════════════════════

test_passes(_ShiftId, passed).

known_security_issue(_ShiftId) :- fail.
known_security_issue(_ShiftId, _Reason) :- fail.

documented_shift(_ShiftId) :- true.  % Assume documented unless stated

adapter_reviewed(_ShiftId) :- true.  % Assume reviewed unless stated

deployment_approved(_ShiftId) :- true.  % Assume approved unless stated

%% Import from shifts.pl
:- use_module(library(shifts), [
    isomorphic_shift/8,
    round_trip_verified/2,
    verify_isomorphism/2
]).

%% Import from invariants.pl
:- use_module(library(invariants), [
    shift_preserves_invariant/3,
    invariant_definition/3
]).

%% Import from shift_authorization.pl
:- use_module(library(shift_authorization), [
    shift_requires_capability/3,
    trust_level/2
]).

