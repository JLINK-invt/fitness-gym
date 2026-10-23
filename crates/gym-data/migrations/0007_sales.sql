-- Sales are immutable snapshots of what was charged. Every line keeps the
-- catalog price version, tax rate and display names it used, so later catalog
-- edits never rewrite a completed sale.
CREATE TABLE sales (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    subtotal_minor INTEGER NOT NULL CHECK (subtotal_minor >= 0),
    tax_minor INTEGER NOT NULL CHECK (tax_minor >= 0),
    total_minor INTEGER NOT NULL CHECK (total_minor >= 0),
    payment_method TEXT NOT NULL CHECK (payment_method IN ('cash', 'transfer', 'point')),
    cash_received_minor INTEGER CHECK (cash_received_minor IS NULL OR cash_received_minor >= 0),
    change_minor INTEGER CHECK (change_minor IS NULL OR change_minor >= 0),
    sold_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    CHECK (subtotal_minor + tax_minor = total_minor),
    CHECK (
        (payment_method = 'cash'
            AND cash_received_minor IS NOT NULL
            AND change_minor IS NOT NULL
            AND cash_received_minor - change_minor = total_minor)
        OR
        (payment_method <> 'cash'
            AND cash_received_minor IS NULL
            AND change_minor IS NULL)
    )
) STRICT;

CREATE INDEX idx_sales_organization_created
    ON sales (organization_id, created_at DESC);

CREATE TABLE sale_items (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    sale_id TEXT NOT NULL REFERENCES sales (id) ON DELETE RESTRICT,
    position INTEGER NOT NULL CHECK (position >= 0),
    product_id TEXT NOT NULL REFERENCES catalog_products (id) ON DELETE RESTRICT,
    variant_id TEXT NOT NULL REFERENCES catalog_variants (id) ON DELETE RESTRICT,
    price_version_id TEXT NOT NULL REFERENCES catalog_price_versions (id) ON DELETE RESTRICT,
    product_name TEXT NOT NULL,
    variant_name TEXT NOT NULL,
    sku TEXT NOT NULL,
    unit TEXT NOT NULL CHECK (unit IN ('piece', 'service', 'session', 'package')),
    quantity INTEGER NOT NULL CHECK (quantity BETWEEN 1 AND 999),
    unit_amount_minor INTEGER NOT NULL CHECK (unit_amount_minor >= 0),
    tax_rate_bps INTEGER NOT NULL CHECK (tax_rate_bps BETWEEN 0 AND 10000),
    price_includes_tax INTEGER NOT NULL CHECK (price_includes_tax IN (0, 1)),
    subtotal_minor INTEGER NOT NULL CHECK (subtotal_minor >= 0),
    tax_minor INTEGER NOT NULL CHECK (tax_minor >= 0),
    total_minor INTEGER NOT NULL CHECK (total_minor >= 0),
    UNIQUE (sale_id, position),
    UNIQUE (sale_id, variant_id),
    CHECK (subtotal_minor + tax_minor = total_minor)
) STRICT;

CREATE INDEX idx_sale_items_sale ON sale_items (sale_id);
CREATE INDEX idx_sale_items_variant ON sale_items (variant_id);

CREATE TRIGGER sales_no_update
BEFORE UPDATE ON sales
BEGIN
    SELECT RAISE(ABORT, 'sales are append-only');
END;

CREATE TRIGGER sales_no_delete
BEFORE DELETE ON sales
BEGIN
    SELECT RAISE(ABORT, 'sales are append-only');
END;

CREATE TRIGGER sale_items_no_update
BEFORE UPDATE ON sale_items
BEGIN
    SELECT RAISE(ABORT, 'sale items are append-only');
END;

CREATE TRIGGER sale_items_no_delete
BEFORE DELETE ON sale_items
BEGIN
    SELECT RAISE(ABORT, 'sale items are append-only');
END;
