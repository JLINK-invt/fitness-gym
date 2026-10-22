CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 2 AND 80),
    password_hash TEXT NOT NULL CHECK (length(password_hash) > 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    last_login_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_users_organization_name
    ON users (organization_id, name COLLATE NOCASE);

CREATE INDEX idx_users_organization ON users (organization_id);

-- A gym has one active owner. Other roles may be scoped to a site.
CREATE UNIQUE INDEX idx_staff_one_active_owner
    ON staff_memberships (organization_id)
    WHERE role = 'owner' AND status = 'active';

CREATE INDEX idx_staff_user ON staff_memberships (organization_id, user_id);
