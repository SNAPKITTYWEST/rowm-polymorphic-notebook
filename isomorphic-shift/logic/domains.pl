%% ════════════════════════════════════════════════════════════════════════════════
%% Domain Definitions — Isomorphic Shift Logic Layer
%% SnapKitty Collective
%% ════════════════════════════════════════════════════════════════════════════════
%% Formal definitions of all domains, their constraints, and invariants.

:- module(domains, [
    domain_definition/3,
    domain_constraints/2,
    domain_invariants/2,
    domain_canonical_type/2,
    verify_domain_member/2
]).

%% ════════════════════════════════════════════════════════════════════════════════
%% DOMAIN DEFINITIONS
%% ════════════════════════════════════════════════════════════════════════════════
%% domain_definition(DomainName, Description, CanonicalTypeSignature)

domain_definition(
    'SurfaceInstruction',
    'User-facing instructions in various syntaxes (EmojiCode, HolyC, Python, JavaScript, Ada)',
    'surface_instruction(source_lang: atom, verb: atom, args: map)'
).

domain_definition(
    'CanonicalInstruction',
    'Normalized instruction in deterministic CBOR form',
    'canonical_instruction(verb: atom, args: list, required_perms: list, hash: blake3)'
).

domain_definition(
    'LogicTerm',
    'Prolog/Datalog term for logical reasoning',
    'logic_term(functor: atom, args: list) or logic_term(atomic_value)'
).

domain_definition(
    'AuthorizedLogicDecision',
    'Logic reasoning result with authorization proof',
    'authorized_decision(agent: atom, decision: logic_term, proof_id: hash, timestamp: u64)'
).

domain_definition(
    'RuntimeCommand',
    'Executable command for HolyC, Rust, Erlang, or LLVM',
    'runtime_command(target: atom, opcode: atom, operands: list, required_context: map)'
).

domain_definition(
    'ProofObligation',
    'Formal proof goal in Lean 4 or Agda',
    'proof_obligation(theorem_name: atom, hypotheses: list, goal: formula, required_by: atom)'
).

domain_definition(
    'VerifierInvocation',
    'Invocation of formal verifier with result',
    'verifier_invocation(proof_id: hash, verifier: atom, status: atom, proof_code: string, timestamp: u64)'
).

domain_definition(
    'ExecutionEvent',
    'Runtime event: state transition, policy, output',
    'execution_event(event_id: hash, timestamp: u64, agent: atom, action: atom, payload: map, parent_receipt_hash: option(hash))'
).

domain_definition(
    'LogicEventFact',
    'Prolog fact representing a runtime event',
    'logic_event_fact(event_id: hash, agent: atom, action: atom, outcome: atom, worm_seal: hash)'
).

domain_definition(
    'ReceiptRecord',
    'Immutable WORM-sealed proof of execution and authorization',
    'receipt_record(receipt_id: hash, event_id: hash, authorization_proof: hash, verification_status: atom, worm_seal: hash, timestamp: u64)'
).

domain_definition(
    'NotebookCellRecord',
    'Jupyter/IPython notebook cell with metadata',
    'notebook_cell(cell_id: hash, cell_type: atom, source: string, outputs: list, execution_count: int, metadata: map)'
).

domain_definition(
    'LogicCellFact',
    'Prolog fact encoding notebook cell semantics',
    'logic_cell_fact(cell_id: hash, cell_type: atom, instructions_extracted: list, proof_obligations: list, worm_seal: hash)'
).

domain_definition(
    'RuntimeSpecificValue',
    'Language-specific values (integers, floats, strings, arrays, capabilities)',
    'runtime_value(type: atom, value: any, runtime_context: map)'
).

domain_definition(
    'CanonicalValue',
    'Language-neutral canonical representation',
    'canonical_value(type: atom, value: cbor_bytes, type_tag: atom)'
).

%% ════════════════════════════════════════════════════════════════════════════════
%% DOMAIN CONSTRAINTS
%% ════════════════════════════════════════════════════════════════════════════════
%% domain_constraints(DomainName, ConstraintList)

domain_constraints('SurfaceInstruction', [
    'source_lang in {emoji, holyc, python, javascript, ada}',
    'verb is non-empty atom',
    'args is valid map for the language',
    'no shell metacharacters in any string argument',
    'syntax must parse in source language'
]).

