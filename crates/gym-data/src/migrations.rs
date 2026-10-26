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
    Migration {
        version: 7,
        name: "sales",
        up: include_str!("../migrations/0007_sales.sql"),
        down: "DROP TRIGGER IF EXISTS sale_items_no_delete; \
               DROP TRIGGER IF EXISTS sale_items_no_update; \
               DROP TRIGGER IF EXISTS sales_no_delete; \
               DROP TRIGGER IF EXISTS sales_no_update; \
               DROP INDEX IF EXISTS idx_sale_items_variant; \
               DROP INDEX IF EXISTS idx_sale_items_sale; \
               DROP TABLE IF EXISTS sale_items; \
               DROP INDEX IF EXISTS idx_sales_organization_created; \
               DROP TABLE IF EXISTS sales;",
    },
    Migration {
        version: 8,
        name: "owner_recovery",
        up: include_str!("../migrations/0008_owner_recovery.sql"),
        down: "ALTER TABLE users DROP COLUMN recovery_code_hash;",
    },
    Migration {
        version: 9,
        name: "cash_sessions",
        up: include_str!("../migrations/0009_cash_sessions.sql"),
        down: "DROP TRIGGER IF EXISTS cash_sessions_no_reopen; \
               DROP TRIGGER IF EXISTS cash_sessions_immutable_fields; \
               DROP TRIGGER IF EXISTS cash_sessions_no_delete; \
               DROP INDEX IF EXISTS idx_sales_cash_session; \
               ALTER TABLE sales DROP COLUMN cash_session_id; \
               DROP INDEX IF EXISTS idx_cash_sessions_organization_opened; \
               DROP INDEX IF EXISTS idx_cash_sessions_one_open; \
               DROP TABLE IF EXISTS cash_sessions;",
    },
    Migration {
        version: 10,
        name: "cash_movements",
        up: include_str!("../migrations/0010_cash_movements.sql"),
        down: "DROP TRIGGER IF EXISTS cash_movements_no_update; \
               DROP TRIGGER IF EXISTS cash_movements_no_delete; \
               DROP TRIGGER IF EXISTS cash_sessions_require_difference_reason; \
               DROP INDEX IF EXISTS idx_cash_movements_organization_created; \
               DROP INDEX IF EXISTS idx_cash_movements_session; \
               DROP TABLE IF EXISTS cash_movements; \
               ALTER TABLE cash_sessions DROP COLUMN difference_reason;",
    },
    Migration {
        version: 11,
        name: "sale_returns",
        up: include_str!("../migrations/0011_sale_returns.sql"),
        down: "DROP TRIGGER IF EXISTS ledger_entries_no_delete; \
               DROP TRIGGER IF EXISTS ledger_entries_no_update; \
               DROP TRIGGER IF EXISTS sale_return_items_no_delete; \
               DROP TRIGGER IF EXISTS sale_return_items_no_update; \
               DROP TRIGGER IF EXISTS sale_returns_no_delete; \
               DROP TRIGGER IF EXISTS sale_returns_no_update; \
               DROP INDEX IF EXISTS idx_ledger_entries_sale; \
               DROP INDEX IF EXISTS idx_ledger_entries_organization_occurred; \
               DROP TABLE IF EXISTS ledger_entries; \
               DROP INDEX IF EXISTS idx_sale_return_items_sale_item; \
               DROP INDEX IF EXISTS idx_sale_return_items_return; \
               DROP TABLE IF EXISTS sale_return_items; \
               DROP INDEX IF EXISTS idx_sale_returns_organization_created; \
               DROP INDEX IF EXISTS idx_sale_returns_session; \
               DROP INDEX IF EXISTS idx_sale_returns_sale; \
               DROP TABLE IF EXISTS sale_returns;",
    },
    Migration {
        version: 12,
        name: "inventory",
        up: include_str!("../migrations/0012_inventory.sql"),
        down: "DROP TRIGGER IF EXISTS inventory_movements_no_delete; \
               DROP TRIGGER IF EXISTS inventory_movements_no_update; \
               DROP INDEX IF EXISTS idx_inventory_movements_transfer_group; \
               DROP INDEX IF EXISTS idx_inventory_movements_source; \
               DROP INDEX IF EXISTS idx_inventory_movements_organization_created; \
               DROP INDEX IF EXISTS idx_inventory_movements_location_created; \
               DROP INDEX IF EXISTS idx_inventory_movements_variant_created; \
               DROP VIEW IF EXISTS inventory_balances; \
               DROP TABLE IF EXISTS inventory_policies; \
               DROP TABLE IF EXISTS inventory_movements; \
               DROP INDEX IF EXISTS idx_inventory_locations_default; \
               DROP INDEX IF EXISTS idx_inventory_locations_organization_name; \
               DROP TABLE IF EXISTS inventory_locations;",
    },
    Migration {
        version: 13,
        name: "single_location",
        up: include_str!("../migrations/0013_single_location.sql"),
        down: "Restore a pre-v13 backup; the multi-location tables were removed.",
    },
    Migration {
        version: 14,
        name: "sale_idempotency",
        up: include_str!("../migrations/0014_sale_idempotency.sql"),
        down: "DROP INDEX IF EXISTS idx_sales_organization_idempotency; \
               ALTER TABLE sales DROP COLUMN request_hash; \
               ALTER TABLE sales DROP COLUMN idempotency_key;",
    },
    Migration {
        version: 15,
        name: "payment_charges",
        up: include_str!("../migrations/0015_payment_charges.sql"),
        down: "DROP TRIGGER IF EXISTS payment_charges_no_delete; \
               DROP TRIGGER IF EXISTS payment_charges_no_update; \
               DROP INDEX IF EXISTS idx_payment_charges_organization_idempotency; \
               DROP INDEX IF EXISTS idx_payment_charges_organization_reference; \
               DROP INDEX IF EXISTS idx_payment_charges_return; \
               DROP INDEX IF EXISTS idx_payment_charges_sale; \
               DROP INDEX IF EXISTS idx_payment_charges_organization_created; \
               DROP TABLE IF EXISTS payment_charges; \
               DROP INDEX IF EXISTS idx_sale_returns_organization_idempotency; \
               ALTER TABLE sale_returns DROP COLUMN request_hash; \
               ALTER TABLE sale_returns DROP COLUMN idempotency_key;",
    },
    Migration {
        version: 16,
        name: "sync_journal",
        up: include_str!("../migrations/0016_sync_journal.sql"),
        down: "DROP INDEX IF EXISTS idx_sync_inbox_organization_sequence; \
               DROP TABLE IF EXISTS sync_state; \
               DROP TABLE IF EXISTS sync_inbox; \
               DROP INDEX IF EXISTS idx_sync_outbox_aggregate; \
               DROP INDEX IF EXISTS idx_sync_outbox_pending; \
               DROP TABLE IF EXISTS sync_outbox;",
    },
];
