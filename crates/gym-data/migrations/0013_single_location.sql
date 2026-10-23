DROP VIEW IF EXISTS inventory_balances;

ALTER TABLE inventory_movements RENAME TO inventory_movements_multi_location;

CREATE TABLE inventory_movements (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    variant_id TEXT NOT NULL REFERENCES catalog_variants (id) ON DELETE RESTRICT,
    movement_type TEXT NOT NULL
        CHECK (movement_type IN ('entry', 'sale', 'return', 'adjustment')),
    quantity_delta INTEGER NOT NULL
        CHECK (quantity_delta BETWEEN -999999 AND 999999 AND quantity_delta <> 0),
    unit_cost_minor INTEGER NOT NULL CHECK (unit_cost_minor >= 0),
    total_cost_minor INTEGER NOT NULL
        CHECK (total_cost_minor = quantity_delta * unit_cost_minor),
    source_type TEXT NOT NULL
        CHECK (source_type IN ('manual', 'sale', 'sale_return', 'adjustment')),
    source_id TEXT,
    reference TEXT CHECK (reference IS NULL OR length(trim(reference)) BETWEEN 1 AND 120),
    note TEXT CHECK (note IS NULL OR length(trim(note)) BETWEEN 1 AND 300),
    authorized_by TEXT REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT NOT NULL
) STRICT;

INSERT INTO inventory_movements
    (id, organization_id, variant_id, movement_type, quantity_delta,
     unit_cost_minor, total_cost_minor, source_type, source_id, reference,
     note, authorized_by, created_at)
SELECT id, organization_id, variant_id, movement_type, quantity_delta,
       unit_cost_minor, total_cost_minor, source_type, source_id, reference,
       note, authorized_by, created_at
FROM inventory_movements_multi_location
WHERE movement_type <> 'transfer' AND source_type <> 'transfer';

DROP TABLE inventory_movements_multi_location;
DROP TABLE inventory_locations;

CREATE INDEX idx_inventory_movements_variant_created
    ON inventory_movements (variant_id, created_at DESC);

CREATE INDEX idx_inventory_movements_organization_created
    ON inventory_movements (organization_id, created_at DESC);

CREATE INDEX idx_inventory_movements_source
    ON inventory_movements (source_type, source_id);

CREATE VIEW inventory_balances AS
SELECT organization_id,
       variant_id,
       SUM(quantity_delta) AS quantity,
       SUM(total_cost_minor) AS value_minor,
       CASE
           WHEN SUM(quantity_delta) > 0
               THEN CAST(ROUND(CAST(SUM(total_cost_minor) AS REAL)
                               / SUM(quantity_delta)) AS INTEGER)
           ELSE 0
       END AS avg_unit_cost_minor
FROM inventory_movements
GROUP BY organization_id, variant_id;

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