domain_constraints('CanonicalInstruction', [
    'verb is normalized atom (lowercase, no whitespace)',
    'args is ordered list (not map) in canonical order',
    'required_perms lists all capabilities needed',
    'hash is deterministic Blake3 of CBOR encoding',
    'CBOR encoding must be canonical (RFC 7049 Section 3.9)'
]).

domain_constraints('LogicTerm', [
    'functor is valid atom (letters, digits, underscore)',
    'args is list of logic_terms (recursive)',
    'no free variables (all instantiated or explicitly quantified)',
    'term must be ground (no unbound logic variables)'
]).

domain_constraints('AuthorizedLogicDecision', [
    'agent is known agent class',
    'decision is valid logic_term',
    'proof_id is Blake3 hash of Prolog query trace',
    'timestamp is Unix seconds',
    'agent must have required trust level'
]).

domain_constraints('RuntimeCommand', [
    'target in {holyc, rust, erlang, llvm, ada}',
    'opcode is valid instruction mnemonic for target',
    'operands match opcode signature',
    'required_context specifies heap/stack/permissions needed',
    'no undefined memory access patterns'
]).

domain_constraints('ProofObligation', [
    'theorem_name is unique identifier',
    'hypotheses is list of formulas',
    'goal is well-formed formula in proof system',
    'required_by identifies which shift/component needs this proof',
    'formula syntax must be valid Lean 4 or Agda'
]).

domain_constraints('VerifierInvocation', [
    'proof_id is Blake3 hash of obligation',
    'verifier in {lean4, agda}',
    'status in {pending, verified, failed, timeout}',
    'proof_code is valid Lean 4 or Agda code',
    'timestamp is Unix seconds'
]).

domain_constraints('ExecutionEvent', [
    'event_id is Blake3 hash (deterministic from content)',
    'timestamp is Unix seconds',
    'agent is known agent identifier',
    'action is valid operation name',
    'payload must match action schema',
    'parent_receipt_hash links to ancestor event'
]).

domain_constraints('LogicEventFact', [
    'event_id is Blake3 hash of execution event',
    'agent is known agent',
    'action functor matches execution event',
    'outcome in {success, blocked, failed, timeout}',
    'worm_seal is immutable WORM anchor'
]).

domain_constraints('ReceiptRecord', [
    'receipt_id is Blake3 hash (deterministic)',
    'event_id references execution event',
    'authorization_proof is hash of Prolog query result',
    'verification_status in {authorized, provisional, denied}',
    'worm_seal is immutable',
    'timestamp is Unix seconds'
]).

domain_constraints('NotebookCellRecord', [
    'cell_id is Blake3 hash of (source, execution_count)',
    'cell_type in {code, markdown, raw}',
    'source is non-empty string',
    'outputs list contains cell outputs',
    'execution_count >= 0',
    'metadata contains notebook-specific data'
]).

domain_constraints('LogicCellFact', [
    'cell_id matches notebook cell ID',
    'cell_type from notebook',
    'instructions_extracted is list of logic_terms',
    'proof_obligations is list of proof_obligation terms',
    'worm_seal is immutable'
]).

domain_constraints('RuntimeSpecificValue', [
    'type in {i32, i64, u32, u64, f32, f64, f128, bool, string, bytes, array, map, capability}',
    'value matches type',
    'f32/f64/f128 must be finite (±inf, NaN explicitly marked)',
    'array has explicit rank/shape',
    'capability includes scope and issuer',
    'runtime_context specifies source runtime'
]).

domain_constraints('CanonicalValue', [
    'type in {integer, rational, float, boolean, atom, string, bytes, list, map, tagged_union, capability_ref, hash_ref}',
    'value is deterministic CBOR (RFC 7049)',
    'type_tag provides type information for deserializer',
    'floats marked as such (not ambiguous with integers)',
    'arrays have explicit shape/rank in type_tag'
]).

%% ════════════════════════════════════════════════════════════════════════════════
%% DOMAIN INVARIANTS
%% ════════════════════════════════════════════════════════════════════════════════
%% domain_invariants(DomainName, InvariantList)

domain_invariants('SurfaceInstruction', [
    'verb_identity — verb unchanged through round-trip',
    'argument_identity — argument list structure preserved',
    'type_preservation — argument types unchanged',
    'authorization_identity — required permissions unchanged'
]).

