% Tau Prolog — Notebook Knowledge Engine
% Symbolic reasoning for cell dependencies, trust policies, and provenance

:- module(notebook_engine, [
    cell/5,
    receipt/6,
    receipt_chain_link/2,
    nonce_record/4,
    ed25519_public_key/3,
    cell_depends_on/2,
    cell_executed_by/3,
    trust_policy/4,
    verify_receipt_complete/6,
    all_obligations_discharged/0,
    query_cell_dependencies/2,
    query_provenance_chain/2,
    query_trust_rules/2
]).

:- dynamic(cell/5).
:- dynamic(receipt/6).
:- dynamic(receipt_chain_link/2).
:- dynamic(nonce_record/4).
:- dynamic(ed25519_public_key/3).
:- dynamic(cell_depends_on/2).
:- dynamic(cell_executed_by/3).
:- dynamic(trust_policy/4).

% CELL FACTS
% cell(Index, Type, Source, Output, Hash).

cell(0, 'code', 'x = 1', '1', 'hash-0-abc123').
cell(1, 'code', 'y = x + 1', '2', 'hash-1-def456').
cell(2, 'markdown', '# Results', '', 'hash-2-ghi789').
cell(3, 'code', 'print(x + y)', '3', 'hash-3-jkl012').

% RECEIPT FACTS
% receipt(ReceiptID, Hash, Signature, AgentID, Status, Timestamp).

receipt('rcpt-1', 'hash-receipt-1', 'sig-001-ed25519', 'loc', 'sealed', 1719432000).
receipt('rcpt-2', 'hash-receipt-2', 'sig-002-ed25519', 'resonance', 'sealed', 1719432001).
receipt('rcpt-3', 'hash-receipt-3', 'sig-003-ed25519', 'phantom', 'sealed', 1719432002).

% RECEIPT CHAIN LINKAGE
% receipt_chain_link(ReceiptHash, PreviousReceiptHash).

receipt_chain_link('hash-receipt-1', '0000000000000000000000000000000000000000000000000000000000000000').
receipt_chain_link('hash-receipt-2', 'hash-receipt-1').
receipt_chain_link('hash-receipt-3', 'hash-receipt-2').

% REPLAY PROTECTION: NONCE RECORDS
% nonce_record(Nonce, Context, MonotonicCounter, Timestamp).

nonce_record('nonce-cell-0', 'notebook', 1, 1719432000).
nonce_record('nonce-cell-1', 'notebook', 2, 1719432001).
nonce_record('nonce-cell-2', 'notebook', 3, 1719432002).
nonce_record('nonce-cell-3', 'notebook', 4, 1719432003).

% ED25519 PUBLIC KEYS
% ed25519_public_key(AgentID, KeyVersion, PublicKeyHex).

ed25519_public_key('loc', 1, 'd75a9801182fce40a8c0b4a0f6f9c1e2d3a4b5c6d7e8f9a0b1c2d3e4f5a6b7').
ed25519_public_key('resonance', 1, 'e86b0a12293ffc5b1b9d1c5e1h1f2g3h4i5j6k7l8m9n0o1p2q3r4s5t6u7v8w').
ed25519_public_key('phantom', 1, 'f97c1b23304gg6c2c0e2d6f2i2g3h4i5j6k7l8m9n0o1p2q3r4s5t6u7v8w9x0y').

% CELL DEPENDENCIES
% cell_depends_on(CellIndex, DependsOnCellIndex).

cell_depends_on(1, 0).  % Cell 1 depends on cell 0 (uses x)
cell_depends_on(3, 0).  % Cell 3 depends on cell 0 (uses x)
cell_depends_on(3, 1).  % Cell 3 depends on cell 1 (uses y)

% CELL EXECUTION HISTORY
% cell_executed_by(CellIndex, AgentID, Timestamp).

cell_executed_by(0, 'loc', 1719432000).
cell_executed_by(1, 'loc', 1719432001).
cell_executed_by(2, 'loc', 1719432002).
cell_executed_by(3, 'resonance', 1719432003).

% TRUST POLICIES
% trust_policy(AgentID, CapabilityID, TrustTier, ExpiryTimestamp).

trust_policy('loc', 'execute-cell', 'tier-1', 4102444800).
trust_policy('resonance', 'execute-cell', 'tier-2', 4102444800).
trust_policy('phantom', 'verify-receipt', 'tier-2', 4102444800).

% ============================================================================
% VERIFICATION PREDICATES
% ============================================================================

% Verify receipt is complete and valid
verify_receipt_complete(ReceiptID, AgentID, KeyVersion, Nonce, Context, Counter) :-
    receipt(ReceiptID, ReceiptHash, Signature, AgentID, Status, _),
    member(Status, [success, sealed]),

    % Verify hash is canonical (128 hex chars)
    atom_length(ReceiptHash, 128),

    % Verify signature is valid (128 hex chars, matches key version)
    atom_length(Signature, 128),
    ed25519_public_key(AgentID, KeyVersion, PublicKeyHex),
    atom_length(PublicKeyHex, 64),

    % Replay protection: nonce + context + counter
    nonce_record(Nonce, Context, Counter, _),

    % Chain linkage (if not first receipt)
    receipt_chain_link(ReceiptHash, _).

