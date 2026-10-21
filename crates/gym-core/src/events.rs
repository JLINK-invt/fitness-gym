use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

use crate::error::CoreError;

/// Local event envelope. `idempotency_key` makes replays safe after a restore.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub aggregate: String,
    pub organization_id: String,
    pub idempotency_key: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

pub trait EventBus {
    fn publish(&self, event: EventEnvelope) -> Result<(), CoreError>;
}

/// In-memory bus used by the vertical slice and tests. The SQLite-backed
/// outbox replaces it in the persistence phase without changing this trait.
#[derive(Debug, Default)]
pub struct InMemoryEventBus {
    seen: Mutex<HashSet<String>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn published_count(&self) -> usize {
        self.seen.lock().expect("event bus lock").len()
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: EventEnvelope) -> Result<(), CoreError> {
        if event.idempotency_key.trim().is_empty() {
            return Err(CoreError::InvalidValue(
                "idempotency key is required".into(),
            ));
        }

        let mut seen = self.seen.lock().expect("event bus lock");
        if !seen.insert(event.idempotency_key.clone()) {
            return Err(CoreError::DuplicateEvent(event.idempotency_key));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(key: &str) -> EventEnvelope {
        EventEnvelope {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "access.granted".into(),
            aggregate: "membership".into(),
            organization_id: "org-1".into(),
            idempotency_key: key.into(),
            occurred_at: Utc::now(),
            payload: serde_json::json!({ "allowed": true }),
        }
    }

    #[test]
    fn duplicate_events_are_rejected() {
        let bus = InMemoryEventBus::new();
        bus.publish(event("sale-1")).expect("first event");
        assert_eq!(
            bus.publish(event("sale-1")),
            Err(CoreError::DuplicateEvent("sale-1".into()))
        );
        assert_eq!(bus.published_count(), 1);
    }

    #[test]
    fn empty_idempotency_key_is_invalid() {
        let bus = InMemoryEventBus::new();
        assert!(matches!(
            bus.publish(event("  ")),
            Err(CoreError::InvalidValue(_))
        ));
    }
}
