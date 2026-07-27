% Facts: Receipt Chain — WORM-sealed execution records
% Each execution generates a cryptographic receipt

:- module(receipts, [
    receipt_issued/9,
    receipt_chain_head/1,
    receipt_valid/2,
    previous_receipt/2
]).

% receipt_issued(ReceiptID, SequenceNumber, AgentID, CapabilityID, InstructionHash,
%                 Action, InputHash, OutputHash, Timestamp)

receipt_issued(
    'rcpt_0000001_loc_001',
    0,
    loc,
    'capa_001_loc_rust_exec',
    'instr_hash_bridge_test_00001',
    dispatch,
    'input_lease_bridge_test',
    'output_lease_bridge_test_valid',
    1719432001
).

receipt_issued(
    'rcpt_0000002_sentinel_001',
    1,
    sentinel,
    'capa_003_sentinel_ada_verify',
    'instr_hash_borrow_proof_00001',
    verify,
    'input_borrow_step_proof',
    'output_borrow_step_verified',
    1719432002
).

receipt_issued(
    'rcpt_0000003_resonance_001',
    2,
    resonance,
    'capa_004_resonance_haskell_compute',
    'instr_hash_emoji_roundtrip_00001',
    compute,
    'input_emoji_expr_str',
    'output_emoji_translation_seal',
    1719432003
).

receipt_issued(
    'rcpt_0000004_resonance_002',
    3,
    resonance,
    'capa_004_resonance_haskell_compute',
    'instr_hash_phi_timing_00001',
    compute,
    'input_phi_harmonic_calc',
    'output_phi_harmonics_1618hz',
    1719432004
).

receipt_issued(
    'rcpt_0000005_forge_001',
    4,
    forge,
    'capa_002_forge_rust_build',
    'instr_hash_memory_seal_00001',
    build,
    'input_seal_chain_def',
    'output_seal_chain_3_entries',
    1719432005
).

receipt_issued(
    'rcpt_0000006_metatron_001',
    5,
    metatron,
    'capa_005_metatron_haskell_seal',
    'instr_hash_triad_pipeline_00001',
    worm_seal,
    'input_pipeline_trace',
    'output_loc_triad_execution_trace',
    1719432006
).

receipt_issued(
    'rcpt_0000007_metatron_002',
    6,
    metatron,
    'capa_005_metatron_haskell_seal',
    'instr_hash_seal_notebook_00001',
    finalize,
    'input_notebook_outputs_hash',
    'seal_worm_anchor_abc123def456',
    1719432007
).

% receipt_chain_head(ReceiptID)
% The current head of the receipt chain (most recent receipt)

receipt_chain_head('rcpt_0000007_metatron_002').

% receipt_valid(ReceiptID, IsValid)
% A receipt is valid if cryptographically sound

receipt_valid('rcpt_0000001_loc_001', true).
receipt_valid('rcpt_0000002_sentinel_001', true).
receipt_valid('rcpt_0000003_resonance_001', true).
receipt_valid('rcpt_0000004_resonance_002', true).
receipt_valid('rcpt_0000005_forge_001', true).
receipt_valid('rcpt_0000006_metatron_001', true).
receipt_valid('rcpt_0000007_metatron_002', true).

% previous_receipt(CurrentReceiptID, PreviousReceiptID)
% Immutable chain links

previous_receipt('rcpt_0000002_sentinel_001', 'rcpt_0000001_loc_001').
previous_receipt('rcpt_0000003_resonance_001', 'rcpt_0000002_sentinel_001').
previous_receipt('rcpt_0000004_resonance_002', 'rcpt_0000003_resonance_001').
previous_receipt('rcpt_0000005_forge_001', 'rcpt_0000004_resonance_002').
previous_receipt('rcpt_0000006_metatron_001', 'rcpt_0000005_forge_001').
previous_receipt('rcpt_0000007_metatron_002', 'rcpt_0000006_metatron_001').
