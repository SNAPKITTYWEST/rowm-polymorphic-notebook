# ROWM API Reference — Crate-by-Crate Guide

**Version:** 1.0.0  
**Status:** Normative  
**Authors:** Ahmad Ali Parr, Jessica SNAPKITTYWEST

---

## Overview

ROWM consists of 8 core Rust crates, each with distinct responsibilities. This document provides quick reference for each crate's public API.

For detailed implementation, see source files in `crates/*/src/`.

---

## 1. subleq-vm — Virtual Machine Core

**Location:** `crates/subleq-vm/`  
**Purpose:** Execute SUBLEQ bytecode with mutation tracking and checkpointing

### Public Types

```rust
pub struct Memory {
    cells: Vec<i64>,  // Von Neumann unified address space
}

pub struct VirtualMachine {
    memory: Memory,
    instruction_pointer: usize,
    mutations: Vec<MutationEvent>,
    checkpoints: VecDeque<Checkpoint>,
}

pub struct MutationEvent {
    pub address: usize,
    pub old_value: i64,
    pub new_value: i64,
    pub timestamp: u64,
}

pub struct Checkpoint {
    pub id: Uuid,
    pub memory_snapshot: Vec<i64>,
    pub instruction_pointer: usize,
    pub timestamp: u64,
    pub hash: String,  // Blake3
}
```

### Public Methods

```rust
impl VirtualMachine {
    pub fn new(program: Vec<i64>) -> Self;
    pub fn execute(&mut self) -> Result<ExecutionResult>;
    pub fn step(&mut self) -> Result<Option<MutationEvent>>;
    pub fn create_checkpoint(&mut self) -> Uuid;
    pub fn rollback(&mut self, checkpoint_id: Uuid) -> Result<()>;
    pub fn get_mutations(&self) -> &[MutationEvent];
    pub fn memory_at(&self, address: usize) -> i64;
    pub fn set_memory(&mut self, address: usize, value: i64);
    pub fn get_telemetry(&self) -> TelemetrySnapshot;
}

pub struct ExecutionResult {
    pub status: ExecutionStatus,  // Normal | Timeout | ViolationHalt
    pub final_memory: Vec<i64>,
    pub mutation_count: usize,
    pub checkpoint_count: usize,
    pub total_cycles: u64,
}

pub enum ExecutionStatus {
    Normal,
    Timeout,
    ViolationHalt,
    InvariantViolation(String),
}
```

---

## 2. subleq-ir — Intermediate Representation

**Location:** `crates/subleq-ir/`  
**Purpose:** Parse source code to AST, compile to bytecode, lower to SUBLEQ

### Public Types

```rust
pub enum Expr {
    Const(i64),
    Var(String),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

pub enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    Return(Expr),
    FunctionDef(String, Vec<String>, Vec<Stmt>),
}

pub struct Program {
    pub statements: Vec<Stmt>,
    pub functions: HashMap<String, Function>,
}

pub enum Bytecode {
    LoadConst(i64),
    LoadVar(String),
    Store(String),
    BinOp(BinOp),
    Jump(usize),
    JumpIfZero(usize),
    Call(String),
    Return,
}

pub struct Type {
    pub kind: TypeKind,  // Int64 | Array | Function | Unknown
    pub nullable: bool,
}
```

### Public Methods

```rust
impl Program {
    pub fn from_ast(ast: Vec<Stmt>) -> Result<Self>;
    pub fn to_bytecode(&self) -> Result<Vec<Bytecode>>;
    pub fn to_subleq(&self) -> Result<Vec<i64>>;
    pub fn validate(&self) -> Result<Vec<ValidationError>>;
}

pub struct BytecodeCompiler;
impl BytecodeCompiler {
    pub fn compile(ast: &Program) -> Result<Vec<Bytecode>>;
    pub fn allocate_registers(bytecode: &[Bytecode]) -> RegisterMap;
}

pub struct SubleqCodegen;
impl SubleqCodegen {
    pub fn lower(bytecode: &[Bytecode], register_map: &RegisterMap) -> Vec<i64>;
}
```

---

## 3. polyglot-frontend — Multi-Language Parsing

**Location:** `crates/polyglot-frontend/`  
**Purpose:** Parse 30+ languages, normalize to unified AST

### Public Types

