-- Cash income and expense entries belong to an open shift and are append-only.
-- Only cash-method entries change the expected drawer; transfer and Point stay
-- in the shift for reconciliation and reporting.
ALTER TABLE cash_sessions ADD COLUMN difference_reason TEXT;

CREATE TABLE cash_movements (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    cash_session_id TEXT NOT NULL REFERENCES cash_sessions (id) ON DELETE RESTRICT,
    movement_type TEXT NOT NULL CHECK (movement_type IN ('income', 'expense')),
    category TEXT NOT NULL CHECK (length(trim(category)) BETWEEN 1 AND 80),
    amount_minor INTEGER NOT NULL CHECK (amount_minor > 0 AND amount_minor <= 1000000000),
    payment_method TEXT NOT NULL CHECK (payment_method IN ('cash', 'transfer', 'point')),
    evidence TEXT NOT NULL CHECK (length(trim(evidence)) BETWEEN 1 AND 200),
    authorized_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_cash_movements_session
    ON cash_movements (cash_session_id, created_at DESC);

CREATE INDEX idx_cash_movements_organization_created
    ON cash_movements (organization_id, created_at DESC);

CREATE TRIGGER cash_sessions_require_difference_reason
BEFORE UPDATE ON cash_sessions
WHEN NEW.status = 'closed'
    AND NEW.difference_minor <> 0
    AND (NEW.difference_reason IS NULL OR length(trim(NEW.difference_reason)) = 0)
BEGIN
    SELECT RAISE(ABORT, 'cash differences require a reason');
END;

CREATE TRIGGER cash_movements_no_delete
BEFORE DELETE ON cash_movements
BEGIN
    SELECT RAISE(ABORT, 'cash movements cannot be deleted');
END;

CREATE TRIGGER cash_movements_no_update
BEFORE UPDATE ON cash_movements
BEGIN
    SELECT RAISE(ABORT, 'cash movements cannot be updated');
END;
