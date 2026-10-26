-- Returns are new compensating operations. They reference the original sale and
-- the shift where the money leaves the drawer; no sale or sale item is edited.
CREATE TABLE sale_returns (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    sale_id TEXT NOT NULL REFERENCES sales (id) ON DELETE RESTRICT,
    cash_session_id TEXT NOT NULL REFERENCES cash_sessions (id) ON DELETE RESTRICT,
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    refund_method TEXT NOT NULL CHECK (refund_method IN ('cash', 'transfer', 'point')),
    total_minor INTEGER NOT NULL CHECK (total_minor > 0 AND total_minor <= 1000000000),
    reason TEXT NOT NULL CHECK (length(trim(reason)) BETWEEN 1 AND 300),
    authorized_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_sale_returns_sale
    ON sale_returns (sale_id, created_at DESC);

CREATE INDEX idx_sale_returns_session
    ON sale_returns (cash_session_id, created_at DESC);

CREATE INDEX idx_sale_returns_organization_created
    ON sale_returns (organization_id, created_at DESC);

CREATE TABLE sale_return_items (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    return_id TEXT NOT NULL REFERENCES sale_returns (id) ON DELETE RESTRICT,
    sale_item_id TEXT NOT NULL REFERENCES sale_items (id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK (quantity BETWEEN 1 AND 999),
    amount_minor INTEGER NOT NULL CHECK (amount_minor >= 0),
    UNIQUE (return_id, sale_item_id)
) STRICT;

CREATE INDEX idx_sale_return_items_return ON sale_return_items (return_id);
CREATE INDEX idx_sale_return_items_sale_item ON sale_return_items (sale_item_id);

-- Immutable double-entry style journal for money movements. Sales add a positive
-- entry and returns add a negative one; reports can rebuild totals from here.
CREATE TABLE ledger_entries (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    entry_type TEXT NOT NULL CHECK (entry_type IN ('sale', 'refund')),
    reference_id TEXT NOT NULL,
    sale_id TEXT NOT NULL REFERENCES sales (id) ON DELETE RESTRICT,
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    amount_minor INTEGER NOT NULL,
    occurred_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (entry_type, reference_id)
) STRICT;

CREATE INDEX idx_ledger_entries_organization_occurred
    ON ledger_entries (organization_id, occurred_at DESC);

CREATE INDEX idx_ledger_entries_sale ON ledger_entries (sale_id);

INSERT INTO ledger_entries
    (id, organization_id, entry_type, reference_id, sale_id, currency,
     amount_minor, occurred_at, created_at)
SELECT lower(hex(randomblob(16))), organization_id, 'sale', id, id, currency,
       total_minor, created_at, created_at
FROM sales;

CREATE TRIGGER sale_returns_no_update
BEFORE UPDATE ON sale_returns
BEGIN
    SELECT RAISE(ABORT, 'sale returns are append-only');
END;

CREATE TRIGGER sale_returns_no_delete
BEFORE DELETE ON sale_returns
BEGIN
    SELECT RAISE(ABORT, 'sale returns are append-only');
END;

CREATE TRIGGER sale_return_items_no_update
BEFORE UPDATE ON sale_return_items
BEGIN
    SELECT RAISE(ABORT, 'sale return items are append-only');
END;

CREATE TRIGGER sale_return_items_no_delete
BEFORE DELETE ON sale_return_items
BEGIN
    SELECT RAISE(ABORT, 'sale return items are append-only');
END;

CREATE TRIGGER ledger_entries_no_update
BEFORE UPDATE ON ledger_entries
BEGIN
    SELECT RAISE(ABORT, 'ledger entries are append-only');
END;

CREATE TRIGGER ledger_entries_no_delete
BEFORE DELETE ON ledger_entries
BEGIN
    SELECT RAISE(ABORT, 'ledger entries are append-only');
END;
