% Facts: Runtime Definitions and Capabilities
% Supported runtimes in the LOC triad

:- module(runtimes, [
    runtime_supported/3,
    runtime_active/2,
    runtime_constraint/3,
    runtime_kernel/2
]).

% runtime_supported(RuntimeName, DisplayName, IsSupported)

runtime_supported(rust,    'Rust LOC Engine', true).
runtime_supported(ada,     'Ada/SPARK Verifier', true).
runtime_supported(holyc,   'HolyC Ring-0 Executor', true).
runtime_supported(haskell, 'Haskell Quantum Kernel', true).
runtime_supported(emoji,   'EmojiCode Translator', true).
runtime_supported(python3, 'Python3 Glue Layer', true).

% runtime_active(RuntimeName, IsActive)
% Indicates operational status at system start

runtime_active(rust, true).
runtime_active(ada, true).
runtime_active(holyc, true).
runtime_active(haskell, true).
runtime_active(emoji, true).
runtime_active(python3, true).

% runtime_constraint(RuntimeName, ConstraintType, Value)
% Operational constraints: max_dispatch, max_memory, execution_timeout_ms, thread_limit

runtime_constraint(rust,    max_dispatch, 1000).
runtime_constraint(rust,    max_memory, 2147483648).
runtime_constraint(rust,    execution_timeout_ms, 30000).
runtime_constraint(rust,    thread_limit, 16).

runtime_constraint(ada,     max_dispatch, 500).
runtime_constraint(ada,     max_memory, 1073741824).
runtime_constraint(ada,     execution_timeout_ms, 60000).
runtime_constraint(ada,     thread_limit, 4).

runtime_constraint(holyc,   max_dispatch, 10000).
runtime_constraint(holyc,   max_memory, 134217728).
runtime_constraint(holyc,   execution_timeout_ms, 100).
runtime_constraint(holyc,   thread_limit, 1).

runtime_constraint(haskell, max_dispatch, 500).
runtime_constraint(haskell, max_memory, 1073741824).
runtime_constraint(haskell, execution_timeout_ms, 45000).
runtime_constraint(haskell, thread_limit, 8).

runtime_constraint(emoji,   max_dispatch, 100).
runtime_constraint(emoji,   max_memory, 104857600).
runtime_constraint(emoji,   execution_timeout_ms, 5000).
runtime_constraint(emoji,   thread_limit, 1).

% runtime_kernel(RuntimeName, KernelType)
% Maps runtime to its notebook kernel identifier

runtime_kernel(rust,    'rust').
runtime_kernel(ada,     'ada').
runtime_kernel(holyc,   'holyc').
runtime_kernel(haskell, 'haskell').
runtime_kernel(emoji,   'python3').
runtime_kernel(python3, 'python3').