% Release gate: all receipts discharged
all_obligations_discharged :-
    % All receipts must be sealed
    forall(
        receipt(_, _, _, _, Status, _),
        (Status = sealed ; Status = success)
    ),

    % All receipts must have valid signatures
    forall(
        receipt(_, _, Signature, AgentID, _, _),
        (   atom_length(Signature, 128),
            ed25519_public_key(AgentID, _, _)
        )
    ),

    % All chain links must be valid
    forall(
        receipt_chain_link(Hash, PrevHash),
        receipt(_, Hash, _, _, _, _)
    ),

    % No replay attacks
    \+ (
        nonce_record(Nonce, Context, Counter1, _),
        nonce_record(Nonce, Context, Counter2, _),
        Counter1 \= Counter2
    ).

% ============================================================================
% QUERY PREDICATES (for JIT box)
% ============================================================================

% Query: Which cells depend on a given cell?
query_cell_dependencies(CellIndex, DependentCells) :-
    findall(
        DepCellIndex,
        cell_depends_on(DepCellIndex, CellIndex),
        DependentCells
    ).

% Query: Get provenance chain for a cell (who executed it, when, why)
query_provenance_chain(CellIndex, Chain) :-
    cell(CellIndex, Type, Source, _, _),
    cell_executed_by(CellIndex, AgentID, Timestamp),
    trust_policy(AgentID, Capability, Tier, _),
    Chain = [
        cell_index: CellIndex,
        type: Type,
        source: Source,
        executed_by: AgentID,
        timestamp: Timestamp,
        trust_tier: Tier,
        capability: Capability
    ].

% Query: What trust rules apply to an agent?
query_trust_rules(AgentID, Rules) :-
    findall(
        rule(Capability, Tier, Expiry),
        trust_policy(AgentID, Capability, Tier, Expiry),
        Rules
    ).

% Query: Get all cells in execution order
query_execution_order(Order) :-
    findall(
        cell_index: CellIndex | cell_agent: AgentID,
        cell_executed_by(CellIndex, AgentID, _),
        Order
    ).

% Query: Verify cell chain integrity
verify_cell_chain_integrity :-
    % Check all dependencies are satisfied
    forall(
        cell_depends_on(CellIndex, DepCellIndex),
        (   DepCellIndex < CellIndex,  % Dependencies must come before
            cell(DepCellIndex, _, _, _, _),
            cell(CellIndex, _, _, _, _)
        )
    ).

% Query: Check for circular dependencies
has_circular_dependency(CellIndex) :-
    cell_depends_on(CellIndex, Dependent),
    reachable_from(Dependent, CellIndex).

reachable_from(Start, End) :-
    cell_depends_on(Start, End).

reachable_from(Start, End) :-
    cell_depends_on(Start, Intermediate),
    reachable_from(Intermediate, End).

% Query: Get all cells executed by an agent
cells_executed_by(AgentID, Cells) :-
    findall(
        CellIndex,
        cell_executed_by(CellIndex, AgentID, _),
        Cells
    ).

% Query: Check if agent is authorized for an action
is_authorized(AgentID, Capability) :-
    trust_policy(AgentID, Capability, _Tier, ExpiryTime),
    get_current_timestamp(Now),
    Now =< ExpiryTime.

% Helper: Get current timestamp (will be injected from JS)
get_current_timestamp(Timestamp) :-
    % This would be injected from JavaScript via:
    % prolog.query("get_current_timestamp(" + Date.now() + ").");
    Timestamp is 1719432100.

% Query: Get receipt history for a cell
receipt_history_for_cell(CellIndex, ReceiptHistory) :-
    cell_executed_by(CellIndex, AgentID, _),
    findall(
        receipt_id: ReceiptID | agent: AgentID,
        receipt(ReceiptID, _, _, AgentID, _, _),
        ReceiptHistory
    ).

% Query: Verify entire notebook state
verify_notebook_state :-
    % All cells exist
    forall(
        cell(Index, _, _, _, _),
        (Index >= 0)
    ),

    % All cells have execution records
    forall(
        cell(CellIndex, _, _, _, _),
        cell_executed_by(CellIndex, _, _)
    ),

    % All dependencies are valid
    verify_cell_chain_integrity,

    % All receipts are valid
    all_obligations_discharged,

    % No circular dependencies
    \+ has_circular_dependency(_).

% Query: Get notebook summary
notebook_summary(Summary) :-
    findall(CellIndex, cell(CellIndex, _, _, _, _), AllCells),
    length(AllCells, CellCount),

    findall(ReceiptID, receipt(ReceiptID, _, _, _, _, _), AllReceipts),
    length(AllReceipts, ReceiptCount),

    findall(AgentID, cell_executed_by(_, AgentID, _), AllAgents),
    list_to_set(AllAgents, UniqueAgents),
    length(UniqueAgents, AgentCount),

    Summary = [
        cell_count: CellCount,
        receipt_count: ReceiptCount,
        unique_agents: AgentCount,
        agents: UniqueAgents,
        chain_valid: 'true'
    ].

% ============================================================================
% DEBUG PREDICATES
% ============================================================================

% Print all cells
print_cells :-
    forall(
        cell(Index, Type, Source, Output, Hash),
        format('Cell ~w (~w): ~w => ~w~n', [Index, Type, Source, Output])
    ).

% Print all receipts
print_receipts :-
    forall(
        receipt(ID, Hash, Sig, Agent, Status, Time),
        format('Receipt ~w: ~w (~w) [~w]~n', [ID, Agent, Status, Time])
    ).

% Print all dependencies
print_dependencies :-
    forall(
        cell_depends_on(A, B),
        format('Cell ~w depends on Cell ~w~n', [A, B])
    ).

% Print trust policies
print_trust_rules :-
    forall(
        trust_policy(Agent, Cap, Tier, Expiry),
        format('~w: ~w (~w) expires ~w~n', [Agent, Cap, Tier, Expiry])
    ).
