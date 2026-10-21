CREATE TABLE organizations (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'closed')),
    timezone TEXT NOT NULL DEFAULT 'America/Mexico_City',
    currency TEXT NOT NULL DEFAULT 'MXN',
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE sites (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'closed')),
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_sites_organization ON sites (organization_id);

CREATE TABLE staff_memberships (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    site_id TEXT REFERENCES sites (id) ON DELETE SET NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('owner', 'manager', 'cashier', 'accounting')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked')),
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, site_id, user_id)
) STRICT;

CREATE INDEX idx_staff_organization ON staff_memberships (organization_id);

CREATE TABLE audit_events (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    actor_id TEXT,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('allowed', 'denied', 'failed')),
    correlation_id TEXT,
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_audit_organization_created ON audit_events (organization_id, created_at);

CREATE TABLE module_events (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    direction TEXT NOT NULL CHECK (direction IN ('inbound', 'outbound')),
    event_type TEXT NOT NULL,
    aggregate TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'applied', 'failed')),
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, idempotency_key)
) STRICT;

CREATE INDEX idx_module_events_state ON module_events (organization_id, state);

CREATE TABLE backup_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT REFERENCES organizations (id) ON DELETE SET NULL,
    file_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    checksum TEXT NOT NULL,
    destination TEXT NOT NULL CHECK (destination IN ('local', 'cloud')),
    status TEXT NOT NULL CHECK (status IN ('completed', 'failed')),
    error TEXT
) STRICT;

CREATE INDEX idx_backup_snapshots_created ON backup_snapshots (created_at);
