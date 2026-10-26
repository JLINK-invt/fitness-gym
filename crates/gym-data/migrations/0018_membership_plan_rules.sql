CREATE TABLE membership_plan_versions (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    plan_id TEXT NOT NULL REFERENCES membership_plans(id) ON DELETE RESTRICT,
    version INTEGER NOT NULL CHECK (version > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 2 AND 80),
    description TEXT,
    duration_days INTEGER NOT NULL CHECK (duration_days BETWEEN 1 AND 3660),
    price_minor INTEGER NOT NULL CHECK (price_minor BETWEEN 0 AND 99999999999),
    currency TEXT NOT NULL CHECK (length(currency) = 3),
    grace_days INTEGER NOT NULL CHECK (grace_days BETWEEN 0 AND 90),
    access_days TEXT NOT NULL CHECK (length(trim(access_days)) > 0),
    access_start_minute INTEGER NOT NULL CHECK (access_start_minute BETWEEN 0 AND 1439),
    access_end_minute INTEGER NOT NULL CHECK (access_end_minute BETWEEN 1 AND 1440),
    min_age INTEGER CHECK (min_age IS NULL OR min_age BETWEEN 0 AND 120),
    max_age INTEGER CHECK (max_age IS NULL OR max_age BETWEEN 0 AND 120),
    created_at TEXT NOT NULL,
    UNIQUE (plan_id, version),
    CHECK (max_age IS NULL OR min_age IS NULL OR max_age >= min_age),
    CHECK (access_start_minute < access_end_minute)
) STRICT;

CREATE INDEX idx_membership_plan_versions_plan
    ON membership_plan_versions(plan_id, version DESC);

CREATE TRIGGER membership_plan_versions_no_update
BEFORE UPDATE ON membership_plan_versions
BEGIN
    SELECT RAISE(ABORT, 'membership plan versions are immutable');
END;

CREATE TRIGGER membership_plan_versions_no_delete
BEFORE DELETE ON membership_plan_versions
BEGIN
    SELECT RAISE(ABORT, 'membership plan versions are immutable');
END;

ALTER TABLE membership_plans ADD COLUMN current_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE membership_plans ADD COLUMN access_days TEXT NOT NULL DEFAULT 'mon,tue,wed,thu,fri,sat,sun';
ALTER TABLE membership_plans ADD COLUMN access_start_minute INTEGER NOT NULL DEFAULT 0;
ALTER TABLE membership_plans ADD COLUMN access_end_minute INTEGER NOT NULL DEFAULT 1440;
ALTER TABLE membership_plans ADD COLUMN min_age INTEGER;
ALTER TABLE membership_plans ADD COLUMN max_age INTEGER;

INSERT INTO membership_plan_versions
    (id, organization_id, plan_id, version, name, description, duration_days,
     price_minor, currency, grace_days, access_days, access_start_minute,
     access_end_minute, min_age, max_age, created_at)
SELECT lower(hex(randomblob(16))), organization_id, id, 1, name, description,
       duration_days, price_minor, currency, grace_days, access_days,
       access_start_minute, access_end_minute, min_age, max_age, updated_at
FROM membership_plans;

ALTER TABLE memberships ADD COLUMN plan_version_id TEXT REFERENCES membership_plan_versions(id) ON DELETE RESTRICT;
ALTER TABLE memberships ADD COLUMN plan_version INTEGER;
ALTER TABLE memberships ADD COLUMN access_days TEXT;
ALTER TABLE memberships ADD COLUMN access_start_minute INTEGER;
ALTER TABLE memberships ADD COLUMN access_end_minute INTEGER;
ALTER TABLE memberships ADD COLUMN min_age INTEGER;
ALTER TABLE memberships ADD COLUMN max_age INTEGER;

UPDATE memberships
SET plan_version_id = (
        SELECT v.id FROM membership_plan_versions v
        WHERE v.plan_id = memberships.plan_id AND v.version = 1
    ),
    plan_version = 1,
    access_days = (
        SELECT v.access_days FROM membership_plan_versions v
        WHERE v.plan_id = memberships.plan_id AND v.version = 1
    ),
    access_start_minute = (
        SELECT v.access_start_minute FROM membership_plan_versions v
        WHERE v.plan_id = memberships.plan_id AND v.version = 1
    ),
    access_end_minute = (
        SELECT v.access_end_minute FROM membership_plan_versions v
        WHERE v.plan_id = memberships.plan_id AND v.version = 1
    ),
    min_age = (
        SELECT v.min_age FROM membership_plan_versions v
        WHERE v.plan_id = memberships.plan_id AND v.version = 1
    ),
    max_age = (
        SELECT v.max_age FROM membership_plan_versions v
        WHERE v.plan_id = memberships.plan_id AND v.version = 1
    );

ALTER TABLE membership_payments ADD COLUMN receipt_number TEXT;
ALTER TABLE membership_payments ADD COLUMN cash_session_id TEXT REFERENCES cash_sessions(id) ON DELETE RESTRICT;

UPDATE membership_payments AS p
SET receipt_number = 'REC-' || printf('%05d', (
    SELECT COUNT(*) FROM membership_payments x
    WHERE x.organization_id = p.organization_id
      AND (x.created_at < p.created_at OR (x.created_at = p.created_at AND x.id <= p.id))
))
WHERE p.receipt_number IS NULL;

CREATE UNIQUE INDEX idx_membership_payments_receipt
    ON membership_payments(organization_id, receipt_number)
    WHERE receipt_number IS NOT NULL;

ALTER TABLE cash_movements ADD COLUMN membership_payment_id TEXT REFERENCES membership_payments(id) ON DELETE RESTRICT;

CREATE UNIQUE INDEX idx_cash_movements_membership_payment
    ON cash_movements(membership_payment_id)
    WHERE membership_payment_id IS NOT NULL;
