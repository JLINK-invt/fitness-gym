/// Ordered, forward-only migrations. Each version is applied once inside a
/// transaction; `down` documents the manual rollback path for runbooks.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: &'static str,
    pub down: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_core_schema",
        up: include_str!("../migrations/0001_initial_core_schema.sql"),
        down: "DROP TABLE IF EXISTS module_events; \
               DROP TABLE IF EXISTS audit_events; \
               DROP TABLE IF EXISTS backup_snapshots; \
               DROP TABLE IF EXISTS staff_memberships; \
               DROP TABLE IF EXISTS sites; \
               DROP TABLE IF EXISTS organizations;",
    },
    Migration {
        version: 2,
        name: "local_users",
        up: include_str!("../migrations/0002_local_users.sql"),
        down: "DROP INDEX IF EXISTS idx_staff_user; \
               DROP INDEX IF EXISTS idx_staff_one_active_owner; \
               DROP INDEX IF EXISTS idx_users_organization; \
               DROP INDEX IF EXISTS idx_users_organization_name; \
               DROP TABLE IF EXISTS users;",
    },
    Migration {
        version: 3,
        name: "sites_staff",
        up: include_str!("../migrations/0003_sites_staff.sql"),
        down: "DROP INDEX IF EXISTS idx_staff_one_membership_per_user; \
               DROP INDEX IF EXISTS idx_sites_organization_name;",
    },
    Migration {
        version: 4,
        name: "immutable_audit",
        up: include_str!("../migrations/0004_immutable_audit.sql"),
        down: "DROP TRIGGER IF EXISTS audit_events_no_delete; \
               DROP TRIGGER IF EXISTS audit_events_no_update; \
               DROP INDEX IF EXISTS idx_audit_organization_site_sequence; \
               DROP INDEX IF EXISTS idx_audit_organization_sequence; \
               ALTER TABLE audit_events DROP COLUMN event_hash; \
               ALTER TABLE audit_events DROP COLUMN prev_hash; \
               ALTER TABLE audit_events DROP COLUMN origin; \
               ALTER TABLE audit_events DROP COLUMN site_id; \
               ALTER TABLE audit_events DROP COLUMN sequence;",
    },
    Migration {
        version: 5,
        name: "catalog_pricing",
        up: include_str!("../migrations/0005_catalog_pricing.sql"),
        down: "DROP TRIGGER IF EXISTS catalog_price_versions_no_delete; \
               DROP TRIGGER IF EXISTS catalog_price_versions_no_update; \
               DROP INDEX IF EXISTS idx_catalog_price_versions_variant_effective; \
               DROP TABLE IF EXISTS catalog_price_versions; \
               DROP INDEX IF EXISTS idx_catalog_variants_product; \
               DROP INDEX IF EXISTS idx_catalog_variants_organization_barcode; \
               DROP INDEX IF EXISTS idx_catalog_variants_organization_sku; \
               DROP TABLE IF EXISTS catalog_variants; \
               DROP INDEX IF EXISTS idx_catalog_products_organization_name; \
               DROP TABLE IF EXISTS catalog_products;",
    },
    Migration {
        version: 6,
        name: "single_site",
        up: include_str!("../migrations/0006_single_site.sql"),
        down: "-- irreversible: restore a snapshot created before schema v6",
    },
];
