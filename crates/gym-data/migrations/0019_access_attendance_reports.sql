CREATE TABLE member_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    member_id TEXT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    issued_at TEXT NOT NULL,
    revoked_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (organization_id, code),
    CHECK (
        (status = 'revoked' AND revoked_at IS NOT NULL)
        OR (status = 'active' AND revoked_at IS NULL)
    )
) STRICT;

CREATE INDEX idx_member_credentials_member
    ON member_credentials (organization_id, member_id, status);

CREATE TRIGGER member_credentials_no_delete
BEFORE DELETE ON member_credentials
BEGIN
    SELECT RAISE(ABORT, 'member credentials cannot be deleted');
END;

INSERT INTO member_credentials
    (id, organization_id, member_id, code, status, revision, issued_at,
     revoked_at, created_at, updated_at)
SELECT lower(hex(randomblob(16))), organization_id, id,
       'GYM-' || member_number || '-' || upper(substr(replace(id, '-', ''), 1, 8)),
       'active', 1, joined_at, NULL, created_at, updated_at
FROM members;

CREATE TABLE access_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    generated_at TEXT NOT NULL,
    checksum TEXT NOT NULL,
    signature TEXT NOT NULL,
    member_count INTEGER NOT NULL CHECK (member_count >= 0),
    credential_count INTEGER NOT NULL CHECK (credential_count >= 0),
    payload_json TEXT NOT NULL,
    UNIQUE (organization_id, revision)
) STRICT;

CREATE INDEX idx_access_snapshots_organization_revision
    ON access_snapshots (organization_id, revision DESC);

CREATE TRIGGER access_snapshots_no_update
BEFORE UPDATE ON access_snapshots
BEGIN
    SELECT RAISE(ABORT, 'access snapshots are immutable');
END;

CREATE TRIGGER access_snapshots_no_delete
BEFORE DELETE ON access_snapshots
BEGIN
    SELECT RAISE(ABORT, 'access snapshots are immutable');
END;

CREATE TABLE access_events (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    credential_code TEXT,
    member_id TEXT REFERENCES members(id) ON DELETE SET NULL,
    membership_id TEXT REFERENCES memberships(id) ON DELETE SET NULL,
    device_id TEXT NOT NULL,
    zone TEXT,
    allowed INTEGER NOT NULL CHECK (allowed IN (0, 1)),
    reason TEXT NOT NULL,
    snapshot_revision INTEGER NOT NULL DEFAULT 0 CHECK (snapshot_revision >= 0),
    idempotency_key TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    reconciliation_status TEXT NOT NULL CHECK (
        reconciliation_status IN ('applied', 'stale', 'legacy')
    ),
    occurred_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, idempotency_key)
) STRICT;

CREATE INDEX idx_access_events_organization_occurred
    ON access_events (organization_id, occurred_at DESC);

CREATE INDEX idx_access_events_member_occurred
    ON access_events (organization_id, member_id, occurred_at DESC);

CREATE TRIGGER access_events_no_update
BEFORE UPDATE ON access_events
BEGIN
    SELECT RAISE(ABORT, 'access events are append-only');
END;

CREATE TRIGGER access_events_no_delete
BEFORE DELETE ON access_events
BEGIN
    SELECT RAISE(ABORT, 'access events are append-only');
END;

INSERT INTO access_events
    (id, organization_id, credential_code, member_id, membership_id,
     device_id, zone, allowed, reason, snapshot_revision, idempotency_key,
     payload_hash, reconciliation_status, occurred_at, created_at)
SELECT id, organization_id, NULL, NULL, NULL, 'legacy', NULL,
       CASE WHEN event_type = 'access.allowed' THEN 1 ELSE 0 END,
       event_type, 0, idempotency_key, payload_hash, 'legacy', created_at, created_at
FROM module_events
WHERE aggregate = 'access_decision';

CREATE TABLE report_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    report_type TEXT NOT NULL CHECK (report_type IN ('financial')),
    cadence TEXT NOT NULL CHECK (cadence IN ('daily', 'weekly', 'monthly')),
    lookback_days INTEGER NOT NULL CHECK (lookback_days BETWEEN 1 AND 366),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    next_run_at TEXT NOT NULL,
    last_run_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_report_schedules_due
    ON report_schedules (organization_id, enabled, next_run_at);

CREATE TABLE report_runs (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    schedule_id TEXT REFERENCES report_schedules(id) ON DELETE SET NULL,
    report_type TEXT NOT NULL,
    period_from TEXT NOT NULL,
    period_to TEXT NOT NULL,
    generated_at TEXT NOT NULL,
    file_name TEXT,
    checksum TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('completed', 'failed')),
    error TEXT
) STRICT;

CREATE INDEX idx_report_runs_organization_generated
    ON report_runs (organization_id, generated_at DESC);

CREATE TRIGGER report_runs_no_update
BEFORE UPDATE ON report_runs
BEGIN
    SELECT RAISE(ABORT, 'report runs are immutable');
END;

CREATE TRIGGER report_runs_no_delete
BEFORE DELETE ON report_runs
BEGIN
    SELECT RAISE(ABORT, 'report runs are immutable');
END;
