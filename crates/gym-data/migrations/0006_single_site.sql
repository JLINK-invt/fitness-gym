-- One installation is one gym location. Remove the site dimension from staff,
-- audit history and the schema. Existing audit rows are kept as historical
-- records, but their legacy hashes are cleared because the canonical hash no
-- longer includes a site. The verifiable chain restarts at the next event.

DROP INDEX IF EXISTS idx_audit_organization_site_sequence;
DROP TRIGGER IF EXISTS audit_events_no_update;
DROP TRIGGER IF EXISTS audit_events_no_delete;

UPDATE audit_events SET prev_hash = NULL, event_hash = NULL;
ALTER TABLE audit_events DROP COLUMN site_id;

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

CREATE TABLE staff_memberships_single_site (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('owner', 'manager', 'cashier', 'accounting')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked')),
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, user_id)
) STRICT;

INSERT INTO staff_memberships_single_site
    (id, organization_id, user_id, role, status, created_at)
SELECT id, organization_id, user_id, role, status, created_at
FROM staff_memberships;

DROP TABLE staff_memberships;
ALTER TABLE staff_memberships_single_site RENAME TO staff_memberships;

CREATE INDEX idx_staff_organization ON staff_memberships (organization_id);
CREATE INDEX idx_staff_user ON staff_memberships (organization_id, user_id);
CREATE UNIQUE INDEX idx_staff_one_active_owner
    ON staff_memberships (organization_id)
    WHERE role = 'owner' AND status = 'active';
CREATE UNIQUE INDEX idx_staff_one_membership_per_user
    ON staff_memberships (organization_id, user_id);

DROP TABLE sites;
