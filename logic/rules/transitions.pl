% Rules: State Transition Validation
% Ensures LOC triad follows valid execution paths

:- module(transitions, [
    transition_valid/3,
    state_reachable/3,
    stage_prerequisite/2,
    next_stage/2
]).

:- use_module(agents).
:- use_module(runtimes).
:- use_module(authorization).
:- use_module(receipts).

% Valid state machine transitions for notebook execution
% RECEIVE → TRANSLATE → VERIFY → DISPATCH → EXECUTE → ENCODE → SEAL → COMPLETE

% next_stage(CurrentStage, NextStage)
next_stage(receive, translate).
next_stage(translate, verify).
next_stage(verify, dispatch).
next_stage(dispatch, execute).
next_stage(execute, encode).
next_stage(encode, seal).
next_stage(seal, complete).

% stage_prerequisite(Stage, PrerequisiteStage)
% What must complete before this stage
stage_prerequisite(translate, receive).
stage_prerequisite(verify, translate).
stage_prerequisite(dispatch, verify).
stage_prerequisite(execute, dispatch).
stage_prerequisite(encode, execute).
stage_prerequisite(seal, encode).
stage_prerequisite(complete, seal).

% transition_valid(FromStage, ToStage, IsValid)
% Only valid transitions are those defined by next_stage/2
transition_valid(FromStage, ToStage, true) :-
    next_stage(FromStage, ToStage).

transition_valid(_, _, false).

% state_reachable(Stage, FromState, IsReachable)
% Recursive path check: is this state reachable from initial state?
state_reachable(receive, initial, true).

state_reachable(Stage, FromState, true) :-
    Stage \= receive,
    stage_prerequisite(Stage, PrevStage),
    state_reachable(PrevStage, FromState, true).

state_reachable(_, _, false).

% Inline path validation for LOC triad
% Each stage is bound to specific agents and runtimes

stage_agent_binding(receive, loc).
stage_agent_binding(translate, resonance).      % EmojiCode translation
stage_agent_binding(verify, sentinel).          % Ada proof verification
stage_agent_binding(dispatch, loc).             % LOC dispatch
stage_agent_binding(execute, forge).            % Runtime execution
stage_agent_binding(encode, resonance).         % Output encoding
stage_agent_binding(seal, metatron).            % WORM sealing
stage_agent_binding(complete, metatron).        % Finalization

% Stage capability requirements
stage_capability_requirement(receive, none).
stage_capability_requirement(translate, compute).
stage_capability_requirement(verify, audit).
stage_capability_requirement(dispatch, dispatch).
stage_capability_requirement(execute, execute).
stage_capability_requirement(encode, synthesize).
stage_capability_requirement(seal, worm_seal).
stage_capability_requirement(complete, finalize).

% Stage runtime targets
stage_runtime_target(receive, rust).
stage_runtime_target(translate, emoji).
stage_runtime_target(verify, ada).
stage_runtime_target(dispatch, rust).
stage_runtime_target(execute, holyc).
stage_runtime_target(encode, emoji).
stage_runtime_target(seal, haskell).
stage_runtime_target(complete, haskell).