```rust
pub enum Language {
    // Tier 1 (Full)
    Rust, Python, JavaScript, Subleq,
    // Tier 2 (Solid)
    Haskell, Ada, Agda, Lean,
    // Tier 3 (Supported)
    Prolog, Lisp, Scheme, Bqn,
    // Tier 4 (Partial)
    C, Go, Zig, Apl, Forth,
    // Tier 5 (Experimental)
    Factor, Brainfuck, J, Holyc, Emojicode,
}

pub struct LanguageRegistry {
    parsers: HashMap<Language, Box<dyn Parser>>,
}

pub trait Parser: Send + Sync {
    fn parse(&self, source: &str) -> Result<Ast>;
    fn language(&self) -> Language;
}

pub struct Ast {
    pub root: AstNode,
    pub source_hash: String,  // Blake3(source)
}

pub enum AstNode {
    Program(Vec<AstNode>),
    Function(FunctionDef),
    Statement(Statement),
    Expression(Expression),
}
```

### Public Methods

```rust
impl LanguageRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, language: Language, parser: Box<dyn Parser>);
    pub fn parse(&self, language: Language, source: &str) -> Result<Ast>;
    pub fn detect_language(&self, source: &str) -> Option<Language>;
    pub fn supported_languages(&self) -> Vec<Language>;
}

pub struct PytonParser;
impl Parser for PythonParser {
    fn parse(&self, source: &str) -> Result<Ast>;
    fn language(&self) -> Language { Language::Python }
}

pub struct RustParser;
impl Parser for RustParser {
    fn parse(&self, source: &str) -> Result<Ast>;
    fn language(&self) -> Language { Language::Rust }
}
```

---

## 4. invariant-extractor — Symbolic Execution & Verification

**Location:** `crates/invariant-extractor/`  
**Purpose:** Extract loop invariants, generate proof obligations

### Public Types

```rust
pub enum SymbolicValue {
    Const(i64),
    Mem(usize),  // memory address
    Reg(usize),  // register index
    BinOp(BinOp, Box<SymbolicValue>, Box<SymbolicValue>),
    UnOp(UnOp, Box<SymbolicValue>),
}

pub enum Predicate {
    True,
    False,
    Eq(SymbolicValue, SymbolicValue),
    Le(SymbolicValue, SymbolicValue),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Not(Box<Predicate>),
}

pub struct SymbolicState {
    pub registers: HashMap<usize, SymbolicValue>,
    pub memory: HashMap<usize, SymbolicValue>,
    pub assumptions: Vec<Predicate>,
}

pub struct ProofObligation {
    pub id: String,
    pub name: String,  // InvariantPreservation, etc
    pub formula: Predicate,
    pub status: ProofStatus,
}

pub enum ProofStatus {
    Unknown,
    Proved,
    Disproved,
    Manual,
}

pub struct Invariant {
    pub loop_id: usize,
    pub predicate: Predicate,
    pub proof_strategy: ProofStrategy,
}
```

### Public Methods

```rust
pub struct InvariantExtractor;
impl InvariantExtractor {
    pub fn extract(&self, bytecode: &[Bytecode]) -> Result<Vec<Invariant>>;
    pub fn generate_proof_obligations(&self, program: &Program) -> Vec<ProofObligation>;
    pub fn compute_abstract_domain(bytecode: &[Bytecode]) -> AbstractDomain;
}

pub struct SymbolicExecutor;
impl SymbolicExecutor {
    pub fn trace(bytecode: &[Bytecode]) -> Result<Vec<ExecutionPath>>;
    pub fn evaluate(&self, expr: &SymbolicValue, state: &SymbolicState) -> SymbolicValue;
}

pub struct AbstractInterpreter;
impl AbstractInterpreter {
    pub fn fixpoint(bytecode: &[Bytecode]) -> HashMap<usize, Interval>;
    pub fn widen(v1: Interval, v2: Interval) -> Interval;
    pub fn narrow(v: Interval, constraint: &Predicate) -> Interval;
}
```

---

## 5. proof-validator — Proof Checking & WORM Rollback

**Location:** `crates/proof-validator/`  
**Purpose:** Type-check proofs (Curry-Howard), manage checkpoints for rollback

### Public Types

```rust
pub enum ProofTerm {
    Var(String),
    Abs(String, Box<ProofTerm>),  // lambda
    App(Box<ProofTerm>, Box<ProofTerm>),  // application
    Const(Constant),
    Pair(Box<ProofTerm>, Box<ProofTerm>),
    Fst(Box<ProofTerm>),  // projection
    Snd(Box<ProofTerm>),
}

pub struct ProofContext {
    pub assumptions: Vec<(String, ProofTerm)>,
}

pub struct ProofObligation {
    pub id: String,
    pub theorem: Predicate,
    pub required_by_stage: String,  // "compiled" | "executed" | "verified"
}

pub struct RollbackManager {
    checkpoints: VecDeque<Checkpoint>,
    max_checkpoints: usize,
}

pub struct ProofValidator;
```

