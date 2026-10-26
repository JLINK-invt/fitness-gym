-- A cash shift is opened once, closed once and never deleted. Expected cash is
-- always derived from immutable cash sales linked to the shift, so closing does
-- not duplicate the drawer arithmetic. The partial unique index guarantees a
-- single open shift per installation.
CREATE TABLE cash_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    status TEXT NOT NULL CHECK (status IN ('open', 'closed')),
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    opening_amount_minor INTEGER NOT NULL CHECK (opening_amount_minor >= 0),
    counted_amount_minor INTEGER CHECK (counted_amount_minor IS NULL OR counted_amount_minor >= 0),
    difference_minor INTEGER,
    opened_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    opened_at TEXT NOT NULL,
    closed_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    closed_at TEXT,
    CHECK (
        (status = 'open'
            AND counted_amount_minor IS NULL
            AND difference_minor IS NULL
            AND closed_by IS NULL
            AND closed_at IS NULL)
        OR
        (status = 'closed'
            AND counted_amount_minor IS NOT NULL
            AND difference_minor IS NOT NULL
            AND closed_at IS NOT NULL)
    )
) STRICT;

CREATE UNIQUE INDEX idx_cash_sessions_one_open
    ON cash_sessions (organization_id)
    WHERE status = 'open';

CREATE INDEX idx_cash_sessions_organization_opened
    ON cash_sessions (organization_id, opened_at DESC);

ALTER TABLE sales
    ADD COLUMN cash_session_id TEXT REFERENCES cash_sessions (id) ON DELETE RESTRICT;

CREATE INDEX idx_sales_cash_session ON sales (cash_session_id);

CREATE TRIGGER cash_sessions_no_delete
BEFORE DELETE ON cash_sessions
BEGIN
    SELECT RAISE(ABORT, 'cash sessions cannot be deleted');
END;

CREATE TRIGGER cash_sessions_immutable_fields
BEFORE UPDATE ON cash_sessions
WHEN NEW.id <> OLD.id
    OR NEW.organization_id <> OLD.organization_id
    OR NEW.currency <> OLD.currency
    OR NEW.opening_amount_minor <> OLD.opening_amount_minor
    OR NEW.opened_by IS NOT OLD.opened_by
    OR NEW.opened_at <> OLD.opened_at
BEGIN
    SELECT RAISE(ABORT, 'cash session identity is immutable');
END;

CREATE TRIGGER cash_sessions_no_reopen
BEFORE UPDATE ON cash_sessions
WHEN OLD.status = 'closed'
BEGIN
    SELECT RAISE(ABORT, 'cash sessions cannot be reopened');
END;
