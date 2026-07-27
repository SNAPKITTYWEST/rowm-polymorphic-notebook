//! Jupyter Notebook Kernel + Execution Ring
//!
//! - Jupyter Protocol Implementation
//! - Zero-copy IPC via shared memory
//! - Decentralized cell agent scheduling
//! - Live telemetry streaming

pub mod kernel;
pub mod ipc;
pub mod cell_agent;
pub mod execution_ring;
pub mod telemetry;

pub use kernel::NotebookKernel;
pub use ipc::IpcChannel;
pub use cell_agent::CellAgent;
pub use execution_ring::ExecutionRing;
pub use telemetry::TelemetryStream;