### Public Methods

```rust
pub struct TypeChecker;
impl TypeChecker {
    pub fn check(&self, term: &ProofTerm, expected_type: &Type, ctx: &ProofContext) -> Result<()>;
    pub fn infer(&self, term: &ProofTerm, ctx: &ProofContext) -> Result<Type>;
}

impl RollbackManager {
    pub fn new(max_checkpoints: usize) -> Self;
    pub fn push(&mut self, checkpoint: Checkpoint) -> Result<()>;
    pub fn pop(&mut self) -> Option<Checkpoint>;
    pub fn rollback_to(&mut self, checkpoint_id: Uuid) -> Result<()>;
    pub fn list_checkpoints(&self) -> Vec<CheckpointSummary>;
}

pub struct ProofValidator;
impl ProofValidator {
    pub fn validate_obligation(&self, obligation: &ProofObligation) -> Result<ProofStatus>;
    pub fn emit_audit(&self, event: ProofEvent) -> Result<()>;
}

pub enum ProofEvent {
    Validated { obligation_id: String, result: ProofStatus },
    Violated { obligation_id: String, evidence: String },
    RolledBack { checkpoint_id: Uuid },
}
```

---

## 6. m4-morph — Macro Engine & Self-Modification

**Location:** `crates/m4-morph/`  
**Purpose:** GNU M4 macro expansion with sandboxing and state feedback

### Public Types

```rust
pub struct SandboxLimits {
    pub max_expansion_depth: usize,  // default 100
    pub max_output_size: usize,      // default 1MB
    pub max_recursion: usize,        // default 50
    pub timeout_ms: u64,             // default 5000
}

pub enum SandboxPreset {
    Permissive,  // for trusted sources
    Strict,      // for untrusted sources
}

pub struct FeedbackBuffer {
    pub definitions: VecDeque<(String, String)>,  // bounded 50
    pub outputs: VecDeque<String>,                 // bounded 100
}

pub struct M4Engine {
    pub sandbox: SandboxConfig,
    pub feedback: FeedbackBuffer,
}

pub struct M4Engine {
    pub sandbox: SandboxConfig,
}
```

### Public Methods

```rust
impl M4Engine {
    pub fn new(limits: SandboxLimits) -> Self;
    pub fn expand(&mut self, source: &str) -> Result<String>;
    pub fn define(&mut self, name: &str, value: &str);
    pub fn get_definition(&self, name: &str) -> Option<&str>;
    pub fn set_sandbox(&mut self, config: SandboxConfig);
}

pub struct FeedbackBuffer;
impl FeedbackBuffer {
    pub fn push_definition(&mut self, name: String, value: String);
    pub fn push_output(&mut self, output: String);
    pub fn get_recent_outputs(&self, n: usize) -> Vec<String>;
    pub fn get_all_definitions(&self) -> Vec<(String, String)>;
}
```

---

## 7. notebook-kernel — Jupyter Protocol & Cell Execution

**Location:** `crates/notebook-kernel/`  
**Purpose:** Jupyter protocol implementation, cell-to-cell communication, execution ring

### Public Types

```rust
pub enum JupyterMessageType {
    ExecuteRequest,
    ExecuteReply,
    DisplayData,
    Stream,
    Error,
    Status,
}

pub struct JupyterMessage {
    pub message_type: JupyterMessageType,
    pub metadata: HashMap<String, String>,
    pub content: serde_json::Value,
}

pub struct NotebookKernel {
    pub kernel_id: String,
    pub execution_ring: ExecutionRing,
    pub ipc_channels: HashMap<usize, IpcChannel>,
}

pub struct CellConfig {
    pub cell_id: String,
    pub language: Language,
    pub kernel: String,
    pub visibility: CellVisibility,
}

pub enum CellVisibility {
    Visible,
    Hidden,
    Collapsed,
}

pub struct ExecutionRing {
    pub work_queue: VecDeque<CellInstruction>,
    pub active_cells: HashSet<String>,
}

pub struct IpcChannel {
    pub shared_buffer: Arc<RwLock<Vec<u8>>>,
}
```

### Public Methods

```rust
impl NotebookKernel {
    pub fn new(kernel_id: String) -> Self;
    pub fn handle_message(&mut self, msg: JupyterMessage) -> Result<()>;
    pub fn execute_cell(&mut self, cell: CellConfig, source: &str) -> Result<CellOutput>;
    pub fn get_execution_status(&self) -> ExecutionStatus;
}

impl ExecutionRing {
    pub fn enqueue(&mut self, instruction: CellInstruction);
    pub fn dequeue(&mut self) -> Option<CellInstruction>;
    pub fn poll(&mut self) -> Vec<ExecutionEvent>;
}

impl IpcChannel {
    pub fn send(&self, data: &[u8]) -> Result<()>;
    pub fn recv(&self) -> Result<Vec<u8>>;
}
```

