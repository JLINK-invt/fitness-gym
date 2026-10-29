-- Fase 4.1: operational modules that close the desktop menu. Gym settings keep
-- the business data and operating rules, integration_links describe the
-- external modules connected by versioned contracts, classes and bookings
-- schedule the gym agenda, campaigns track commercial actions and equipment
-- with maintenance_orders keep the gym running.
CREATE TABLE gym_settings (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL UNIQUE REFERENCES organizations (id) ON DELETE CASCADE,
    business_name TEXT NOT NULL,
    legal_name TEXT,
    tax_id TEXT,
    currency TEXT NOT NULL DEFAULT 'MXN',
    timezone TEXT NOT NULL DEFAULT 'America/Mexico_City',
    locale TEXT NOT NULL DEFAULT 'es-MX',
    opening_time TEXT NOT NULL DEFAULT '06:00',
    closing_time TEXT NOT NULL DEFAULT '22:00',
    slot_minutes INTEGER NOT NULL DEFAULT 60 CHECK (slot_minutes BETWEEN 15 AND 240),
    grace_minutes INTEGER NOT NULL DEFAULT 10 CHECK (grace_minutes BETWEEN 0 AND 120),
    inactivity_days INTEGER NOT NULL DEFAULT 30 CHECK (inactivity_days BETWEEN 1 AND 365),
    frozen_days INTEGER NOT NULL DEFAULT 0 CHECK (frozen_days BETWEEN 0 AND 365),
    updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE integration_links (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    module_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    contract_version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'connected'
        CHECK (status IN ('connected', 'degraded', 'disconnected')),
    capabilities TEXT NOT NULL DEFAULT '[]',
    last_seen_at TEXT,
    last_event_at TEXT,
    last_snapshot_at TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (organization_id, module_id)
) STRICT;

CREATE INDEX idx_integration_links_organization
    ON integration_links (organization_id, status);

CREATE TABLE classes (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    coach_user_id TEXT REFERENCES users (id) ON DELETE SET NULL,
    weekday INTEGER NOT NULL CHECK (weekday BETWEEN 0 AND 6),
    start_time TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL CHECK (duration_minutes BETWEEN 15 AND 480),
    capacity INTEGER NOT NULL CHECK (capacity BETWEEN 1 AND 500),
    room TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'cancelled')),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_classes_organization
    ON classes (organization_id, weekday, start_time);

CREATE TABLE class_bookings (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    class_id TEXT NOT NULL REFERENCES classes (id) ON DELETE CASCADE,
    member_id TEXT NOT NULL REFERENCES members (id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'booked'
        CHECK (status IN ('booked', 'attended', 'cancelled', 'no_show')),
    note TEXT,
    booked_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (class_id, member_id)
) STRICT;

CREATE INDEX idx_class_bookings_class
    ON class_bookings (organization_id, class_id, status);

CREATE INDEX idx_class_bookings_member
    ON class_bookings (organization_id, member_id, status);

CREATE TABLE campaigns (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    channel TEXT NOT NULL
        CHECK (channel IN ('whatsapp', 'sms', 'email', 'in_app', 'referral')),
    audience TEXT NOT NULL DEFAULT 'all',
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'active', 'paused', 'completed')),
    starts_on TEXT,
    ends_on TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_campaigns_organization
    ON campaigns (organization_id, status);

CREATE TABLE equipment (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    code TEXT,
    area TEXT,
    status TEXT NOT NULL DEFAULT 'operational'
        CHECK (status IN ('operational', 'maintenance', 'out_of_service')),
    acquired_on TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_equipment_organization
    ON equipment (organization_id, status);

CREATE TABLE maintenance_orders (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    equipment_id TEXT REFERENCES equipment (id) ON DELETE SET NULL,
    kind TEXT NOT NULL CHECK (kind IN ('preventive', 'corrective')),
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'in_progress', 'completed', 'cancelled')),
    priority TEXT NOT NULL DEFAULT 'normal'
        CHECK (priority IN ('low', 'normal', 'high')),
    summary TEXT NOT NULL,
    detail TEXT,
    due_on TEXT,
    reported_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_maintenance_orders_organization
    ON maintenance_orders (organization_id, status, due_on);
