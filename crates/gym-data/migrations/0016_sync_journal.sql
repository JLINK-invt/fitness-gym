-- Local durable journal for eventual synchronization. Business writes enqueue an
-- outbox row in the same SQLite transaction so a crash cannot lose the event or
-- leave a half-published sale. The inbox records remote events exactly once and
-- advances a cursor; it never applies an event ID or server sequence twice.
CREATE TABLE sync_outbox (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending'
        CHECK (state IN ('pending', 'sent', 'dead_letter')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    last_error TEXT,
    next_attempt_at TEXT NOT NULL,
    sent_at TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, idempotency_key)
) STRICT;

CREATE INDEX idx_sync_outbox_pending
    ON sync_outbox (organization_id, state, next_attempt_at, created_at);

CREATE INDEX idx_sync_outbox_aggregate
    ON sync_outbox (organization_id, aggregate_type, aggregate_id);

CREATE TABLE sync_inbox (
    event_id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    server_sequence INTEGER NOT NULL CHECK (server_sequence >= 1),
    event_type TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    received_at TEXT NOT NULL,
    UNIQUE (organization_id, server_sequence)
) STRICT;

CREATE INDEX idx_sync_inbox_organization_sequence
    ON sync_inbox (organization_id, server_sequence);

CREATE TABLE sync_state (
    organization_id TEXT PRIMARY KEY NOT NULL
        REFERENCES organizations (id) ON DELETE CASCADE,
    last_server_sequence INTEGER NOT NULL DEFAULT 0
        CHECK (last_server_sequence >= 0),
    updated_at TEXT NOT NULL
) STRICT;