---

## 8. notebook-orchestrator — Non-Recursive Receipt Chain

**Location:** `crates/notebook-orchestrator/`  
**Purpose:** 8-stage pipeline, receipt generation, Prolog integration

### Public Types

```rust
pub struct Instruction {
    pub id: String,  // SHA-256
    pub agent: String,
    pub capability: String,
    pub runtime: String,
    pub permission: String,
    pub payload: Vec<u8>,
}

pub enum Stage {
    Receive,
    Translate,
    Verify,
    Dispatch,
    Execute,
    Encode,
    Seal,
    Complete,
}

pub struct Receipt {
    pub id: String,  // Blake3(contents)
    pub sequence: usize,
    pub instruction_hash: String,
    pub stage: Stage,
    pub result: ExecutionResult,
    pub timestamp: u64,
    pub signature: String,  // Ed25519
    pub previous_receipt_hash: String,  // chain link
}

pub struct Orchestrator {
    pub work_queue: VecDeque<Instruction>,
    pub receipts: Vec<Receipt>,
    pub prolog_engine: PrologBridge,
}
```

### Public Methods

```rust
pub struct Orchestrator;
impl Orchestrator {
    pub fn new(prolog_path: &str) -> Result<Self>;
    pub fn process_instruction(&mut self, instr: Instruction) -> Result<Receipt>;
    pub fn get_receipt_chain(&self) -> Vec<Receipt>;
    pub fn verify_chain_integrity(&self) -> bool;
    pub fn emit_receipt(&mut self, receipt: Receipt) -> Result<()>;
}

pub struct PrologBridge;
impl PrologBridge {
    pub fn query(&self, predicate: &str, args: &[&str]) -> Result<Vec<String>>;
    pub fn assert_fact(&self, fact: &str) -> Result<()>;
    pub fn check_authorization(&self, agent: &str, cap: &str, runtime: &str, perm: &str) -> Result<bool>;
    pub fn is_release_ready(&self) -> Result<bool>;
}

impl Receipt {
    pub fn compute_hash(contents: &[u8]) -> String;
    pub fn verify_signature(&self, pubkey: &str) -> Result<bool>;
}
```

---

## Error Handling

All public methods return `Result<T, Error>` where `Error` implements `std::error::Error`.

```rust
pub enum Error {
    ParseError(String),
    TypeError(String),
    ExecutionError(String),
    ProofError(String),
    AuthorizationError(String),
    TimeoutError(String),
    ReceiptError(String),
}
```

---

## Examples

### Execute SUBLEQ Bytecode

```rust
use subleq_vm::{VirtualMachine, ExecutionStatus};

let program = vec![
    0, 0, 3,    // M[0] -= M[0]; IP=3
    100, 100, 106,  // M[100] -= M[100]; IP=106
    // ... rest of program
];

let mut vm = VirtualMachine::new(program);
let result = vm.execute()?;

match result.status {
    ExecutionStatus::Normal => println!("Success: {} mutations", result.mutation_count),
    ExecutionStatus::Timeout => println!("Timed out"),
    ExecutionStatus::ViolationHalt => println!("Invariant violated"),
}
```

### Parse Python and Compile to SUBLEQ

```rust
use polyglot_frontend::{LanguageRegistry, Language};
use subleq_ir::Program;

let mut registry = LanguageRegistry::new();
let python_parser = PythonParser::new();
registry.register(Language::Python, Box::new(python_parser));

let source = r#"
x = 10
y = 20
z = x + y
"#;

let ast = registry.parse(Language::Python, source)?;
let program = Program::from_ast(ast)?;
let bytecode = program.to_bytecode()?;
let subleq = program.to_subleq()?;
```

### Extract Invariants and Generate Proofs

```rust
use invariant_extractor::InvariantExtractor;

let extractor = InvariantExtractor::new();
let invariants = extractor.extract(&bytecode)?;
let obligations = extractor.generate_proof_obligations(&program)?;

for obligation in obligations {
    println!("Obligation: {}", obligation.name);
    println!("Formula: {:?}", obligation.formula);
}
```

---

**For detailed implementation examples, consult crate-specific documentation in each source directory.**

*"LOC WRITES. LEDGER CERTIFIES. METATRON SEALS."*
