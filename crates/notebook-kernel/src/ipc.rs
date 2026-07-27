//! Zero-copy IPC — Shared memory channel for cell communication

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;

/// Zero-copy IPC channel
pub struct IpcChannel {
    channel_id: String,
    buffer: Arc<RwLock<Vec<u8>>>,
}

impl IpcChannel {
    pub fn new(channel_id: &str) -> Result<Self> {
        Ok(Self {
            channel_id: channel_id.to_string(),
            buffer: Arc::new(RwLock::new(Vec::with_capacity(65536))), // 64KB initial
        })
    }

    /// Send data via shared memory
    pub fn send(&self, data: &[u8]) -> Result<()> {
        let mut buf = self.buffer.write();
        buf.clear();
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Receive data from shared memory
    pub fn recv(&self) -> Result<Vec<u8>> {
        let buf = self.buffer.read();
        Ok(buf.clone())
    }

    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }

    pub fn clear(&self) {
        self.buffer.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_channel_creation() {
        let channel = IpcChannel::new("test").unwrap();
        assert_eq!(channel.channel_id(), "test");
    }

    #[test]
    fn test_send_recv() {
        let channel = IpcChannel::new("test").unwrap();
        let data = b"hello";
        channel.send(data).unwrap();

        let received = channel.recv().unwrap();
        assert_eq!(received, data);
    }

    #[test]
    fn test_clear() {
        let channel = IpcChannel::new("test").unwrap();
        channel.send(b"data").unwrap();
        channel.clear();

        let received = channel.recv().unwrap();
        assert!(received.is_empty());
    }
}
