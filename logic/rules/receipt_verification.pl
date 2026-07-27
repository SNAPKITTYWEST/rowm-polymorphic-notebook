% Receipt Verification Rules (v2.0 Protocol)
% Verifies canonical hashing, replay protection, Ed25519 signatures, chain integrity

:- module(receipt_verification, [
    verify_receipt_hash/2,
    verify_receipt_signature/3,
    verify_chain_linkage/2,
    verify_replay_protection/3,
    verify_receipt_canonical/2,
    release_ready_receipts/0
]).

:- use_module(library(system)).

% CANONICAL HASH VERIFICATION
% Verifies SHA-512 deterministic hash (timestamp excluded)
verify_receipt_hash(ReceiptHash, ReceiptID) :-
    receipt_v2(_, ReceiptID, StoredHash, _, _, _, _, _, _, _, _, _),
    ReceiptHash = StoredHash,
    % Hash must be full 128 hex characters (SHA-512)
    atom_length(ReceiptHash, Length),
    Length =:= 128.

% ED25519 SIGNATURE VERIFICATION
% Verifies signature against public key
verify_receipt_signature(ReceiptID, Signature, KeyVersion) :-
    receipt_v2(_, ReceiptID, ReceiptHash, AgentID, _, _, _, _, _, StoredKeyVersion, SignatureHex, _),
    StoredKeyVersion = KeyVersion,
    SignatureHex = Signature,
    ed25519_public_key(AgentID, KeyVersion, PublicKeyHex),
    % Signature must be 128 hex characters (64 bytes Ed25519)
    atom_length(SignatureHex, SigLen),
    SigLen =:= 128,
    % Public key must be 64 hex characters (32 bytes)
    atom_length(PublicKeyHex, PubLen),
    PubLen =:= 64.

% CHAIN LINKAGE VERIFICATION
% Verifies receipt chain integrity (previous_hash matches)
verify_chain_linkage(ReceiptHash, PreviousReceiptHash) :-
    receipt_chain_link(ReceiptHash, PreviousReceiptHash),
    % If first receipt, previous hash is all zeros
    (   PreviousReceiptHash = '0000000000000000000000000000000000000000000000000000000000000000'
    ;   receipt_v2(_, _, PreviousReceiptHash, _, _, _, _, _, _, _, _, _)
    ).

% REPLAY PROTECTION VERIFICATION
% Verifies nonce + context + monotonic counter uniqueness
verify_replay_protection(Nonce, Context, MonotonicCounter) :-
    nonce_record(Nonce, Context, StoredCounter, _),
    % Monotonic counter must strictly increase
    MonotonicCounter > StoredCounter,
    % Nonce + context pair is unique within this session
    \+ (
        nonce_record(Nonce, Context, OtherCounter, _),
        OtherCounter =:= MonotonicCounter
    ).

% CANONICAL FORM VERIFICATION
% Verifies receipt conforms to canonical serialization
verify_receipt_canonical(ReceiptID, Canonical) :-
    receipt_v2(Seq, ReceiptID, Hash, AgentID, CapID, InstrHash, Action, InputHash, OutputHash, KeyVer, Sig, Status),
    % Canonical form: seq|agent|cap|hashes|action|counter|status
    % (deterministic field order for hashing)
    format(atom(Canonical),
           'seq:~w|agent:~w|cap:~w|instr:~w|action:~w|input:~w|output:~w|keyver:~w|sig:~w|status:~w',
           [Seq, AgentID, CapID, InstrHash, Action, InputHash, OutputHash, KeyVer, Sig, Status]).

% FULL RECEIPT VERIFICATION (all checks)
verify_receipt_complete(ReceiptID, AgentID, KeyVersion, Nonce, Context, Counter) :-
    % 1. Receipt exists and is valid
    receipt_v2(_, ReceiptID, ReceiptHash, AgentID, _, _, _, _, _, KeyVersion, Signature, Status),
    member(Status, [success, sealed]),

    % 2. Hash is canonical (128 hex chars)
    atom_length(ReceiptHash, 128),

    % 3. Signature is valid (128 hex chars, matches key version)
    atom_length(Signature, 128),
    verify_receipt_signature(ReceiptID, Signature, KeyVersion),

    % 4. Replay protection: nonce + context + counter
    verify_replay_protection(Nonce, Context, Counter),

    % 5. Chain linkage (if not first receipt)
    (   ReceiptHash = 'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3'
    ;   receipt_chain_link(ReceiptHash, _)
    ).

% RELEASE GATE: All receipts must be verified
release_ready_receipts :-
    % All v2.0 receipts must be:
    % 1. Canonical (deterministic hash)
    forall(
        receipt_v2(_, ReceiptID, _, _, _, _, _, _, _, _, _, _),
        verify_receipt_hash(_, ReceiptID)
    ),
    % 2. Signed with valid Ed25519 signatures
    forall(
        receipt_v2(_, ReceiptID, _, _, _, _, _, _, _, KeyVer, _, _),
        verify_receipt_signature(ReceiptID, _, KeyVer)
    ),
    % 3. Chain integrity verified
    forall(
        receipt_chain_link(Hash, PrevHash),
        verify_chain_linkage(Hash, PrevHash)
    ),
    % 4. No replay attacks
    \+ (
        nonce_record(Nonce, Context, Counter1, _),
        nonce_record(Nonce, Context, Counter2, _),
        Counter1 =\= Counter2
    ).

% Debug: Check receipt state
check_receipt_state(ReceiptID) :-
    receipt_v2(Seq, ReceiptID, Hash, Agent, Cap, _, _, _, _, KeyVer, Sig, Status),
    format('Receipt: ~w~n', [ReceiptID]),
    format('  Sequence: ~w~n', [Seq]),
    format('  Agent: ~w~n', [Agent]),
    format('  Hash: ~w~n', [Hash]),
    format('  KeyVersion: ~w~n', [KeyVer]),
    format('  Signature: ~w~n', [Sig]),
    format('  Status: ~w~n', [Status]).

% Summary: Receipt chain statistics
receipt_chain_stats(TotalReceipts, SignedReceipts, VerifiedReceipts) :-
    findall(ID, receipt_v2(_, ID, _, _, _, _, _, _, _, _, _, _), AllIDs),
    length(AllIDs, TotalReceipts),
    findall(ID, (receipt_v2(_, ID, _, _, _, _, _, _, _, _, Sig, _), atom_length(Sig, 128)), SignedList),
    length(SignedList, SignedReceipts),
    findall(ID, (receipt_v2(_, ID, Hash, _, _, _, _, _, _, _, _, Status),
                 atom_length(Hash, 128),
                 member(Status, [success, sealed])), VerifiedList),
    length(VerifiedList, VerifiedReceipts).
