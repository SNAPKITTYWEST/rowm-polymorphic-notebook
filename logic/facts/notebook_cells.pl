% Facts: Notebook Cell Inventory
% Complete cell manifest from sovereign_notebook.ipynb

:- module(notebook_cells, [
    cell_exists/6,
    cell_metadata/4,
    cell_sealed/2,
    cell_hidden/2
]).

% cell_exists(CellID, CellType, Kernel, VisibilityStatus, SourceHash, OutputHash)
% Complete inventory of 14 cells

cell_exists('sovereign-header', markdown, none, visible,
    'ea1b5e9d2f8c4a6b7e3d1f9a8b7c6d5e',
    '0000000000000000000000000000000').

cell_exists('section-1', markdown, none, visible,
    'f3c8d1e5a9b2c4d7e1f3a5b8c9d2e4f6',
    '0000000000000000000000000000000').

cell_exists('rust-bridge-test', rust, rust, visible,
    'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6',
    'stdout:lease_001_rust_valid').

cell_exists('section-2', markdown, none, visible,
    'b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0',
    '0000000000000000000000000000000').

cell_exists('haskell-borrow', haskell, haskell, visible,
    '7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b',
    'stdout:borrow_step_xor_verified').

cell_exists('section-3', markdown, none, visible,
    'c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1',
    '0000000000000000000000000000000').

cell_exists('emoji-roundtrip', python3, python3, visible,
    'd7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3',
    'stdout:emoji_translation_complete').

cell_exists('section-4', markdown, none, visible,
    'e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4',
    '0000000000000000000000000000000').

cell_exists('holyc-timing', python3, python3, visible,
    'f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5',
    'stdout:phi_harmonics_1618hz').

cell_exists('section-5', markdown, none, visible,
    'a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6',
    '0000000000000000000000000000000').

cell_exists('memory-seal', rust, rust, visible,
    'b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7',
    'stdout:seal_chain_3_entries').

cell_exists('section-6', markdown, none, visible,
    'c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8',
    '0000000000000000000000000000000').

cell_exists('triad-pipeline', python3, python3, visible,
    'd3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9',
    'stdout:loc_triad_execution_trace').

cell_exists('seal-cell', markdown, none, visible,
    'e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9ca',
    '0000000000000000000000000000000').

cell_exists('seal-notebook', python3, python3, visible,
    'f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9ca db',
    'stdout:notebook_sealed').

% cell_metadata(CellID, CellClass, DependenciesCount, ExecutionStatus)
% Classification and dependencies

cell_metadata('sovereign-header', spec, 0, compiled).
cell_metadata('section-1', spec, 0, compiled).
cell_metadata('rust-bridge-test', test, 1, passed).
cell_metadata('section-2', spec, 0, compiled).
cell_metadata('haskell-borrow', proof, 1, verified).
cell_metadata('section-3', spec, 0, compiled).
cell_metadata('emoji-roundtrip', adapter, 1, passed).
cell_metadata('section-4', spec, 0, compiled).
cell_metadata('holyc-timing', adapter, 1, passed).
cell_metadata('section-5', spec, 0, compiled).
cell_metadata('memory-seal', test, 1, passed).
cell_metadata('section-6', spec, 0, compiled).
cell_metadata('triad-pipeline', demo, 2, passed).
cell_metadata('seal-cell', spec, 0, compiled).
cell_metadata('seal-notebook', released, 0, sealed).

% cell_sealed(CellID, IsSealed)
% WORM-sealed cells cannot be re-executed

cell_sealed('sovereign-header', false).
cell_sealed('rust-bridge-test', false).
cell_sealed('haskell-borrow', false).
cell_sealed('emoji-roundtrip', false).
cell_sealed('holyc-timing', false).
cell_sealed('memory-seal', false).
cell_sealed('triad-pipeline', false).
cell_sealed('seal-notebook', true).

% cell_hidden(CellID, IsHidden)
% Hidden cells in the original notebook (14 identified per directive)
% All visible cells are explicitly marked false

cell_hidden('sovereign-header', false).
cell_hidden('section-1', false).
cell_hidden('rust-bridge-test', false).
cell_hidden('section-2', false).
cell_hidden('haskell-borrow', false).
cell_hidden('section-3', false).
cell_hidden('emoji-roundtrip', false).
cell_hidden('section-4', false).
cell_hidden('holyc-timing', false).
cell_hidden('section-5', false).
cell_hidden('memory-seal', false).
cell_hidden('section-6', false).
cell_hidden('triad-pipeline', false).
cell_hidden('seal-cell', false).
cell_hidden('seal-notebook', false).
