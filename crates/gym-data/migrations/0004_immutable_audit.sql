ALTER TABLE audit_events ADD COLUMN sequence INTEGER;
ALTER TABLE audit_events ADD COLUMN site_id TEXT;
ALTER TABLE audit_events ADD COLUMN origin TEXT NOT NULL DEFAULT 'legacy';
ALTER TABLE audit_events ADD COLUMN prev_hash TEXT;
ALTER TABLE audit_events ADD COLUMN event_hash TEXT;

UPDATE audit_events
SET sequence = (
    SELECT COUNT(*)
    FROM audit_events AS earlier
    WHERE earlier.organization_id = audit_events.organization_id
      AND (
          earlier.created_at < audit_events.created_at
          OR (earlier.created_at = audit_events.created_at AND earlier.id <= audit_events.id)
      )
);

CREATE UNIQUE INDEX idx_audit_organization_sequence
    ON audit_events (organization_id, sequence);

CREATE INDEX idx_audit_organization_site_sequence
    ON audit_events (organization_id, site_id, sequence);

CREATE TRIGGER audit_events_no_update
BEFORE UPDATE ON audit_events
BEGIN
    SELECT RAISE(ABORT, 'audit_events is append-only');
END;

CREATE TRIGGER audit_events_no_delete
BEFORE DELETE ON audit_events
BEGIN
    SELECT RAISE(ABORT, 'audit_events is append-only');
END;
