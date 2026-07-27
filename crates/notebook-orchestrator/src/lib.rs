// Non-Recursive Notebook Orchestration Runtime
// Implements LOC triad dispatch with WORM-sealed receipt chains

pub mod orchestrator;
pub mod receipt;
pub mod receipt_v2;
pub mod instruction;
pub mod stage;
pub mod ed25519_keymanager;
pub mod replay_protection;
pub mod notebook_merkle;
pub mod web_agent;

pub use orchestrator::Orchestrator;
pub use receipt::{Receipt, ReceiptChain};
pub use receipt_v2::{ReceiptV2, ReceiptChainV2, ReceiptStatus, PerformanceMetrics};
pub use instruction::Instruction;
pub use stage::Stage;
pub use ed25519_keymanager::{Ed25519KeyPair, KeyStore, KeyMetadata, KeyStatus, KeyVersion};
pub use replay_protection::{GlobalReplayProtection, ReplayDetector, NonceGenerator, Context, Nonce};
pub use notebook_merkle::{NotebookCell, NotebookMerkleTree};
pub use web_agent::{AgentPersona, WebAgentRuntime, WebLLMBridge, ChatMessage, generate_web_agent_widget};
