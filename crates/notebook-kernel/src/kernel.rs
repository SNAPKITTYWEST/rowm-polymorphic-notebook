//! Jupyter Kernel Protocol — Message handling and dispatch

use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::debug;

/// Jupyter message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JupyterMessageType {
    ExecuteRequest,
    InspectRequest,
    CompleteRequest,
    ShutdownRequest,
    DisplayData,
    ExecuteReply,
    Error,
}

/// Jupyter message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterMessage {
    pub msg_type: JupyterMessageType,
    pub parent_header: Option<serde_json::Value>,
    pub content: serde_json::Value,
}

/// Notebook kernel for Jupyter
pub struct NotebookKernel {
    kernel_id: String,
}

impl NotebookKernel {
    pub fn new(kernel_id: String) -> Self {
        Self { kernel_id }
    }

    /// Handle incoming Jupyter message
    pub fn handle_message(&self, msg: JupyterMessage) -> Result<Option<JupyterMessage>> {
        debug!("Kernel {} received message: {:?}", self.kernel_id, msg.msg_type);

        match msg.msg_type {
            JupyterMessageType::ExecuteRequest => {
                // Execute cell code
                Ok(Some(JupyterMessage {
                    msg_type: JupyterMessageType::ExecuteReply,
                    parent_header: msg.parent_header,
                    content: serde_json::json!({
                        "status": "ok",
                        "execution_count": 1
                    }),
                }))
            }
            JupyterMessageType::ShutdownRequest => {
                // Shutdown kernel
                Ok(None)
            }
            _ => {
                Ok(Some(JupyterMessage {
                    msg_type: JupyterMessageType::Error,
                    parent_header: msg.parent_header,
                    content: serde_json::json!({
                        "ename": "NotImplemented",
                        "evalue": "Message type not implemented"
                    }),
                }))
            }
        }
    }

    pub fn kernel_id(&self) -> &str {
        &self.kernel_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_creation() {
        let kernel = NotebookKernel::new("test-kernel".into());
        assert_eq!(kernel.kernel_id(), "test-kernel");
    }

    #[test]
    fn test_execute_request() {
        let kernel = NotebookKernel::new("test".into());
        let msg = JupyterMessage {
            msg_type: JupyterMessageType::ExecuteRequest,
            parent_header: None,
            content: serde_json::json!({"code": "print('hello')"}),
        };

        let response = kernel.handle_message(msg).unwrap();
        assert!(response.is_some());
        let reply = response.unwrap();
        assert!(matches!(reply.msg_type, JupyterMessageType::ExecuteReply));
    }
}
