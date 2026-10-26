-- Charges and refunds are recorded as immutable payment facts tied to a sale or
-- a return. The external reference is the bridge to a terminal, bank movement or
-- manual receipt; no card data is ever stored. Refund retries need their own
-- idempotency key so a repeated request cannot move money twice.
ALTER TABLE sale_returns ADD COLUMN idempotency_key TEXT;
ALTER TABLE sale_returns ADD COLUMN request_hash TEXT;

CREATE UNIQUE INDEX idx_sale_returns_organization_idempotency
    ON sale_returns (organization_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE payment_charges (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    sale_id TEXT NOT NULL REFERENCES sales (id) ON DELETE RESTRICT,
    sale_return_id TEXT REFERENCES sale_returns (id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK (kind IN ('charge', 'refund')),
    method TEXT NOT NULL CHECK (method IN ('cash', 'transfer', 'point')),
    provider TEXT NOT NULL CHECK (provider IN ('manual', 'point')),
    status TEXT NOT NULL CHECK (status IN ('approved')),
    amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    external_reference TEXT CHECK (
        external_reference IS NULL
        OR (length(external_reference) BETWEEN 1 AND 120)
    ),
    idempotency_key TEXT NOT NULL CHECK (length(idempotency_key) BETWEEN 1 AND 128),
    request_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    CHECK (
        (kind = 'charge' AND sale_return_id IS NULL)
        OR (kind = 'refund' AND sale_return_id IS NOT NULL)
    )
) STRICT;

CREATE INDEX idx_payment_charges_organization_created
    ON payment_charges (organization_id, created_at DESC);

CREATE INDEX idx_payment_charges_sale ON payment_charges (sale_id);
CREATE INDEX idx_payment_charges_return ON payment_charges (sale_return_id);

CREATE INDEX idx_payment_charges_organization_reference
    ON payment_charges (organization_id, kind, external_reference)
    WHERE external_reference IS NOT NULL;

CREATE UNIQUE INDEX idx_payment_charges_organization_idempotency
    ON payment_charges (organization_id, kind, idempotency_key);

CREATE TRIGGER payment_charges_no_update
BEFORE UPDATE ON payment_charges
BEGIN
    SELECT RAISE(ABORT, 'payment charges are append-only');
END;

CREATE TRIGGER payment_charges_no_delete
BEFORE DELETE ON payment_charges
BEGIN
    SELECT RAISE(ABORT, 'payment charges are append-only');
END;
