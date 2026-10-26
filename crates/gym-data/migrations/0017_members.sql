CREATE TABLE membership_plans (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    duration_days INTEGER NOT NULL CHECK (duration_days BETWEEN 1 AND 3660),
    price_minor INTEGER NOT NULL CHECK (price_minor BETWEEN 0 AND 1000000000),
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    grace_days INTEGER NOT NULL DEFAULT 0 CHECK (grace_days BETWEEN 0 AND 90),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (organization_id, name)
) STRICT;

CREATE TABLE members (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    member_number TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    phone TEXT,
    email TEXT,
    birth_date TEXT,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive', 'archived')),
    joined_at TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (organization_id, member_number)
) STRICT;

CREATE INDEX idx_members_organization_name
    ON members (organization_id, last_name COLLATE NOCASE, first_name COLLATE NOCASE);

CREATE INDEX idx_members_organization_status
    ON members (organization_id, status);

CREATE TABLE prospects (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    phone TEXT,
    email TEXT,
    source TEXT,
    status TEXT NOT NULL CHECK (status IN ('new', 'contacted', 'trial', 'converted', 'lost')),
    notes TEXT,
    converted_member_id TEXT REFERENCES members(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_prospects_organization_status
    ON prospects (organization_id, status, created_at DESC);

CREATE TABLE memberships (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    member_id TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    plan_id TEXT NOT NULL REFERENCES membership_plans(id) ON DELETE RESTRICT,
    plan_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('scheduled', 'active', 'paused', 'expired', 'cancelled')),
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    price_minor INTEGER NOT NULL CHECK (price_minor BETWEEN 0 AND 1000000000),
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    grace_days INTEGER NOT NULL CHECK (grace_days BETWEEN 0 AND 90),
    paused_at TEXT,
    pause_until TEXT,
    cancellation_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (end_date >= start_date),
    CHECK (
        (status = 'paused' AND paused_at IS NOT NULL AND pause_until IS NOT NULL)
        OR (status <> 'paused' AND paused_at IS NULL AND pause_until IS NULL)
    ),
    CHECK (status <> 'cancelled' OR cancellation_reason IS NOT NULL)
) STRICT;

CREATE UNIQUE INDEX idx_memberships_one_current
    ON memberships (organization_id, member_id)
    WHERE status IN ('scheduled', 'active', 'paused');

CREATE INDEX idx_memberships_organization_end
    ON memberships (organization_id, end_date);

CREATE INDEX idx_memberships_member_created
    ON memberships (member_id, created_at DESC);

CREATE TABLE membership_events (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    membership_id TEXT NOT NULL REFERENCES memberships(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (
        event_type IN ('created', 'renewed', 'paused', 'resumed', 'cancelled', 'expired')
    ),
    from_status TEXT,
    to_status TEXT NOT NULL,
    note TEXT,
    actor_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_membership_events_membership
    ON membership_events (membership_id, created_at);

CREATE TRIGGER membership_events_no_update
BEFORE UPDATE ON membership_events
BEGIN
    SELECT RAISE(ABORT, 'membership events are append-only');
END;

CREATE TRIGGER membership_events_no_delete
BEFORE DELETE ON membership_events
BEGIN
    SELECT RAISE(ABORT, 'membership events are append-only');
END;

CREATE TABLE membership_charges (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    membership_id TEXT NOT NULL REFERENCES memberships(id) ON DELETE RESTRICT,
    member_id TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    description TEXT NOT NULL,
    amount_minor INTEGER NOT NULL CHECK (amount_minor BETWEEN 1 AND 1000000000),
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    due_date TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'paid', 'void')),
    paid_minor INTEGER NOT NULL DEFAULT 0 CHECK (paid_minor BETWEEN 0 AND amount_minor),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_membership_charges_member_status
    ON membership_charges (organization_id, member_id, status, due_date);

CREATE INDEX idx_membership_charges_membership
    ON membership_charges (membership_id, created_at);

CREATE TRIGGER membership_charges_immutable_terms
BEFORE UPDATE OF organization_id, membership_id, member_id, description,
                 amount_minor, currency, due_date, created_at ON membership_charges
BEGIN
    SELECT RAISE(ABORT, 'membership charge terms are immutable');
END;

CREATE TRIGGER membership_charges_no_delete
BEFORE DELETE ON membership_charges
BEGIN
    SELECT RAISE(ABORT, 'membership charges cannot be deleted');
END;

CREATE TABLE membership_payments (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    charge_id TEXT NOT NULL REFERENCES membership_charges(id) ON DELETE RESTRICT,
    membership_id TEXT NOT NULL REFERENCES memberships(id) ON DELETE RESTRICT,
    member_id TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    amount_minor INTEGER NOT NULL CHECK (amount_minor BETWEEN 1 AND 1000000000),
    payment_method TEXT NOT NULL CHECK (payment_method IN ('cash', 'transfer', 'point')),
    reference TEXT,
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    received_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, idempotency_key)
) STRICT;

CREATE INDEX idx_membership_payments_member_created
    ON membership_payments (organization_id, member_id, created_at DESC);

CREATE INDEX idx_membership_payments_charge
    ON membership_payments (charge_id, created_at);

CREATE TRIGGER membership_payments_no_update
BEFORE UPDATE ON membership_payments
BEGIN
    SELECT RAISE(ABORT, 'membership payments are append-only');
END;

CREATE TRIGGER membership_payments_no_delete
BEFORE DELETE ON membership_payments
BEGIN
    SELECT RAISE(ABORT, 'membership payments are append-only');
END;
