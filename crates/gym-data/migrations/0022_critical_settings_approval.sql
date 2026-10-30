-- Fase 5: critical gym settings require a second active user to approve the
-- change. Requests store the validated proposal; approval applies it and the
-- partial index keeps at most one pending request per gym.
CREATE TABLE settings_change_requests (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    requested_by TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    requested_by_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'rejected')),
    proposed_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    reviewed_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    reviewed_by_name TEXT,
    reviewed_at TEXT
) STRICT;

CREATE UNIQUE INDEX idx_settings_change_one_pending
    ON settings_change_requests (organization_id)
    WHERE status = 'pending';

CREATE INDEX idx_settings_change_organization_created
    ON settings_change_requests (organization_id, created_at);
