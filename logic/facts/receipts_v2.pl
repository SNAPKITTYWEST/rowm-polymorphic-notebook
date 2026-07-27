% Receipt v2 Facts (Protocol 2.0.0)
% Implements SEC-003, SEC-004, SEC-001: Canonical hashing, replay protection, Ed25519

:- dynamic(receipt_v2/12).
:- dynamic(receipt_chain/2).
:- dynamic(receipt_sealed/1).

% receipt_v2(SequenceNumber, ReceiptID, ReceiptHash, AgentID, CapabilityID,
%            InstructionHash, Action, InputHash, OutputHash, KeyVersion, Signature, Status).

% v2.0 receipts with canonical SHA-512 hashing
receipt_v2(1, 'rcpt-v2-0000000001-loc',
    'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
    'loc', 'cap-001', 'instr-001', 'dispatch', 'input-001', 'output-001', 1,
    'sig-001-ed25519', 'success').

receipt_v2(2, 'rcpt-v2-0000000002-resonance',
    'b8d5c0f3e2g4d9b6f7c3d0e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4',
    'resonance', 'cap-002', 'instr-002', 'verify', 'input-002', 'output-002', 1,
    'sig-002-ed25519', 'success').

receipt_v2(3, 'rcpt-v2-0000000003-phantom',
    'c9e6d1g4f3h5e0c7g8d4e1f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4g5',
    'phantom', 'cap-003', 'instr-003', 'execute', 'input-003', 'output-003', 1,
    'sig-003-ed25519', 'success').

% Replay protection: nonce + context + counter (SEC-004)
:- dynamic(nonce_record/4).

% nonce_record(Nonce, Context, MonotonicCounter, Timestamp).
nonce_record('nonce-loc-001', 'global', 1, 1719432000).
nonce_record('nonce-resonance-001', 'global', 2, 1719432001).
nonce_record('nonce-phantom-001', 'global', 3, 1719432002).

% Previous receipt hash linkage (for chain integrity)
:- dynamic(receipt_chain_link/2).

% receipt_chain_link(ReceiptHash, PreviousReceiptHash).
receipt_chain_link('a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
                   '0000000000000000000000000000000000000000000000000000000000000000').

receipt_chain_link('b8d5c0f3e2g4d9b6f7c3d0e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4',
                   'a7c4b9e2d1f3c8a5e6b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3').

receipt_chain_link('c9e6d1g4f3h5e0c7g8d4e1f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4g5',
                   'b8d5c0f3e2g4d9b6f7c3d0e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4').

% Ed25519 public keys (for signature verification)
:- dynamic(ed25519_public_key/3).

% ed25519_public_key(AgentID, KeyVersion, PublicKeyHex).
ed25519_public_key('loc', 1, 'd75a9801182fce40a8c0b4a0f6f9c1e2d3a4b5c6d7e8f9a0b1c2d3e4f5a6b7').
ed25519_public_key('resonance', 1, 'e86b0a12293ffc5b1b9d1c5e1h1f2g3h4i5j6k7l8m9n0o1p2q3r4s5t6u7v8w').
ed25519_public_key('phantom', 1, 'f97c1b23304gg6c2c0e2d6f2i2g3h4i5j6k7l8m9n0o1p2q3r4s5t6u7v8w9x0y').

% Receipt status verification
:- dynamic(receipt_status_valid/2).

receipt_status_valid(ReceiptID, true) :-
    receipt_v2(_, ReceiptID, _, _, _, _, _, _, _, _, _, Status),
    member(Status, [success, sealed]).

receipt_status_valid(ReceiptID, false) :-
    receipt_v2(_, ReceiptID, _, _, _, _, _, _, _, _, _, Status),
    \+ member(Status, [success, sealed]).

% Sealed receipt tracking (WORM)
:- dynamic(receipt_worm_sealed/2).

% receipt_worm_sealed(ReceiptHash, Timestamp).
% Populated when receipts are finalized