domain_invariants('CanonicalInstruction', [
    'verb_identity — same verb as source',
    'argument_identity — same arguments (reordered canonically)',
    'type_preservation — all types maintained',
    'determinism — same input always produces same hash'
]).

domain_invariants('LogicTerm', [
    'functor_identity — same functor as instruction verb',
    'argument_identity — same argument count and structure',
    'grounding_identity — all variables instantiated',
    'type_correspondence — types correspond to Prolog types'
]).

domain_invariants('AuthorizedLogicDecision', [
    'agent_identity — agent unchanged',
    'decision_identity — decision logic unchanged',
    'proof_integrity — proof_id matches query execution',
    'no_permission_increase — authorization not escalated'
]).

domain_invariants('RuntimeCommand', [
    'semantics_preservation — runtime behavior matches decision semantics',
    'target_executability — command is valid on target',
    'safety_preservation — no unsafe operations not approved by decision'
]).

domain_invariants('ProofObligation', [
    'theorem_identity — same theorem throughout pipeline',
    'hypothesis_preservation — assumptions unchanged',
    'goal_preservation — statement unchanged'
]).

domain_invariants('VerifierInvocation', [
    'obligation_correspondence — proof_id matches obligation',
    'status_integrity — status reflects actual verification result',
    'proof_code_identity — proof unchanged'
]).

domain_invariants('ExecutionEvent', [
    'causality_preservation — parent_receipt_hash maintains event order',
    'action_identity — action type unchanged',
    'agent_identity — agent unchanged',
    'timestamp_immutability — timestamp never modified'
]).

domain_invariants('LogicEventFact', [
    'event_correspondence — fact describes the same event',
    'outcome_integrity — outcome reflects actual execution',
    'worm_immutability — seal is permanent'
]).

domain_invariants('ReceiptRecord', [
    'event_ancestry — receipt_id traces to original event',
    'authorization_integrity — proof matches authorization given',
    'worm_immutability — seal never changes',
    'timestamp_immutability — timestamp locked at creation'
]).

domain_invariants('NotebookCellRecord', [
    'source_identity — source code unchanged',
    'output_sealing — outputs sealed once execution complete',
    'execution_order — execution_count reflects sequence'
]).

domain_invariants('LogicCellFact', [
    'cell_correspondence — fact describes same cell',
    'instruction_extraction_completeness — all executable instructions extracted',
    'worm_immutability — seal permanent'
]).

domain_invariants('RuntimeSpecificValue', [
    'type_preservation — type unchanged',
    'value_preservation — value unchanged',
    'no_precision_loss — no implicit narrowing',
    'no_capability_escalation — scope unchanged'
]).

domain_invariants('CanonicalValue', [
    'type_preservation — runtime type maps to canonical type',
    'determinism — same input always produces same CBOR',
    'no_precision_loss — exact representation in rational/big-int if needed'
]).

%% ════════════════════════════════════════════════════════════════════════════════
%% CANONICAL TYPE MAPPING
%% ════════════════════════════════════════════════════════════════════════════════

domain_canonical_type('SurfaceInstruction', 'surface_instruction').
domain_canonical_type('CanonicalInstruction', 'canonical_instruction').
domain_canonical_type('LogicTerm', 'logic_term').
domain_canonical_type('AuthorizedLogicDecision', 'authorized_decision').
domain_canonical_type('RuntimeCommand', 'runtime_command').
domain_canonical_type('ProofObligation', 'proof_obligation').
domain_canonical_type('VerifierInvocation', 'verifier_invocation').
domain_canonical_type('ExecutionEvent', 'execution_event').
domain_canonical_type('LogicEventFact', 'logic_event_fact').
domain_canonical_type('ReceiptRecord', 'receipt_record').
domain_canonical_type('NotebookCellRecord', 'notebook_cell').
domain_canonical_type('LogicCellFact', 'logic_cell_fact').
domain_canonical_type('RuntimeSpecificValue', 'runtime_value').
domain_canonical_type('CanonicalValue', 'canonical_value').

%% ════════════════════════════════════════════════════════════════════════════════
%% DOMAIN MEMBERSHIP VERIFICATION
%% ════════════════════════════════════════════════════════════════════════════════

%% verify_domain_member(+Domain, +Value) — True if Value is member of Domain
%% This is a placeholder for adapter-level implementation
verify_domain_member(Domain, _Value) :-
    domain_definition(Domain, _Description, _TypeSig).

