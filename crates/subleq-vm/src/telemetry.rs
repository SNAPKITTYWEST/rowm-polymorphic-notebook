//! Telemetry — Live mutation delta streaming

use crate::memory::Address;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crossbeam::channel::{Sender, Receiver, unbounded};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationDelta {
    pub address: Address,
    pub old_value: i64,
    pub new_value: i64,
    pub ip: Address,
    pub step: u64,
}

pub struct TelemetryEmitter {
    tx: Sender<MutationDelta>,
}

impl TelemetryEmitter {
    pub fn new() -> (Arc<Self>, Receiver<MutationDelta>) {
        let (tx, rx) = unbounded();
        (Arc::new(Self { tx }), rx)
    }

    pub fn emit(&self, delta: MutationDelta) {
        let _ = self.tx.send(delta);
    }

    pub fn try_recv(&self, rx: &Receiver<MutationDelta>) -> Option<MutationDelta> {
        rx.try_recv().ok()
    }
}

impl Default for TelemetryEmitter {
    fn default() -> Self {
        let (tx, _) = unbounded();
        Self { tx }
    }
}
