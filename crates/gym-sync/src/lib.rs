use chrono::{DateTime, Duration, Utc};
use gym_data::{Db, InboundSyncEvent, SyncOutboxRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "mtls")]
pub mod mtls;
pub mod wire;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("database error: {0}")]
    Database(#[from] gym_data::DataError),

    #[error("invalid sync payload: {0}")]
    InvalidPayload(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundEvent {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub payload: Value,
    pub attempts: i64,
    pub created_at: String,
}

impl TryFrom<SyncOutboxRecord> for OutboundEvent {
    type Error = SyncError;

    fn try_from(record: SyncOutboxRecord) -> Result<Self, Self::Error> {
        let payload = serde_json::from_str(&record.payload_json).map_err(|error| {
            SyncError::InvalidPayload(format!("outbox event {}: {error}", record.id))
        })?;
        Ok(Self {
            event_id: record.id,
            event_type: record.event_type,
            aggregate_type: record.aggregate_type,
            aggregate_id: record.aggregate_id,
            idempotency_key: record.idempotency_key,
            payload,
            attempts: record.attempts,
            created_at: record.created_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundEvent {
    pub event_id: String,
    pub server_sequence: i64,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub payload: Value,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PushOutcome {
    Accepted,
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushAck {
    pub idempotency_key: String,
    pub outcome: PushOutcome,
}

/// Transport boundary. The production implementation will use HTTPS with
/// device mTLS; tests use an in-memory fake server.
pub trait SyncTransport {
    fn push(
        &mut self,
        organization_id: &str,
        events: &[OutboundEvent],
    ) -> Result<Vec<PushAck>, String>;

    fn pull(
        &mut self,
        organization_id: &str,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<InboundEvent>, String>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub pushed: usize,
    pub accepted: usize,
    pub retried: usize,
    pub dead_lettered: usize,
    pub pulled: usize,
    pub applied: usize,
    pub duplicates: usize,
    pub transport_error: Option<String>,
}

pub struct SyncEngine<'a, T: SyncTransport> {
    db: &'a Db,
    transport: &'a mut T,
    batch_size: usize,
    max_attempts: i64,
    retry_delay_seconds: i64,
}

impl<'a, T: SyncTransport> SyncEngine<'a, T> {
    pub fn new(db: &'a Db, transport: &'a mut T) -> Self {
        Self {
            db,
            transport,
            batch_size: 50,
            max_attempts: 5,
            retry_delay_seconds: 30,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.clamp(1, 100);
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: i64) -> Self {
        self.max_attempts = max_attempts.max(1);
        self
    }

    pub fn with_retry_delay_seconds(mut self, retry_delay_seconds: i64) -> Self {
        self.retry_delay_seconds = retry_delay_seconds.max(1);
        self
    }

    pub fn sync_once(
        &mut self,
        organization_id: &str,
        now: DateTime<Utc>,
    ) -> Result<SyncReport, SyncError> {
        let mut report = SyncReport::default();
        let pending =
            self.db
                .pending_sync_events(organization_id, &now.to_rfc3339(), self.batch_size)?;
        let events = pending
            .into_iter()
            .map(OutboundEvent::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        report.pushed = events.len();

        if !events.is_empty() {
            match self.transport.push(organization_id, &events) {
                Ok(acks) => {
                    for event in &events {
                        let ack = acks
                            .iter()
                            .find(|ack| ack.idempotency_key == event.idempotency_key);
                        match ack.map(|ack| &ack.outcome) {
                            Some(PushOutcome::Accepted) => {
                                if self.db.mark_sync_event_sent(
                                    organization_id,
                                    &event.event_id,
                                    &now.to_rfc3339(),
                                )? {
                                    report.accepted += 1;
                                }
                            }
                            Some(PushOutcome::Rejected(error)) => {
                                self.record_failure(
                                    organization_id,
                                    event,
                                    error,
                                    now,
                                    &mut report,
                                )?;
                            }
                            None => {
                                self.record_failure(
                                    organization_id,
                                    event,
                                    "the server did not acknowledge the event",
                                    now,
                                    &mut report,
                                )?;
                            }
                        }
                    }
                }
                Err(error) => {
                    for event in &events {
                        self.record_failure(organization_id, event, &error, now, &mut report)?;
                    }
                    report.transport_error = Some(error);
                }
            }
        }

        let cursor = self.db.sync_cursor(organization_id)?;
        match self
            .transport
            .pull(organization_id, cursor, self.batch_size)
        {
            Ok(inbound) => {
                report.pulled = inbound.len();
                for event in inbound {
                    let payload_json = serde_json::to_string(&event.payload).map_err(|error| {
                        SyncError::InvalidPayload(format!(
                            "inbound event {}: {error}",
                            event.event_id
                        ))
                    })?;
                    let applied = self.db.apply_inbox_event(
                        organization_id,
                        InboundSyncEvent {
                            event_id: event.event_id,
                            server_sequence: event.server_sequence,
                            event_type: event.event_type,
                            aggregate_type: event.aggregate_type,
                            aggregate_id: event.aggregate_id,
                            idempotency_key: event.idempotency_key,
                            payload_json,
                            occurred_at: event.occurred_at,
                        },
                    )?;
                    if applied {
                        report.applied += 1;
                    } else {
                        report.duplicates += 1;
                    }
                }
            }
            Err(error) => {
                report.transport_error = Some(error);
            }
        }

        Ok(report)
    }

    fn record_failure(
        &self,
        organization_id: &str,
        event: &OutboundEvent,
        error: &str,
        now: DateTime<Utc>,
        report: &mut SyncReport,
    ) -> Result<(), SyncError> {
        let delay = self
            .retry_delay_seconds
            .saturating_mul(event.attempts.saturating_add(1));
        let next_attempt_at = now + Duration::seconds(delay);
        let failure = self.db.fail_sync_event(
            organization_id,
            &event.event_id,
            error,
            &next_attempt_at.to_rfc3339(),
            self.max_attempts,
        )?;
        if failure.state == "dead_letter" {
            report.dead_lettered += 1;
        } else {
            report.retried += 1;
        }
        Ok(())
    }
}
