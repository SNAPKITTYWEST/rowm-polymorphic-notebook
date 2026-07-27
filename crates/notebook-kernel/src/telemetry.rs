//! Telemetry — Live mutation delta streaming

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: u64,
    pub cell_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Live telemetry stream
pub struct TelemetryStream {
    events: Arc<RwLock<Vec<TelemetryEvent>>>,
}

impl TelemetryStream {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Emit a telemetry event
    pub fn emit(&self, event: TelemetryEvent) {
        self.events.write().push(event);
    }

    /// Get all events
    pub fn all_events(&self) -> Vec<TelemetryEvent> {
        self.events.read().clone()
    }

    /// Get recent events
    pub fn recent_events(&self, n: usize) -> Vec<TelemetryEvent> {
        let events = self.events.read();
        events
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect()
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.write().clear();
    }

    pub fn event_count(&self) -> usize {
        self.events.read().len()
    }
}

impl Default for TelemetryStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_stream_creation() {
        let stream = TelemetryStream::new();
        assert_eq!(stream.event_count(), 0);
    }

    #[test]
    fn test_emit_event() {
        let stream = TelemetryStream::new();
        let event = TelemetryEvent {
            timestamp: 0,
            cell_id: "cell-1".into(),
            event_type: "mutation".into(),
            data: serde_json::json!({"address": 0, "value": 42}),
        };

        stream.emit(event);
        assert_eq!(stream.event_count(), 1);
    }

    #[test]
    fn test_recent_events() {
        let stream = TelemetryStream::new();
        for i in 0..5 {
            stream.emit(TelemetryEvent {
                timestamp: i,
                cell_id: format!("cell-{}", i),
                event_type: "test".into(),
                data: serde_json::json!({}),
            });
        }

        let recent = stream.recent_events(2);
        assert_eq!(recent.len(), 2);
    }
}
