-- Catalog is organization-wide. Price rows are append-only: a sale references
-- the exact version it charged, so editing the catalog never rewrites history.
CREATE TABLE catalog_products (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 2 AND 80),
    category TEXT CHECK (category IS NULL OR length(trim(category)) BETWEEN 2 AND 60),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_catalog_products_organization_name
    ON catalog_products (organization_id, name COLLATE NOCASE);

CREATE TABLE catalog_variants (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    product_id TEXT NOT NULL REFERENCES catalog_products (id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
    sku TEXT NOT NULL CHECK (length(trim(sku)) BETWEEN 1 AND 40),
    barcode TEXT CHECK (barcode IS NULL OR length(trim(barcode)) BETWEEN 1 AND 64),
    unit TEXT NOT NULL CHECK (unit IN ('piece', 'service', 'session', 'package')),
    cost_minor INTEGER NOT NULL DEFAULT 0 CHECK (cost_minor >= 0),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_catalog_variants_organization_sku
    ON catalog_variants (organization_id, sku COLLATE NOCASE);

CREATE UNIQUE INDEX idx_catalog_variants_organization_barcode
    ON catalog_variants (organization_id, barcode)
    WHERE barcode IS NOT NULL;

CREATE INDEX idx_catalog_variants_product
    ON catalog_variants (product_id);

CREATE TABLE catalog_price_versions (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    variant_id TEXT NOT NULL REFERENCES catalog_variants (id) ON DELETE RESTRICT,
    currency TEXT NOT NULL CHECK (length(currency) = 3 AND currency = upper(currency)),
    amount_minor INTEGER NOT NULL CHECK (amount_minor >= 0),
    tax_rate_bps INTEGER NOT NULL CHECK (tax_rate_bps BETWEEN 0 AND 10000),
    price_includes_tax INTEGER NOT NULL CHECK (price_includes_tax IN (0, 1)),
    effective_from TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users (id) ON DELETE SET NULL
) STRICT;

CREATE INDEX idx_catalog_price_versions_variant_effective
    ON catalog_price_versions (variant_id, effective_from DESC, created_at DESC);

CREATE TRIGGER catalog_price_versions_no_update
BEFORE UPDATE ON catalog_price_versions
BEGIN
    SELECT RAISE(ABORT, 'catalog price versions are append-only');
END;

CREATE TRIGGER catalog_price_versions_no_delete
BEFORE DELETE ON catalog_price_versions
BEGIN
    SELECT RAISE(ABORT, 'catalog price versions are append-only');
END;
