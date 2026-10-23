-- Inventory is derived from immutable movements. No stock counter is edited:
-- entries, sales, returns, adjustments and transfers are new rows only.
CREATE TABLE inventory_locations (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_inventory_locations_organization_name
    ON inventory_locations (organization_id, name COLLATE NOCASE);

CREATE UNIQUE INDEX idx_inventory_locations_default
    ON inventory_locations (organization_id)
    WHERE is_default = 1;

INSERT INTO inventory_locations
    (id, organization_id, name, is_default, active, created_at)
SELECT lower(hex(randomblob(16))), id, 'Principal', 1, 1,
       strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM organizations;

CREATE TABLE inventory_movements (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    variant_id TEXT NOT NULL REFERENCES catalog_variants (id) ON DELETE RESTRICT,
    location_id TEXT NOT NULL REFERENCES inventory_locations (id) ON DELETE RESTRICT,
    movement_type TEXT NOT NULL
        CHECK (movement_type IN ('entry', 'sale', 'return', 'adjustment', 'transfer')),
    quantity_delta INTEGER NOT NULL
        CHECK (quantity_delta BETWEEN -999999 AND 999999 AND quantity_delta <> 0),
    unit_cost_minor INTEGER NOT NULL CHECK (unit_cost_minor >= 0),
    total_cost_minor INTEGER NOT NULL
        CHECK (total_cost_minor = quantity_delta * unit_cost_minor),
    source_type TEXT NOT NULL
        CHECK (source_type IN ('manual', 'sale', 'sale_return', 'adjustment', 'transfer')),
    source_id TEXT,
    reference TEXT CHECK (reference IS NULL OR length(trim(reference)) BETWEEN 1 AND 120),
    note TEXT CHECK (note IS NULL OR length(trim(note)) BETWEEN 1 AND 300),
    transfer_group_id TEXT,
    authorized_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX idx_inventory_movements_variant_created
    ON inventory_movements (variant_id, created_at DESC);

CREATE INDEX idx_inventory_movements_location_created
    ON inventory_movements (location_id, created_at DESC);

CREATE INDEX idx_inventory_movements_organization_created
    ON inventory_movements (organization_id, created_at DESC);

CREATE INDEX idx_inventory_movements_source
    ON inventory_movements (source_type, source_id);

CREATE INDEX idx_inventory_movements_transfer_group
    ON inventory_movements (transfer_group_id)
    WHERE transfer_group_id IS NOT NULL;

CREATE TABLE inventory_policies (
    variant_id TEXT PRIMARY KEY NOT NULL REFERENCES catalog_variants (id) ON DELETE RESTRICT,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    min_stock INTEGER NOT NULL DEFAULT 0 CHECK (min_stock BETWEEN 0 AND 999999),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users (id) ON DELETE SET NULL
) STRICT;

CREATE VIEW inventory_balances AS
SELECT organization_id,
       variant_id,
       location_id,
       SUM(quantity_delta) AS quantity,
       SUM(total_cost_minor) AS value_minor,
       CASE
           WHEN SUM(quantity_delta) > 0
               THEN CAST(ROUND(CAST(SUM(total_cost_minor) AS REAL)
                               / SUM(quantity_delta)) AS INTEGER)
           ELSE 0
       END AS avg_unit_cost_minor
FROM inventory_movements
GROUP BY organization_id, variant_id, location_id;

CREATE TRIGGER inventory_movements_no_update
BEFORE UPDATE ON inventory_movements
BEGIN
    SELECT RAISE(ABORT, 'inventory movements are append-only');
END;

CREATE TRIGGER inventory_movements_no_delete
BEFORE DELETE ON inventory_movements
BEGIN
    SELECT RAISE(ABORT, 'inventory movements are append-only');
END;
