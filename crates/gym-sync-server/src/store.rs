use std::path::Path;

use gym_sync::InboundEvent;
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::StoreError;

/// Durable event log shared by every gym connected to this server.
pub struct SyncStore {
    connection: Connection,
}

/// Event ready to be stored in the shared log.
pub struct NewSyncEvent<'a> {
    pub event_id: &'a str,
    pub event_type: &'a str,
    pub aggregate_type: &'a str,
    pub aggregate_id: &'a str,
    pub idempotency_key: &'a str,
    pub payload_json: &'a str,
    pub occurred_at: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppendOutcome {
    /// The event was stored and received the given server sequence.
    Accepted { sequence: i64 },
    /// The idempotency key was already stored for this organization.
    Duplicate { sequence: i64 },
}

impl SyncStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS sync_events (
                 organization_id TEXT NOT NULL,
                 sequence INTEGER NOT NULL,
                 event_id TEXT NOT NULL,
                 event_type TEXT NOT NULL,
                 aggregate_type TEXT NOT NULL,
                 aggregate_id TEXT NOT NULL,
                 idempotency_key TEXT NOT NULL,
                 payload_json TEXT NOT NULL,
                 occurred_at TEXT NOT NULL,
                 received_at TEXT NOT NULL,
                 PRIMARY KEY (organization_id, sequence)
             ) STRICT;
             CREATE UNIQUE INDEX IF NOT EXISTS idx_sync_events_dedupe
                 ON sync_events (organization_id, idempotency_key);
             CREATE INDEX IF NOT EXISTS idx_sync_events_received
                 ON sync_events (organization_id, received_at);",
        )?;
        Ok(())
    }

    /// Stores an event, returning the assigned sequence or the sequence of the
    /// event that was already received with the same idempotency key.
    pub fn append(
        &mut self,
        organization_id: &str,
        event: NewSyncEvent<'_>,
    ) -> Result<AppendOutcome, StoreError> {
        let transaction = self.connection.transaction()?;
        let existing: Option<i64> = transaction
            .query_row(
                "SELECT sequence FROM sync_events
                 WHERE organization_id = ?1 AND idempotency_key = ?2",
                params![organization_id, event.idempotency_key],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(sequence) = existing {
            transaction.rollback()?;
            return Ok(AppendOutcome::Duplicate { sequence });
        }
        let next_sequence: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(sequence), 0) + 1 FROM sync_events
             WHERE organization_id = ?1",
            params![organization_id],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT INTO sync_events
                 (organization_id, sequence, event_id, event_type, aggregate_type,
                  aggregate_id, idempotency_key, payload_json, occurred_at, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                organization_id,
                next_sequence,
                event.event_id,
                event.event_type,
                event.aggregate_type,
                event.aggregate_id,
                event.idempotency_key,
                event.payload_json,
                event.occurred_at,
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        transaction.commit()?;
        Ok(AppendOutcome::Accepted {
            sequence: next_sequence,
        })
    }

    pub fn events_after(
        &self,
        organization_id: &str,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<InboundEvent>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT event_id, sequence, event_type, aggregate_type, aggregate_id,
                    idempotency_key, payload_json, occurred_at
             FROM sync_events
             WHERE organization_id = ?1 AND sequence > ?2
             ORDER BY sequence ASC
             LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![organization_id, after_sequence, limit.clamp(1, 200) as i64],
            |row| {
                let payload_json: String = row.get(6)?;
                let payload = serde_json::from_str(&payload_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(InboundEvent {
                    event_id: row.get(0)?,
                    server_sequence: row.get(1)?,
                    event_type: row.get(2)?,
                    aggregate_type: row.get(3)?,
                    aggregate_id: row.get(4)?,
                    idempotency_key: row.get(5)?,
                    payload,
                    occurred_at: row.get(7)?,
                })
            },
        )?;
        let events = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(events)
    }

    pub fn last_sequence(&self, organization_id: &str) -> Result<i64, StoreError> {
        let sequence: i64 = self.connection.query_row(
            "SELECT COALESCE(MAX(sequence), 0) FROM sync_events WHERE organization_id = ?1",
            params![organization_id],
            |row| row.get(0),
        )?;
        Ok(sequence)
    }
}
