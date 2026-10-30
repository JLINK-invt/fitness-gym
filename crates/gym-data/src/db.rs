use gym_contracts::DatabaseStatus;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::DataError;
use crate::key::DbKey;
use crate::migrations::MIGRATIONS;
use crate::sha256_hex;

mod access;
mod members;
mod operations;
mod reports;
pub use access::{
    AccessConflictRecord, AccessEventRecord, AccessEventWriteOutcome, AccessReconciliationRecord,
    AccessSnapshotRecord, AttendanceDayRecord, AttendanceHourRecord, AttendanceMemberRecord,
    AttendancePlanRecord, AttendanceReportRecord, AttendanceZoneRecord, CredentialAccessRuleRecord,
    MemberCredentialRecord, NewAccessEvent,
};
pub use members::{
    MemberAccountRecord, MemberRecord, MembershipChargeRecord, MembershipDashboardRecord,
    MembershipDashboardTrendRecord, MembershipPaymentReceiptRecord, MembershipPaymentRecord,
    MembershipPlanRecord, MembershipRecord, NewMember, NewMembershipPayment, NewMembershipPlan,
    NewProspect, ProspectRecord, UpdateMember, UpdateMembershipPlan, UpdateProspect,
};
pub use operations::{
    CampaignRecord, ClassBookingRecord, ClassRecord, EquipmentRecord, FunnelStageRecord,
    GymSettingsRecord, IntegrationJournalSummary, IntegrationLinkRecord, MaintenanceOrderRecord,
    MaintenanceOrderUpdate, MarketingFunnelRecord, NewCampaign, NewClass, NewEquipment,
    NewIntegrationLink, NewMaintenanceOrder, UpdateClass, UpdateEquipment, UpdateGymSettings,
};
pub use reports::{NewReportSchedule, ReportRunRecord, ReportScheduleRecord};

pub type LocalDatabaseStatus = DatabaseStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationRecord {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserCredentialRecord {
    pub user_id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub name: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerRecoveryRecord {
    pub user_id: String,
    pub name: String,
    pub recovery_code_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffRecord {
    pub user_id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub last_login_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionOverrideRecord {
    pub permission: String,
    pub effect: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogPriceRecord {
    pub id: String,
    pub currency: String,
    pub amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub effective_from: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogVariantRecord {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub status: String,
    pub current_price: Option<CatalogPriceRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProductRecord {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub status: String,
    pub variants: Vec<CatalogVariantRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogPrice {
    pub amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub effective_from: Option<String>,
}

struct ValidatedCatalogPrice {
    amount_minor: i64,
    tax_rate_bps: i64,
    price_includes_tax: bool,
    effective_from: String,
}

const MAX_AMOUNT_MINOR: i64 = 1_000_000_000;

struct SaleVariantRow {
    product_id: String,
    product_name: String,
    variant_name: String,
    sku: String,
    unit: String,
    variant_status: String,
    product_status: String,
    price_version_id: Option<String>,
    amount_minor: Option<i64>,
    tax_rate_bps: Option<i64>,
    price_includes_tax: Option<i64>,
}

struct SaleLine {
    position: usize,
    product_id: String,
    variant_id: String,
    price_version_id: String,
    product_name: String,
    variant_name: String,
    sku: String,
    unit: String,
    quantity: i64,
    unit_amount_minor: i64,
    tax_rate_bps: i64,
    price_includes_tax: bool,
    subtotal_minor: i64,
    tax_minor: i64,
    total_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogItem {
    pub name: String,
    pub category: Option<String>,
    pub variant_name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub price: NewCatalogPrice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogVariant {
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub price: NewCatalogPrice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogProductUpdate {
    pub name: String,
    pub category: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogVariantUpdate {
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSaleItem {
    pub variant_id: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSale {
    pub items: Vec<NewSaleItem>,
    pub payment_method: String,
    pub cash_received_minor: Option<i64>,
    pub payment_reference: Option<String>,
    pub idempotency_key: String,
}

impl NewSale {
    fn request_hash(&self) -> String {
        let mut items: Vec<(&str, i64)> = self
            .items
            .iter()
            .map(|item| (item.variant_id.as_str(), item.quantity))
            .collect();
        items.sort_unstable();
        let cash = self
            .cash_received_minor
            .map(|amount| amount.to_string())
            .unwrap_or_else(|| "none".into());
        let reference = self.payment_reference.as_deref().unwrap_or("none");
        let mut canonical = format!("{}|{cash}|{reference}", self.payment_method);
        for (variant_id, quantity) in items {
            canonical.push('\n');
            canonical.push_str(variant_id);
            canonical.push('|');
            canonical.push_str(&quantity.to_string());
        }
        crate::sha256_hex(canonical)
    }

    fn provider(&self) -> &'static str {
        if self.payment_method == "point" {
            "point"
        } else {
            "manual"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSaleReturnItem {
    pub sale_item_id: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSaleReturn {
    pub sale_id: String,
    pub reason: String,
    pub items: Vec<NewSaleReturnItem>,
    pub payment_reference: Option<String>,
    pub idempotency_key: String,
}

impl NewSaleReturn {
    fn request_hash(&self) -> String {
        let mut items: Vec<(&str, i64)> = self
            .items
            .iter()
            .map(|item| (item.sale_item_id.as_str(), item.quantity))
            .collect();
        items.sort_unstable();
        let reference = self.payment_reference.as_deref().unwrap_or("none");
        let mut canonical = format!("{}|{}|{reference}", self.sale_id, self.reason.trim());
        for (sale_item_id, quantity) in items {
            canonical.push('\n');
            canonical.push_str(sale_item_id);
            canonical.push('|');
            canonical.push_str(&quantity.to_string());
        }
        crate::sha256_hex(canonical)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleItemRecord {
    pub id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub unit: String,
    pub quantity: i64,
    pub unit_amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub subtotal_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub refunded_quantity: i64,
    pub refunded_minor: i64,
    pub price_version_id: String,
    pub variant_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleRecord {
    pub id: String,
    pub currency: String,
    pub subtotal_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub refunded_minor: i64,
    pub payment_method: String,
    pub cash_received_minor: Option<i64>,
    pub change_minor: Option<i64>,
    pub sold_by: Option<String>,
    pub sold_by_name: Option<String>,
    pub cash_session_id: Option<String>,
    pub payment_reference: Option<String>,
    pub created_at: String,
    pub items: Vec<SaleItemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleReturnItemRecord {
    pub id: String,
    pub sale_item_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub quantity: i64,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleReturnRecord {
    pub id: String,
    pub sale_id: String,
    pub cash_session_id: String,
    pub currency: String,
    pub refund_method: String,
    pub total_minor: i64,
    pub reason: String,
    pub authorized_by: Option<String>,
    pub authorized_by_name: Option<String>,
    pub payment_reference: Option<String>,
    pub created_at: String,
    pub items: Vec<SaleReturnItemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentChargeRecord {
    pub id: String,
    pub sale_id: String,
    pub sale_return_id: Option<String>,
    pub kind: String,
    pub method: String,
    pub provider: String,
    pub status: String,
    pub amount_minor: i64,
    pub currency: String,
    pub external_reference: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinancialReportRecord {
    pub from: String,
    pub to: String,
    pub generated_at: String,
    pub currency: String,
    pub sales_count: i64,
    pub gross_sales_minor: i64,
    pub tax_minor: i64,
    pub refunds_count: i64,
    pub refunded_minor: i64,
    pub net_sales_minor: i64,
    pub average_sale_minor: i64,
    pub attach_rate_bps: i64,
    pub items_per_sale_milli: i64,
    pub cost_of_goods_minor: i64,
    pub margin_minor: i64,
    pub margin_bps: i64,
    pub inventory_value_start_minor: i64,
    pub inventory_value_end_minor: i64,
    pub inventory_turnover_bps: i64,
    pub adjustments_units: i64,
    pub adjustments_cost_minor: i64,
    pub payment_methods: Vec<PaymentMethodTotalRecord>,
    pub cash_sessions_count: i64,
    pub open_cash_sessions: i64,
    pub closed_cash_sessions: i64,
    pub cash_difference_minor: i64,
    pub cash_difference_abs_minor: i64,
    pub product_sales: Vec<ProductSalesRecord>,
    pub inventory_adjustments: Vec<InventoryAdjustmentRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentMethodTotalRecord {
    pub payment_method: String,
    pub sales_count: i64,
    pub gross_sales_minor: i64,
    pub refunds_count: i64,
    pub refunded_minor: i64,
    pub net_sales_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductSalesRecord {
    pub variant_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub unit: String,
    pub sold_units: i64,
    pub refunded_units: i64,
    pub net_units: i64,
    pub gross_sales_minor: i64,
    pub refunded_minor: i64,
    pub net_sales_minor: i64,
    pub cost_minor: i64,
    pub margin_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryAdjustmentRecord {
    pub variant_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub units_delta: i64,
    pub cost_minor: i64,
}

struct ProductAccumulator {
    variant_id: String,
    product_name: String,
    variant_name: String,
    sku: String,
    unit: String,
    sold_units: i64,
    refunded_units: i64,
    gross_sales_minor: i64,
    refunded_minor: i64,
    cost_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItemRecord {
    pub variant_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub unit: String,
    pub stock: i64,
    pub value_minor: i64,
    pub avg_unit_cost_minor: i64,
    pub min_stock: i64,
    pub low_stock: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryMovementRecord {
    pub id: String,
    pub variant_id: String,
    pub movement_type: String,
    pub quantity_delta: i64,
    pub unit_cost_minor: i64,
    pub total_cost_minor: i64,
    pub source_type: String,
    pub source_id: Option<String>,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub authorized_by: Option<String>,
    pub authorized_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInventoryEntry {
    pub variant_id: String,
    pub quantity: i64,
    pub unit_cost_minor: i64,
    pub reference: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInventoryAdjustment {
    pub variant_id: String,
    pub quantity_delta: i64,
    pub unit_cost_minor: Option<i64>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashSessionRecord {
    pub id: String,
    pub status: String,
    pub currency: String,
    pub opening_amount_minor: i64,
    pub expected_cash_minor: i64,
    pub cash_sales_minor: i64,
    pub sales_count: i64,
    pub cash_income_minor: i64,
    pub income_minor: i64,
    pub cash_expense_minor: i64,
    pub expense_minor: i64,
    pub cash_refunds_minor: i64,
    pub refunded_minor: i64,
    pub refunds_count: i64,
    pub movements_count: i64,
    pub counted_amount_minor: Option<i64>,
    pub difference_minor: Option<i64>,
    pub difference_reason: Option<String>,
    pub opened_by: Option<String>,
    pub opened_by_name: Option<String>,
    pub opened_at: String,
    pub closed_by: Option<String>,
    pub closed_by_name: Option<String>,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashMovementRecord {
    pub id: String,
    pub cash_session_id: String,
    pub movement_type: String,
    pub category: String,
    pub amount_minor: i64,
    pub payment_method: String,
    pub evidence: String,
    pub authorized_by: Option<String>,
    pub authorized_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCashMovement {
    pub cash_session_id: String,
    pub movement_type: String,
    pub category: String,
    pub amount_minor: i64,
    pub payment_method: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventRecord {
    pub id: String,
    pub sequence: i64,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub origin: String,
    pub correlation_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditIntegrityRecord {
    pub valid: bool,
    pub checked_events: i64,
    pub first_broken_sequence: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEventRecord {
    pub id: String,
    pub event_type: String,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedModuleEvent {
    pub event: ModuleEventRecord,
    pub inserted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOutboxRecord {
    pub id: String,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub payload_json: String,
    pub payload_hash: String,
    pub state: String,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub next_attempt_at: String,
    pub sent_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncInboxRecord {
    pub event_id: String,
    pub server_sequence: i64,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub payload_json: String,
    pub payload_hash: String,
    pub occurred_at: String,
    pub received_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundSyncEvent {
    pub event_id: String,
    pub server_sequence: i64,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub payload_json: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncFailureRecord {
    pub state: String,
    pub attempts: i64,
}

/// Handle to the local encrypted database. `Mutex<Option<Connection>>` keeps
/// Tauri state `Send + Sync`, serializes writes and allows closing every file
/// handle before a restore replaces the database on disk.
pub struct Db {
    conn: Mutex<Option<Connection>>,
    path: PathBuf,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Db")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Db {
    pub fn open(path: impl AsRef<Path>, key: &DbKey) -> Result<Self, DataError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Self::open_connection(&path, key)?;
        let db = Self {
            conn: Mutex::new(Some(conn)),
            path,
        };
        db.migrate()?;
        Ok(db)
    }

    fn open_connection(path: &Path, key: &DbKey) -> Result<Connection, DataError> {
        let conn = Connection::open(path)?;

        let pragma_key = key.to_pragma_hex();
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", pragma_key.as_str()))?;
        conn.execute_batch("PRAGMA cipher_memory_security = ON;")?;

        let cipher_version: Option<String> = conn
            .query_row("PRAGMA cipher_version", [], |row| row.get(0))
            .optional()?;
        match cipher_version {
            Some(version) if !version.trim().is_empty() => {}
            _ => return Err(DataError::EncryptionUnavailable),
        }

        // Touch the schema so a wrong key fails now, not on the first query.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|_| {
            DataError::InvalidKey("the database cannot be opened with the provided key".into())
        })?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        Ok(conn)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn migrate(&self) -> Result<(), DataError> {
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 version INTEGER PRIMARY KEY NOT NULL,
                 name TEXT NOT NULL,
                 checksum TEXT NOT NULL,
                 applied_at TEXT NOT NULL
             ) STRICT;",
        )?;

        let applied = {
            let mut statement =
                transaction.prepare("SELECT version, checksum FROM schema_migrations")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut map = HashMap::new();
            for row in rows {
                let (version, checksum) = row?;
                map.insert(version, checksum);
            }
            map
        };

        for migration in MIGRATIONS {
            let checksum = hex::encode(Sha256::digest(migration.up.as_bytes()));
            if let Some(existing) = applied.get(&migration.version) {
                if existing != &checksum {
                    return Err(DataError::MigrationDrift {
                        version: migration.version,
                        expected: checksum,
                        found: existing.clone(),
                    });
                }
                continue;
            }

            transaction.execute_batch(migration.up)?;
            transaction.execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    migration.version,
                    migration.name,
                    checksum,
                    chrono::Utc::now().to_rfc3339()
                ],
            )?;
        }

        let latest = MIGRATIONS
            .iter()
            .map(|migration| migration.version)
            .max()
            .unwrap_or(0);
        transaction.pragma_update(None, "user_version", latest)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn status(&self) -> Result<LocalDatabaseStatus, DataError> {
        self.with_connection(|conn| {
            let cipher_version: Option<String> = conn
                .query_row("PRAGMA cipher_version", [], |row| row.get(0))
                .optional()?;
            let integrity: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
            let schema_version: i64 =
                conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
            let organizations: i64 =
                conn.query_row("SELECT count(*) FROM organizations", [], |row| row.get(0))?;

            Ok(LocalDatabaseStatus {
                path: self.path.display().to_string(),
                encrypted: cipher_version.is_some(),
                cipher_version,
                schema_version,
                integrity_ok: integrity.eq_ignore_ascii_case("ok"),
                organizations,
            })
        })
    }

    pub fn create_organization(&self, name: &str) -> Result<String, DataError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(DataError::InvalidValue(
                "organization name cannot be empty".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        self.with_connection(|conn| {
            let exists: bool =
                conn.query_row("SELECT EXISTS(SELECT 1 FROM organizations)", [], |row| {
                    row.get(0)
                })?;
            if exists {
                return Err(DataError::OrganizationAlreadyExists);
            }

            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO organizations (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, now],
            )?;
            Ok(())
        })?;
        Ok(id)
    }

    pub fn organization(&self) -> Result<Option<OrganizationRecord>, DataError> {
        self.with_connection(|conn| {
            conn.query_row("SELECT id, name FROM organizations LIMIT 1", [], |row| {
                Ok(OrganizationRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .optional()
            .map_err(DataError::from)
        })
    }

    pub fn has_owner(&self) -> Result<bool, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM staff_memberships
                     WHERE role = 'owner' AND status = 'active'
                 )",
                [],
                |row| row.get(0),
            )
            .map_err(DataError::from)
        })
    }

    pub fn list_staff(&self, organization_id: &str) -> Result<Vec<StaffRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT u.id, u.name, s.role, s.status, u.last_login_at, u.created_at
                 FROM users u
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1
                 ORDER BY CASE s.role
                     WHEN 'owner' THEN 0
                     WHEN 'manager' THEN 1
                     WHEN 'receptionist' THEN 2
                     WHEN 'advisor' THEN 3
                     WHEN 'trainer' THEN 4
                     ELSE 5
                 END, u.name COLLATE NOCASE",
            )?;
            let rows = statement.query_map(params![organization_id], |row| {
                Ok(StaffRecord {
                    user_id: row.get(0)?,
                    name: row.get(1)?,
                    role: row.get(2)?,
                    status: row.get(3)?,
                    last_login_at: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    pub fn create_staff(
        &self,
        organization_id: &str,
        name: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<StaffRecord, DataError> {
        let name = name.trim();
        if name.is_empty() || password_hash.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "staff name and password hash are required".into(),
            ));
        }
        if !matches!(role, "manager" | "receptionist" | "advisor" | "trainer") {
            return Err(DataError::InvalidValue("invalid staff role".into()));
        }

        let user_id = uuid::Uuid::new_v4().to_string();
        let membership_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        let user_exists: bool = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM users
                 WHERE organization_id = ?1 AND name = ?2 COLLATE NOCASE
             )",
            params![organization_id, name],
            |row| row.get(0),
        )?;
        if user_exists {
            return Err(DataError::UserAlreadyExists);
        }

        transaction.execute(
            "INSERT INTO users
                 (id, organization_id, name, password_hash, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
            params![user_id, organization_id, name, password_hash, created_at],
        )?;
        transaction.execute(
            "INSERT INTO staff_memberships
                 (id, organization_id, user_id, role, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5)",
            params![membership_id, organization_id, user_id, role, created_at],
        )?;
        transaction.commit()?;

        Ok(StaffRecord {
            user_id,
            name: name.to_string(),
            role: role.to_string(),
            status: "active".into(),
            last_login_at: None,
            created_at,
        })
    }

    pub fn update_staff(
        &self,
        organization_id: &str,
        user_id: &str,
        role: &str,
        status: &str,
    ) -> Result<StaffRecord, DataError> {
        if !matches!(role, "manager" | "receptionist" | "advisor" | "trainer") {
            return Err(DataError::InvalidValue("invalid staff role".into()));
        }
        if !matches!(status, "active" | "revoked") {
            return Err(DataError::InvalidValue("invalid staff status".into()));
        }

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        let existing: Option<(String, String, String)> = transaction
            .query_row(
                "SELECT s.role, u.name, u.created_at
                 FROM users u
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1 AND u.id = ?2",
                params![organization_id, user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((current_role, name, created_at)) = existing else {
            return Err(DataError::StaffNotFound);
        };
        if current_role == "owner" {
            return Err(DataError::InvalidValue(
                "the owner membership cannot be modified".into(),
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        transaction.execute(
            "UPDATE staff_memberships
             SET role = ?3, status = ?4
             WHERE organization_id = ?1 AND user_id = ?2",
            params![organization_id, user_id, role, status],
        )?;
        transaction.execute(
            "UPDATE users SET status = ?3, updated_at = ?4
             WHERE organization_id = ?1 AND id = ?2",
            params![
                organization_id,
                user_id,
                if status == "active" {
                    "active"
                } else {
                    "disabled"
                },
                now
            ],
        )?;
        let last_login_at: Option<String> = transaction.query_row(
            "SELECT last_login_at FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        transaction.commit()?;

        Ok(StaffRecord {
            user_id: user_id.to_string(),
            name,
            role: role.to_string(),
            status: status.to_string(),
            last_login_at,
            created_at,
        })
    }

    pub fn list_catalog(
        &self,
        organization_id: &str,
    ) -> Result<Vec<CatalogProductRecord>, DataError> {
        self.load_catalog(organization_id, None)
    }

    pub fn catalog_product(
        &self,
        organization_id: &str,
        product_id: &str,
    ) -> Result<CatalogProductRecord, DataError> {
        self.load_catalog(organization_id, Some(product_id))?
            .into_iter()
            .next()
            .ok_or(DataError::CatalogProductNotFound)
    }

    fn load_catalog(
        &self,
        organization_id: &str,
        product_id: Option<&str>,
    ) -> Result<Vec<CatalogProductRecord>, DataError> {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.with_connection(|conn| {
            let mut product_statement = conn.prepare(
                "SELECT id, name, category, status, created_at, updated_at
                 FROM catalog_products
                 WHERE organization_id = ?1
                   AND (?2 IS NULL OR id = ?2)
                 ORDER BY (status = 'active') DESC, name COLLATE NOCASE",
            )?;
            let products = product_statement
                .query_map(params![organization_id, product_id], |row| {
                    Ok(CatalogProductRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        category: row.get(2)?,
                        status: row.get(3)?,
                        variants: Vec::new(),
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut variant_statement = conn.prepare(
                "SELECT v.id, v.product_id, v.name, v.sku, v.barcode, v.unit,
                        v.cost_minor, v.status, v.created_at, v.updated_at,
                        p.id, p.currency, p.amount_minor, p.tax_rate_bps,
                        p.price_includes_tax, p.effective_from, p.created_at
                 FROM catalog_variants v
                 LEFT JOIN catalog_price_versions p
                   ON p.id = (
                       SELECT id
                       FROM catalog_price_versions
                       WHERE variant_id = v.id AND effective_from <= ?2
                       ORDER BY effective_from DESC, created_at DESC
                       LIMIT 1
                   )
                 WHERE v.organization_id = ?1
                   AND (?3 IS NULL OR v.product_id = ?3)
                 ORDER BY v.name COLLATE NOCASE, v.created_at",
            )?;
            let variants = variant_statement
                .query_map(params![organization_id, now, product_id], |row| {
                    let price_id: Option<String> = row.get(10)?;
                    Ok(CatalogVariantRecord {
                        id: row.get(0)?,
                        product_id: row.get(1)?,
                        name: row.get(2)?,
                        sku: row.get(3)?,
                        barcode: row.get(4)?,
                        unit: row.get(5)?,
                        cost_minor: row.get(6)?,
                        status: row.get(7)?,
                        current_price: price_id
                            .map(|id| {
                                Ok::<CatalogPriceRecord, rusqlite::Error>(CatalogPriceRecord {
                                    id,
                                    currency: row.get(11)?,
                                    amount_minor: row.get(12)?,
                                    tax_rate_bps: row.get(13)?,
                                    price_includes_tax: row.get::<_, i64>(14)? != 0,
                                    effective_from: row.get(15)?,
                                    created_at: row.get(16)?,
                                })
                            })
                            .transpose()?,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut products: Vec<CatalogProductRecord> = products;
            let positions: HashMap<String, usize> = products
                .iter()
                .enumerate()
                .map(|(index, product)| (product.id.clone(), index))
                .collect();
            for variant in variants {
                if let Some(index) = positions.get(&variant.product_id) {
                    products[*index].variants.push(variant);
                }
            }
            Ok(products)
        })
    }

    pub fn create_catalog_item(
        &self,
        organization_id: &str,
        created_by: &str,
        item: NewCatalogItem,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&item.name, "product name", 2, 80)?;
        let category = optional_text(item.category.as_deref(), "category", 2, 60)?;
        let variant_name = required_text(&item.variant_name, "variant name", 1, 80)?;
        let sku = required_text(&item.sku, "SKU", 1, 40)?;
        let barcode = optional_text(item.barcode.as_deref(), "barcode", 1, 64)?;
        validate_catalog_unit(&item.unit)?;
        if item.cost_minor < 0 {
            return Err(DataError::InvalidValue(
                "cost must be zero or greater".into(),
            ));
        }
        let price = validate_new_price(item.price)?;

        let product_id = uuid::Uuid::new_v4().to_string();
        let variant_id = uuid::Uuid::new_v4().to_string();
        let price_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let currency = organization_currency(&transaction, organization_id)?;

            ensure_unique_product_name(&transaction, organization_id, &name, None)?;
            ensure_unique_sku(&transaction, organization_id, &sku, None)?;
            ensure_unique_barcode(&transaction, organization_id, barcode.as_deref(), None)?;

            transaction.execute(
                "INSERT INTO catalog_products
                     (id, organization_id, name, category, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
                params![product_id, organization_id, name, category, now],
            )?;
            transaction.execute(
                "INSERT INTO catalog_variants
                     (id, organization_id, product_id, name, sku, barcode, unit,
                      cost_minor, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9)",
                params![
                    variant_id,
                    organization_id,
                    product_id,
                    variant_name,
                    sku,
                    barcode,
                    item.unit,
                    item.cost_minor,
                    now
                ],
            )?;
            transaction.execute(
                "INSERT INTO catalog_price_versions
                     (id, organization_id, variant_id, currency, amount_minor,
                      tax_rate_bps, price_includes_tax, effective_from, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    price_id,
                    organization_id,
                    variant_id,
                    currency,
                    price.amount_minor,
                    price.tax_rate_bps,
                    i64::from(price.price_includes_tax),
                    price.effective_from,
                    now,
                    created_by
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, &product_id)
    }

    pub fn add_catalog_variant(
        &self,
        organization_id: &str,
        created_by: &str,
        product_id: &str,
        variant: NewCatalogVariant,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&variant.name, "variant name", 1, 80)?;
        let sku = required_text(&variant.sku, "SKU", 1, 40)?;
        let barcode = optional_text(variant.barcode.as_deref(), "barcode", 1, 64)?;
        validate_catalog_unit(&variant.unit)?;
        if variant.cost_minor < 0 {
            return Err(DataError::InvalidValue(
                "cost must be zero or greater".into(),
            ));
        }
        let price = validate_new_price(variant.price)?;
        let variant_id = uuid::Uuid::new_v4().to_string();
        let price_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let currency = organization_currency(&transaction, organization_id)?;
            let product_status: Option<String> = transaction
                .query_row(
                    "SELECT status FROM catalog_products
                     WHERE id = ?1 AND organization_id = ?2",
                    params![product_id, organization_id],
                    |row| row.get(0),
                )
                .optional()?;
            match product_status.as_deref() {
                Some("active") => {}
                Some(_) => {
                    return Err(DataError::InvalidValue(
                        "cannot add variants to an archived product".into(),
                    ))
                }
                None => return Err(DataError::CatalogProductNotFound),
            }

            ensure_unique_sku(&transaction, organization_id, &sku, None)?;
            ensure_unique_barcode(&transaction, organization_id, barcode.as_deref(), None)?;

            transaction.execute(
                "INSERT INTO catalog_variants
                     (id, organization_id, product_id, name, sku, barcode, unit,
                      cost_minor, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9)",
                params![
                    variant_id,
                    organization_id,
                    product_id,
                    name,
                    sku,
                    barcode,
                    variant.unit,
                    variant.cost_minor,
                    now
                ],
            )?;
            transaction.execute(
                "INSERT INTO catalog_price_versions
                     (id, organization_id, variant_id, currency, amount_minor,
                      tax_rate_bps, price_includes_tax, effective_from, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    price_id,
                    organization_id,
                    variant_id,
                    currency,
                    price.amount_minor,
                    price.tax_rate_bps,
                    i64::from(price.price_includes_tax),
                    price.effective_from,
                    now,
                    created_by
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, product_id)
    }

    pub fn update_catalog_product(
        &self,
        organization_id: &str,
        product_id: &str,
        update: NewCatalogProductUpdate,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&update.name, "product name", 2, 80)?;
        let category = optional_text(update.category.as_deref(), "category", 2, 60)?;
        validate_catalog_status(&update.status)?;
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM catalog_products
                     WHERE id = ?1 AND organization_id = ?2
                 )",
                params![product_id, organization_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(DataError::CatalogProductNotFound);
            }
            ensure_unique_product_name(&transaction, organization_id, &name, Some(product_id))?;
            transaction.execute(
                "UPDATE catalog_products
                 SET name = ?3, category = ?4, status = ?5, updated_at = ?6
                 WHERE id = ?1 AND organization_id = ?2",
                params![
                    product_id,
                    organization_id,
                    name,
                    category,
                    update.status,
                    now
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, product_id)
    }

    pub fn update_catalog_variant(
        &self,
        organization_id: &str,
        variant_id: &str,
        update: NewCatalogVariantUpdate,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&update.name, "variant name", 1, 80)?;
        let sku = required_text(&update.sku, "SKU", 1, 40)?;
        let barcode = optional_text(update.barcode.as_deref(), "barcode", 1, 64)?;
        validate_catalog_unit(&update.unit)?;
        validate_catalog_status(&update.status)?;
        if update.cost_minor < 0 {
            return Err(DataError::InvalidValue(
                "cost must be zero or greater".into(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();

        let product_id: String;
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            product_id = transaction
                .query_row(
                    "SELECT product_id FROM catalog_variants
                     WHERE id = ?1 AND organization_id = ?2",
                    params![variant_id, organization_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(DataError::CatalogVariantNotFound)?;

            ensure_unique_sku(&transaction, organization_id, &sku, Some(variant_id))?;
            ensure_unique_barcode(
                &transaction,
                organization_id,
                barcode.as_deref(),
                Some(variant_id),
            )?;
            transaction.execute(
                "UPDATE catalog_variants
                 SET name = ?3, sku = ?4, barcode = ?5, unit = ?6,
                     cost_minor = ?7, status = ?8, updated_at = ?9
                 WHERE id = ?1 AND organization_id = ?2",
                params![
                    variant_id,
                    organization_id,
                    name,
                    sku,
                    barcode,
                    update.unit,
                    update.cost_minor,
                    update.status,
                    now
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, &product_id)
    }

    pub fn add_catalog_price(
        &self,
        organization_id: &str,
        created_by: &str,
        variant_id: &str,
        price: NewCatalogPrice,
    ) -> Result<CatalogProductRecord, DataError> {
        let price = validate_new_price(price)?;
        let price_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let product_id: String;
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let (variant_product_id, variant_status, product_status): (String, String, String) =
                transaction
                    .query_row(
                        "SELECT v.product_id, v.status, p.status
                         FROM catalog_variants v
                         JOIN catalog_products p ON p.id = v.product_id
                         WHERE v.id = ?1 AND v.organization_id = ?2",
                        params![variant_id, organization_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?
                    .ok_or(DataError::CatalogVariantNotFound)?;
            if variant_status != "active" || product_status != "active" {
                return Err(DataError::InvalidValue(
                    "cannot add prices to an archived product or variant".into(),
                ));
            }
            product_id = variant_product_id;
            let currency = organization_currency(&transaction, organization_id)?;
            transaction.execute(
                "INSERT INTO catalog_price_versions
                     (id, organization_id, variant_id, currency, amount_minor,
                      tax_rate_bps, price_includes_tax, effective_from, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    price_id,
                    organization_id,
                    variant_id,
                    currency,
                    price.amount_minor,
                    price.tax_rate_bps,
                    i64::from(price.price_includes_tax),
                    price.effective_from,
                    now,
                    created_by
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, &product_id)
    }

    pub fn list_sales(
        &self,
        organization_id: &str,
        limit: i64,
    ) -> Result<Vec<SaleRecord>, DataError> {
        let limit = limit.clamp(1, 100);
        self.load_sales(organization_id, None, limit)
    }

    pub fn sale(&self, organization_id: &str, sale_id: &str) -> Result<SaleRecord, DataError> {
        self.load_sales(organization_id, Some(sale_id), 1)?
            .into_iter()
            .next()
            .ok_or(DataError::SaleNotFound)
    }

    pub fn current_cash_session(
        &self,
        organization_id: &str,
    ) -> Result<Option<CashSessionRecord>, DataError> {
        self.load_cash_sessions(organization_id, None, Some("open"), 1)
            .map(|sessions| sessions.into_iter().next())
    }

    pub fn cash_session(
        &self,
        organization_id: &str,
        session_id: &str,
    ) -> Result<Option<CashSessionRecord>, DataError> {
        self.load_cash_sessions(organization_id, Some(session_id), None, 1)
            .map(|sessions| sessions.into_iter().next())
    }

    pub fn list_cash_sessions(
        &self,
        organization_id: &str,
        limit: i64,
    ) -> Result<Vec<CashSessionRecord>, DataError> {
        self.load_cash_sessions(organization_id, None, None, limit.clamp(1, 100))
    }

    fn load_cash_sessions(
        &self,
        organization_id: &str,
        session_id: Option<&str>,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CashSessionRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT c.id, c.status, c.currency, c.opening_amount_minor,
                        c.opening_amount_minor
                            + COALESCE(SUM(
                                CASE WHEN s.payment_method = 'cash' THEN s.total_minor ELSE 0 END
                            ), 0)
                            + COALESCE((
                                SELECT SUM(m.amount_minor) FROM cash_movements m
                                WHERE m.cash_session_id = c.id
                                  AND m.movement_type = 'income'
                                  AND m.payment_method = 'cash'
                            ), 0)
                            - COALESCE((
                                SELECT SUM(m.amount_minor) FROM cash_movements m
                                WHERE m.cash_session_id = c.id
                                  AND m.movement_type = 'expense'
                                  AND m.payment_method = 'cash'
                            ), 0)
                            - COALESCE((
                                SELECT SUM(r.total_minor) FROM sale_returns r
                                WHERE r.cash_session_id = c.id
                                  AND r.refund_method = 'cash'
                            ), 0) AS expected_cash_minor,
                        COALESCE(SUM(
                            CASE WHEN s.payment_method = 'cash' THEN s.total_minor ELSE 0 END
                        ), 0) AS cash_sales_minor,
                        COUNT(s.id) AS sales_count,
                        COALESCE((
                            SELECT SUM(m.amount_minor) FROM cash_movements m
                            WHERE m.cash_session_id = c.id
                              AND m.movement_type = 'income'
                              AND m.payment_method = 'cash'
                        ), 0) AS cash_income_minor,
                        COALESCE((
                            SELECT SUM(m.amount_minor) FROM cash_movements m
                            WHERE m.cash_session_id = c.id
                              AND m.movement_type = 'income'
                        ), 0) AS income_minor,
                        COALESCE((
                            SELECT SUM(m.amount_minor) FROM cash_movements m
                            WHERE m.cash_session_id = c.id
                              AND m.movement_type = 'expense'
                              AND m.payment_method = 'cash'
                        ), 0) AS cash_expense_minor,
                        COALESCE((
                            SELECT SUM(m.amount_minor) FROM cash_movements m
                            WHERE m.cash_session_id = c.id
                              AND m.movement_type = 'expense'
                        ), 0) AS expense_minor,
                        COALESCE((
                            SELECT SUM(r.total_minor) FROM sale_returns r
                            WHERE r.cash_session_id = c.id
                              AND r.refund_method = 'cash'
                        ), 0) AS cash_refunds_minor,
                        COALESCE((
                            SELECT SUM(r.total_minor) FROM sale_returns r
                            WHERE r.cash_session_id = c.id
                        ), 0) AS refunded_minor,
                        (
                            SELECT COUNT(*) FROM sale_returns r
                            WHERE r.cash_session_id = c.id
                        ) AS refunds_count,
                        (
                            SELECT COUNT(*) FROM cash_movements m
                            WHERE m.cash_session_id = c.id
                        ) AS movements_count,
                        c.counted_amount_minor, c.difference_minor, c.difference_reason,
                        c.opened_by, ou.name, c.opened_at,
                        c.closed_by, cu.name, c.closed_at
                 FROM cash_sessions c
                 LEFT JOIN users ou ON ou.id = c.opened_by
                 LEFT JOIN users cu ON cu.id = c.closed_by
                 LEFT JOIN sales s ON s.cash_session_id = c.id
                 WHERE c.organization_id = ?1
                   AND (?2 IS NULL OR c.id = ?2)
                   AND (?3 IS NULL OR c.status = ?3)
                 GROUP BY c.id
                 ORDER BY c.opened_at DESC, c.id DESC
                 LIMIT ?4",
            )?;
            let rows = statement.query_map(
                params![organization_id, session_id, status, limit],
                |row| {
                    Ok(CashSessionRecord {
                        id: row.get(0)?,
                        status: row.get(1)?,
                        currency: row.get(2)?,
                        opening_amount_minor: row.get(3)?,
                        expected_cash_minor: row.get(4)?,
                        cash_sales_minor: row.get(5)?,
                        sales_count: row.get(6)?,
                        cash_income_minor: row.get(7)?,
                        income_minor: row.get(8)?,
                        cash_expense_minor: row.get(9)?,
                        expense_minor: row.get(10)?,
                        cash_refunds_minor: row.get(11)?,
                        refunded_minor: row.get(12)?,
                        refunds_count: row.get(13)?,
                        movements_count: row.get(14)?,
                        counted_amount_minor: row.get(15)?,
                        difference_minor: row.get(16)?,
                        difference_reason: row.get(17)?,
                        opened_by: row.get(18)?,
                        opened_by_name: row.get(19)?,
                        opened_at: row.get(20)?,
                        closed_by: row.get(21)?,
                        closed_by_name: row.get(22)?,
                        closed_at: row.get(23)?,
                    })
                },
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    pub fn record_cash_movement(
        &self,
        organization_id: &str,
        authorized_by: &str,
        input: NewCashMovement,
    ) -> Result<CashMovementRecord, DataError> {
        if !matches!(input.movement_type.as_str(), "income" | "expense") {
            return Err(DataError::InvalidValue(
                "the cash movement type must be income or expense".into(),
            ));
        }
        if !matches!(input.payment_method.as_str(), "cash" | "transfer" | "point") {
            return Err(DataError::InvalidValue(
                "the cash movement payment method is invalid".into(),
            ));
        }
        if !(1..=MAX_AMOUNT_MINOR).contains(&input.amount_minor) {
            return Err(DataError::InvalidValue(
                "the cash movement amount must be between 1 and the maximum amount".into(),
            ));
        }
        let category = input.category.trim();
        if category.is_empty() || category.chars().count() > 80 {
            return Err(DataError::InvalidValue(
                "the cash movement category must have between 1 and 80 characters".into(),
            ));
        }
        let evidence = input.evidence.trim();
        if evidence.is_empty() || evidence.chars().count() > 200 {
            return Err(DataError::InvalidValue(
                "the cash movement evidence must have between 1 and 200 characters".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let status = transaction
                .query_row(
                    "SELECT status FROM cash_sessions
                     WHERE id = ?1 AND organization_id = ?2",
                    params![input.cash_session_id, organization_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(status) = status else {
                return Err(DataError::CashSessionNotFound);
            };
            if status != "open" {
                return Err(DataError::CashSessionNotOpen);
            }
            transaction.execute(
                "INSERT INTO cash_movements
                     (id, organization_id, cash_session_id, movement_type, category,
                      amount_minor, payment_method, evidence, authorized_by, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    organization_id,
                    input.cash_session_id,
                    input.movement_type,
                    category,
                    input.amount_minor,
                    input.payment_method,
                    evidence,
                    authorized_by,
                    created_at
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "cash.movement_recorded",
                "cash_movement",
                &id,
                &format!("cash-movement:{id}:created"),
                &serde_json::json!({
                    "movementId": id,
                    "cashSessionId": input.cash_session_id,
                    "movementType": input.movement_type,
                    "category": category,
                    "amountMinor": input.amount_minor,
                    "paymentMethod": input.payment_method,
                    "evidence": evidence,
                    "authorizedBy": authorized_by,
                    "createdAt": created_at,
                }),
                &created_at,
            )?;
            transaction.commit()?;
        }

        self.load_cash_movements(organization_id, Some(&input.cash_session_id), Some(&id), 1)?
            .into_iter()
            .next()
            .ok_or(DataError::CashMovementNotFound)
    }

    pub fn list_cash_movements(
        &self,
        organization_id: &str,
        session_id: &str,
        limit: i64,
    ) -> Result<Vec<CashMovementRecord>, DataError> {
        self.load_cash_movements(organization_id, Some(session_id), None, limit.clamp(1, 100))
    }

    fn load_cash_movements(
        &self,
        organization_id: &str,
        session_id: Option<&str>,
        movement_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CashMovementRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT m.id, m.cash_session_id, m.movement_type, m.category,
                        m.amount_minor, m.payment_method, m.evidence,
                        m.authorized_by, u.name, m.created_at
                 FROM cash_movements m
                 JOIN cash_sessions c ON c.id = m.cash_session_id
                 LEFT JOIN users u ON u.id = m.authorized_by
                 WHERE m.organization_id = ?1
                   AND c.organization_id = ?1
                   AND (?2 IS NULL OR m.cash_session_id = ?2)
                   AND (?3 IS NULL OR m.id = ?3)
                 ORDER BY m.created_at DESC, m.id DESC
                 LIMIT ?4",
            )?;
            let rows = statement.query_map(
                params![organization_id, session_id, movement_id, limit],
                |row| {
                    Ok(CashMovementRecord {
                        id: row.get(0)?,
                        cash_session_id: row.get(1)?,
                        movement_type: row.get(2)?,
                        category: row.get(3)?,
                        amount_minor: row.get(4)?,
                        payment_method: row.get(5)?,
                        evidence: row.get(6)?,
                        authorized_by: row.get(7)?,
                        authorized_by_name: row.get(8)?,
                        created_at: row.get(9)?,
                    })
                },
            )?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    pub fn open_cash_session(
        &self,
        organization_id: &str,
        opened_by: &str,
        opening_amount_minor: i64,
    ) -> Result<CashSessionRecord, DataError> {
        if !(0..=MAX_AMOUNT_MINOR).contains(&opening_amount_minor) {
            return Err(DataError::InvalidValue(
                "opening cash must be between 0 and the maximum amount".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let opened_at = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let already_open: bool = transaction.query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM cash_sessions
                     WHERE organization_id = ?1 AND status = 'open'
                 )",
                params![organization_id],
                |row| row.get(0),
            )?;
            if already_open {
                return Err(DataError::CashSessionAlreadyOpen);
            }
            let currency = organization_currency(&transaction, organization_id)?;
            transaction.execute(
                "INSERT INTO cash_sessions
                     (id, organization_id, status, currency, opening_amount_minor,
                      opened_by, opened_at)
                 VALUES (?1, ?2, 'open', ?3, ?4, ?5, ?6)",
                params![
                    id,
                    organization_id,
                    currency,
                    opening_amount_minor,
                    opened_by,
                    opened_at
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "cash.session_opened",
                "cash_session",
                &id,
                &format!("cash-session:{id}:opened"),
                &serde_json::json!({
                    "cashSessionId": id,
                    "currency": currency,
                    "openingAmountMinor": opening_amount_minor,
                    "openedBy": opened_by,
                    "openedAt": opened_at,
                }),
                &opened_at,
            )?;
            transaction.commit()?;
        }

        self.cash_session(organization_id, &id)?
            .ok_or(DataError::CashSessionNotFound)
    }

    pub fn close_cash_session(
        &self,
        organization_id: &str,
        session_id: &str,
        closed_by: &str,
        counted_amount_minor: i64,
        difference_reason: Option<&str>,
    ) -> Result<CashSessionRecord, DataError> {
        if !(0..=MAX_AMOUNT_MINOR).contains(&counted_amount_minor) {
            return Err(DataError::InvalidValue(
                "counted cash must be between 0 and the maximum amount".into(),
            ));
        }
        let reason = difference_reason
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if reason.is_some_and(|value| value.chars().count() > 300) {
            return Err(DataError::InvalidValue(
                "the cash difference reason must have at most 300 characters".into(),
            ));
        }

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let session = transaction
                .query_row(
                    "SELECT status, opening_amount_minor FROM cash_sessions
                     WHERE id = ?1 AND organization_id = ?2",
                    params![session_id, organization_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
                )
                .optional()?;
            let Some((status, opening_amount_minor)) = session else {
                return Err(DataError::CashSessionNotFound);
            };
            if status != "open" {
                return Err(DataError::CashSessionNotOpen);
            }
            let cash_sales_minor: i64 = transaction.query_row(
                "SELECT COALESCE(SUM(total_minor), 0) FROM sales
                 WHERE organization_id = ?1 AND cash_session_id = ?2
                   AND payment_method = 'cash'",
                params![organization_id, session_id],
                |row| row.get(0),
            )?;
            let cash_income_minor: i64 = transaction.query_row(
                "SELECT COALESCE(SUM(amount_minor), 0) FROM cash_movements
                 WHERE organization_id = ?1 AND cash_session_id = ?2
                   AND movement_type = 'income' AND payment_method = 'cash'",
                params![organization_id, session_id],
                |row| row.get(0),
            )?;
            let cash_expense_minor: i64 = transaction.query_row(
                "SELECT COALESCE(SUM(amount_minor), 0) FROM cash_movements
                 WHERE organization_id = ?1 AND cash_session_id = ?2
                   AND movement_type = 'expense' AND payment_method = 'cash'",
                params![organization_id, session_id],
                |row| row.get(0),
            )?;
            let cash_refunds_minor: i64 = transaction.query_row(
                "SELECT COALESCE(SUM(total_minor), 0) FROM sale_returns
                 WHERE organization_id = ?1 AND cash_session_id = ?2
                   AND refund_method = 'cash'",
                params![organization_id, session_id],
                |row| row.get(0),
            )?;
            let expected_cash_minor = opening_amount_minor
                .checked_add(cash_sales_minor)
                .and_then(|value| value.checked_add(cash_income_minor))
                .and_then(|value| value.checked_sub(cash_expense_minor))
                .and_then(|value| value.checked_sub(cash_refunds_minor))
                .ok_or_else(|| DataError::InvalidValue("the expected cash is too large".into()))?;
            let difference_minor = counted_amount_minor
                .checked_sub(expected_cash_minor)
                .ok_or_else(|| {
                    DataError::InvalidValue("the cash difference is too large".into())
                })?;
            if difference_minor != 0 && reason.is_none() {
                return Err(DataError::InvalidValue(
                    "a cash difference requires an auditable reason".into(),
                ));
            }
            let stored_reason = if difference_minor == 0 { None } else { reason };
            let closed_at = chrono::Utc::now().to_rfc3339();
            let updated = transaction.execute(
                "UPDATE cash_sessions
                 SET status = 'closed', counted_amount_minor = ?1,
                     difference_minor = ?2, difference_reason = ?3,
                     closed_by = ?4, closed_at = ?5
                 WHERE id = ?6 AND organization_id = ?7 AND status = 'open'",
                params![
                    counted_amount_minor,
                    difference_minor,
                    stored_reason,
                    closed_by,
                    closed_at,
                    session_id,
                    organization_id
                ],
            )?;
            if updated != 1 {
                return Err(DataError::CashSessionNotOpen);
            }
            enqueue_sync_event(
                &transaction,
                organization_id,
                "cash.session_closed",
                "cash_session",
                session_id,
                &format!("cash-session:{session_id}:closed"),
                &serde_json::json!({
                    "cashSessionId": session_id,
                    "expectedCashMinor": expected_cash_minor,
                    "countedAmountMinor": counted_amount_minor,
                    "differenceMinor": difference_minor,
                    "differenceReason": stored_reason,
                    "closedBy": closed_by,
                    "closedAt": closed_at,
                }),
                &closed_at,
            )?;
            transaction.commit()?;
        }

        self.cash_session(organization_id, session_id)?
            .ok_or(DataError::CashSessionNotFound)
    }

    fn load_sales(
        &self,
        organization_id: &str,
        sale_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SaleRecord>, DataError> {
        self.with_connection(|conn| {
            let mut sale_statement = conn.prepare(
                "SELECT s.id, s.currency, s.subtotal_minor, s.tax_minor, s.total_minor,
                        COALESCE((
                            SELECT SUM(r.total_minor) FROM sale_returns r
                            WHERE r.sale_id = s.id
                        ), 0) AS refunded_minor,
                        s.payment_method, s.cash_received_minor, s.change_minor,
                        s.sold_by, u.name, s.cash_session_id,
                        (
                            SELECT pc.external_reference FROM payment_charges pc
                            WHERE pc.sale_id = s.id AND pc.kind = 'charge'
                            LIMIT 1
                        ), s.created_at
                 FROM sales s
                 LEFT JOIN users u ON u.id = s.sold_by
                 WHERE s.organization_id = ?1
                   AND (?2 IS NULL OR s.id = ?2)
                 ORDER BY s.created_at DESC, s.id DESC
                 LIMIT ?3",
            )?;
            let sales = sale_statement
                .query_map(params![organization_id, sale_id, limit], |row| {
                    Ok(SaleRecord {
                        id: row.get(0)?,
                        currency: row.get(1)?,
                        subtotal_minor: row.get(2)?,
                        tax_minor: row.get(3)?,
                        total_minor: row.get(4)?,
                        refunded_minor: row.get(5)?,
                        payment_method: row.get(6)?,
                        cash_received_minor: row.get(7)?,
                        change_minor: row.get(8)?,
                        sold_by: row.get(9)?,
                        sold_by_name: row.get(10)?,
                        cash_session_id: row.get(11)?,
                        payment_reference: row.get(12)?,
                        created_at: row.get(13)?,
                        items: Vec::new(),
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut item_statement = conn.prepare(
                "SELECT si.sale_id, si.id, si.product_name, si.variant_name, si.sku,
                        si.unit, si.quantity, si.unit_amount_minor, si.tax_rate_bps,
                        si.price_includes_tax, si.subtotal_minor, si.tax_minor,
                        si.total_minor, si.price_version_id, si.variant_id,
                        COALESCE((
                            SELECT SUM(ri.quantity) FROM sale_return_items ri
                            WHERE ri.sale_item_id = si.id
                        ), 0) AS refunded_quantity,
                        COALESCE((
                            SELECT SUM(ri.amount_minor) FROM sale_return_items ri
                            WHERE ri.sale_item_id = si.id
                        ), 0) AS refunded_minor
                 FROM sale_items si
                 WHERE si.organization_id = ?1
                   AND (
                       (?2 IS NOT NULL AND si.sale_id = ?2)
                       OR
                       (?2 IS NULL AND si.sale_id IN (
                           SELECT id FROM sales
                           WHERE organization_id = ?1
                           ORDER BY created_at DESC, id DESC
                           LIMIT ?3
                       ))
                   )
                 ORDER BY si.sale_id, si.position",
            )?;
            let items = item_statement
                .query_map(params![organization_id, sale_id, limit], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        SaleItemRecord {
                            id: row.get(1)?,
                            product_name: row.get(2)?,
                            variant_name: row.get(3)?,
                            sku: row.get(4)?,
                            unit: row.get(5)?,
                            quantity: row.get(6)?,
                            unit_amount_minor: row.get(7)?,
                            tax_rate_bps: row.get(8)?,
                            price_includes_tax: row.get::<_, i64>(9)? != 0,
                            subtotal_minor: row.get(10)?,
                            tax_minor: row.get(11)?,
                            total_minor: row.get(12)?,
                            price_version_id: row.get(13)?,
                            variant_id: row.get(14)?,
                            refunded_quantity: row.get(15)?,
                            refunded_minor: row.get(16)?,
                        },
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut sales = sales;
            let positions: HashMap<String, usize> = sales
                .iter()
                .enumerate()
                .map(|(index, sale)| (sale.id.clone(), index))
                .collect();
            for (sale_id, item) in items {
                if let Some(index) = positions.get(&sale_id) {
                    sales[*index].items.push(item);
                }
            }
            Ok(sales)
        })
    }

    pub fn create_sale(
        &self,
        organization_id: &str,
        sold_by: &str,
        sale: NewSale,
    ) -> Result<SaleRecord, DataError> {
        if sale.items.is_empty() {
            return Err(DataError::InvalidValue(
                "a sale needs at least one item".into(),
            ));
        }
        if sale.items.len() > 50 {
            return Err(DataError::InvalidValue(
                "a sale accepts at most 50 items".into(),
            ));
        }
        if !matches!(sale.payment_method.as_str(), "cash" | "transfer" | "point") {
            return Err(DataError::InvalidValue("invalid payment method".into()));
        }
        let payment_reference =
            normalize_optional_text(sale.payment_reference.clone(), 120, "payment reference")?;

        let idempotency_key = sale.idempotency_key.trim();
        if idempotency_key.is_empty() || idempotency_key.len() > 128 {
            return Err(DataError::InvalidValue(
                "idempotency key must have between 1 and 128 characters".into(),
            ));
        }
        let request_hash = sale.request_hash();

        let replay_sale_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let existing = conn
                .query_row(
                    "SELECT id, request_hash FROM sales
                     WHERE organization_id = ?1 AND idempotency_key = ?2",
                    params![organization_id, idempotency_key],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;
            match existing {
                Some((sale_id, stored_hash)) => {
                    if stored_hash != request_hash {
                        return Err(DataError::SaleIdempotencyConflict);
                    }
                    Some(sale_id)
                }
                None => None,
            }
        };

        if let Some(sale_id) = replay_sale_id {
            return self.sale(organization_id, &sale_id);
        }

        let mut seen = std::collections::HashSet::new();
        for item in &sale.items {
            if !(1..=999).contains(&item.quantity) {
                return Err(DataError::InvalidValue(
                    "quantity must be between 1 and 999".into(),
                ));
            }
            if !seen.insert(item.variant_id.as_str()) {
                return Err(DataError::InvalidValue(
                    "the same variant appears more than once".into(),
                ));
            }
        }

        let sale_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let currency = organization_currency(&transaction, organization_id)?;
            let cash_session_id: String = transaction
                .query_row(
                    "SELECT id FROM cash_sessions
                     WHERE organization_id = ?1 AND status = 'open'",
                    params![organization_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(DataError::CashSessionRequired)?;
            let effective_at =
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

            let mut lines = Vec::with_capacity(sale.items.len());
            for (position, item) in sale.items.iter().enumerate() {
                let row = transaction
                    .query_row(
                        "SELECT v.id, p.id, p.name, v.name, v.sku, v.unit,
                                v.status, p.status, pr.id, pr.amount_minor,
                                pr.tax_rate_bps, pr.price_includes_tax
                         FROM catalog_variants v
                         JOIN catalog_products p ON p.id = v.product_id
                         LEFT JOIN catalog_price_versions pr
                           ON pr.id = (
                               SELECT id FROM catalog_price_versions
                               WHERE variant_id = v.id AND effective_from <= ?3
                               ORDER BY effective_from DESC, created_at DESC
                               LIMIT 1
                           )
                         WHERE v.id = ?1 AND v.organization_id = ?2",
                        params![item.variant_id, organization_id, effective_at],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                SaleVariantRow {
                                    product_id: row.get(1)?,
                                    product_name: row.get(2)?,
                                    variant_name: row.get(3)?,
                                    sku: row.get(4)?,
                                    unit: row.get(5)?,
                                    variant_status: row.get(6)?,
                                    product_status: row.get(7)?,
                                    price_version_id: row.get(8)?,
                                    amount_minor: row.get(9)?,
                                    tax_rate_bps: row.get(10)?,
                                    price_includes_tax: row.get(11)?,
                                },
                            ))
                        },
                    )
                    .optional()?;
                let Some((variant_id, row)) = row else {
                    return Err(DataError::CatalogVariantNotFound);
                };
                if row.variant_status != "active" || row.product_status != "active" {
                    return Err(DataError::InvalidValue(
                        "archived products or variants cannot be sold".into(),
                    ));
                }
                let (
                    Some(price_version_id),
                    Some(amount_minor),
                    Some(tax_rate_bps),
                    Some(includes),
                ) = (
                    row.price_version_id,
                    row.amount_minor,
                    row.tax_rate_bps,
                    row.price_includes_tax,
                )
                else {
                    return Err(DataError::InvalidValue(
                        "a variant has no effective price".into(),
                    ));
                };
                if amount_minor < 0 || !(0..=10_000).contains(&tax_rate_bps) {
                    return Err(DataError::InvalidValue("invalid catalog price".into()));
                }
                let price_includes_tax = includes != 0;
                let (subtotal_minor, tax_minor, total_minor) = line_totals(
                    amount_minor,
                    item.quantity,
                    tax_rate_bps,
                    price_includes_tax,
                )?;
                lines.push(SaleLine {
                    position,
                    product_id: row.product_id,
                    variant_id,
                    price_version_id,
                    product_name: row.product_name,
                    variant_name: row.variant_name,
                    sku: row.sku,
                    unit: row.unit,
                    quantity: item.quantity,
                    unit_amount_minor: amount_minor,
                    tax_rate_bps,
                    price_includes_tax,
                    subtotal_minor,
                    tax_minor,
                    total_minor,
                });
            }

            let subtotal_minor = lines.iter().try_fold(0_i64, |total, line| {
                total.checked_add(line.subtotal_minor).ok_or_else(overflow)
            })?;
            let tax_minor = lines.iter().try_fold(0_i64, |total, line| {
                total.checked_add(line.tax_minor).ok_or_else(overflow)
            })?;
            let total_minor = lines.iter().try_fold(0_i64, |total, line| {
                total.checked_add(line.total_minor).ok_or_else(overflow)
            })?;
            if subtotal_minor > MAX_AMOUNT_MINOR || total_minor > MAX_AMOUNT_MINOR {
                return Err(DataError::InvalidValue(
                    "the sale total is too large".into(),
                ));
            }

            let (cash_received_minor, change_minor) = match sale.payment_method.as_str() {
                "cash" => {
                    let received = sale.cash_received_minor.unwrap_or(total_minor);
                    if received < total_minor {
                        return Err(DataError::InvalidValue(
                            "cash received is less than the total".into(),
                        ));
                    }
                    if received > MAX_AMOUNT_MINOR {
                        return Err(DataError::InvalidValue("cash received is too large".into()));
                    }
                    (Some(received), Some(received - total_minor))
                }
                _ => {
                    if sale.cash_received_minor.is_some() {
                        return Err(DataError::InvalidValue(
                            "cash received only applies to cash payments".into(),
                        ));
                    }
                    (None, None)
                }
            };

            transaction.execute(
                "INSERT INTO sales
                     (id, organization_id, currency, subtotal_minor, tax_minor,
                      total_minor, payment_method, cash_received_minor, change_minor,
                      sold_by, cash_session_id, created_at, idempotency_key, request_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    sale_id,
                    organization_id,
                    currency,
                    subtotal_minor,
                    tax_minor,
                    total_minor,
                    sale.payment_method,
                    cash_received_minor,
                    change_minor,
                    sold_by,
                    cash_session_id,
                    now,
                    idempotency_key,
                    request_hash
                ],
            )?;
            insert_payment_charge(
                &transaction,
                PaymentChargeInsert {
                    id: &uuid::Uuid::new_v4().to_string(),
                    organization_id,
                    sale_id: &sale_id,
                    sale_return_id: None,
                    kind: "charge",
                    method: &sale.payment_method,
                    provider: sale.provider(),
                    status: "approved",
                    amount_minor: total_minor,
                    currency: &currency,
                    external_reference: payment_reference.as_deref(),
                    idempotency_key,
                    request_hash: &request_hash,
                    created_at: &now,
                },
            )?;
            for line in &lines {
                let sale_item_id = uuid::Uuid::new_v4().to_string();
                transaction.execute(
                    "INSERT INTO sale_items
                         (id, organization_id, sale_id, position, product_id, variant_id,
                          price_version_id, product_name, variant_name, sku, unit,
                          quantity, unit_amount_minor, tax_rate_bps, price_includes_tax,
                          subtotal_minor, tax_minor, total_minor)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                             ?14, ?15, ?16, ?17, ?18)",
                    params![
                        sale_item_id,
                        organization_id,
                        sale_id,
                        line.position as i64,
                        line.product_id,
                        line.variant_id,
                        line.price_version_id,
                        line.product_name,
                        line.variant_name,
                        line.sku,
                        line.unit,
                        line.quantity,
                        line.unit_amount_minor,
                        line.tax_rate_bps,
                        i64::from(line.price_includes_tax),
                        line.subtotal_minor,
                        line.tax_minor,
                        line.total_minor
                    ],
                )?;
                if line.unit == "piece" {
                    let unit_cost_minor =
                        inventory_average_cost(&transaction, organization_id, &line.variant_id)?;
                    let total_cost_minor = -line
                        .quantity
                        .checked_mul(unit_cost_minor)
                        .ok_or_else(overflow)?;
                    insert_inventory_movement(
                        &transaction,
                        InventoryMovementInsert {
                            id: &uuid::Uuid::new_v4().to_string(),
                            organization_id,
                            variant_id: &line.variant_id,
                            movement_type: "sale",
                            quantity_delta: -line.quantity,
                            unit_cost_minor,
                            total_cost_minor,
                            source_type: "sale",
                            source_id: Some(&sale_item_id),
                            reference: Some(&sale_id),
                            note: None,
                            authorized_by: Some(sold_by),
                            created_at: &now,
                        },
                    )?;
                }
            }
            transaction.execute(
                "INSERT INTO ledger_entries
                     (id, organization_id, entry_type, reference_id, sale_id,
                      currency, amount_minor, occurred_at, created_at)
                 VALUES (?1, ?2, 'sale', ?3, ?3, ?4, ?5, ?6, ?6)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    organization_id,
                    sale_id,
                    currency,
                    total_minor,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "sale.created",
                "sale",
                &sale_id,
                &format!("sale:{sale_id}:created"),
                &serde_json::json!({
                    "saleId": sale_id,
                    "currency": currency,
                    "subtotalMinor": subtotal_minor,
                    "taxMinor": tax_minor,
                    "totalMinor": total_minor,
                    "paymentMethod": sale.payment_method,
                    "cashReceivedMinor": cash_received_minor,
                    "changeMinor": change_minor,
                    "cashSessionId": cash_session_id,
                    "soldBy": sold_by,
                    "paymentReference": payment_reference,
                    "createdAt": now,
                    "items": lines.iter().map(|line| serde_json::json!({
                        "variantId": line.variant_id,
                        "productName": line.product_name,
                        "variantName": line.variant_name,
                        "sku": line.sku,
                        "unit": line.unit,
                        "quantity": line.quantity,
                        "unitAmountMinor": line.unit_amount_minor,
                        "taxRateBps": line.tax_rate_bps,
                        "priceIncludesTax": line.price_includes_tax,
                        "subtotalMinor": line.subtotal_minor,
                        "taxMinor": line.tax_minor,
                        "totalMinor": line.total_minor,
                    })).collect::<Vec<_>>(),
                }),
                &now,
            )?;
            transaction.commit()?;
        }

        self.sale(organization_id, &sale_id)
    }

    pub fn sale_return(
        &self,
        organization_id: &str,
        return_id: &str,
    ) -> Result<SaleReturnRecord, DataError> {
        self.load_sale_returns(organization_id, Some(return_id), None, 1)?
            .into_iter()
            .next()
            .ok_or(DataError::SaleReturnNotFound)
    }

    pub fn list_sale_returns(
        &self,
        organization_id: &str,
        sale_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SaleReturnRecord>, DataError> {
        self.load_sale_returns(organization_id, None, sale_id, limit.clamp(1, 100))
    }

    pub fn financial_report(
        &self,
        organization_id: &str,
        from: &str,
        to: &str,
    ) -> Result<FinancialReportRecord, DataError> {
        let (from, to) = normalized_report_range(from, to)?;
        self.with_connection(|conn| {
            let currency = conn.query_row(
                "SELECT currency FROM organizations WHERE id = ?1",
                params![organization_id],
                |row| row.get::<_, String>(0),
            )?;
            let (sales_count, gross_sales_minor, tax_minor) = conn.query_row(
                "SELECT COUNT(*), COALESCE(SUM(total_minor), 0), COALESCE(SUM(tax_minor), 0)
                 FROM sales
                 WHERE organization_id = ?1
                   AND julianday(created_at) >= julianday(?2)
                   AND julianday(created_at) < julianday(?3)",
                params![organization_id, from, to],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
            let (refunds_count, refunded_minor) = conn.query_row(
                "SELECT COUNT(*), COALESCE(SUM(total_minor), 0)
                 FROM sale_returns
                 WHERE organization_id = ?1
                   AND julianday(created_at) >= julianday(?2)
                   AND julianday(created_at) < julianday(?3)",
                params![organization_id, from, to],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;

            let mut payments: HashMap<String, PaymentMethodTotalRecord> = HashMap::new();
            {
                let mut statement = conn.prepare(
                    "SELECT payment_method, COUNT(*), COALESCE(SUM(total_minor), 0)
                     FROM sales
                     WHERE organization_id = ?1
                       AND julianday(created_at) >= julianday(?2)
                       AND julianday(created_at) < julianday(?3)
                     GROUP BY payment_method",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })?;
                for row in rows {
                    let (method, count, total) = row?;
                    payments.insert(
                        method.clone(),
                        PaymentMethodTotalRecord {
                            payment_method: method,
                            sales_count: count,
                            gross_sales_minor: total,
                            refunds_count: 0,
                            refunded_minor: 0,
                            net_sales_minor: total,
                        },
                    );
                }
            }
            {
                let mut statement = conn.prepare(
                    "SELECT refund_method, COUNT(*), COALESCE(SUM(total_minor), 0)
                     FROM sale_returns
                     WHERE organization_id = ?1
                       AND julianday(created_at) >= julianday(?2)
                       AND julianday(created_at) < julianday(?3)
                     GROUP BY refund_method",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })?;
                for row in rows {
                    let (method, count, total) = row?;
                    let entry =
                        payments
                            .entry(method.clone())
                            .or_insert(PaymentMethodTotalRecord {
                                payment_method: method,
                                sales_count: 0,
                                gross_sales_minor: 0,
                                refunds_count: 0,
                                refunded_minor: 0,
                                net_sales_minor: 0,
                            });
                    entry.refunds_count = count;
                    entry.refunded_minor = total;
                }
            }
            let payment_methods = ["cash", "transfer", "point"]
                .into_iter()
                .map(|method| {
                    let mut total = payments.remove(method).unwrap_or(PaymentMethodTotalRecord {
                        payment_method: method.to_string(),
                        sales_count: 0,
                        gross_sales_minor: 0,
                        refunds_count: 0,
                        refunded_minor: 0,
                        net_sales_minor: 0,
                    });
                    total.net_sales_minor = total.gross_sales_minor - total.refunded_minor;
                    total
                })
                .collect::<Vec<_>>();

            let mut products: HashMap<String, ProductAccumulator> = HashMap::new();
            {
                let mut statement = conn.prepare(
                    "SELECT si.variant_id, MAX(si.product_name), MAX(si.variant_name),
                            MAX(si.sku), MAX(si.unit), SUM(si.quantity), SUM(si.total_minor)
                     FROM sale_items si
                     JOIN sales s ON s.id = si.sale_id
                     WHERE s.organization_id = ?1
                       AND julianday(s.created_at) >= julianday(?2)
                       AND julianday(s.created_at) < julianday(?3)
                     GROUP BY si.variant_id",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                })?;
                for row in rows {
                    let (variant_id, product_name, variant_name, sku, unit, units, total) = row?;
                    products.insert(
                        variant_id.clone(),
                        ProductAccumulator {
                            variant_id,
                            product_name,
                            variant_name,
                            sku,
                            unit,
                            sold_units: units,
                            refunded_units: 0,
                            gross_sales_minor: total,
                            refunded_minor: 0,
                            cost_minor: 0,
                        },
                    );
                }
            }
            {
                let mut statement = conn.prepare(
                    "SELECT si.variant_id, MAX(si.product_name), MAX(si.variant_name),
                            MAX(si.sku), MAX(si.unit), SUM(ri.quantity),
                            COALESCE(SUM(ri.amount_minor), 0)
                     FROM sale_return_items ri
                     JOIN sale_returns r ON r.id = ri.return_id
                     JOIN sale_items si ON si.id = ri.sale_item_id
                     WHERE r.organization_id = ?1
                       AND julianday(r.created_at) >= julianday(?2)
                       AND julianday(r.created_at) < julianday(?3)
                     GROUP BY si.variant_id",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                })?;
                for row in rows {
                    let (variant_id, product_name, variant_name, sku, unit, units, total) = row?;
                    let entry = products
                        .entry(variant_id.clone())
                        .or_insert(ProductAccumulator {
                            variant_id,
                            product_name,
                            variant_name,
                            sku,
                            unit,
                            sold_units: 0,
                            refunded_units: 0,
                            gross_sales_minor: 0,
                            refunded_minor: 0,
                            cost_minor: 0,
                        });
                    entry.refunded_units = units;
                    entry.refunded_minor = total;
                }
            }
            {
                let mut statement = conn.prepare(
                    "SELECT variant_id, COALESCE(SUM(total_cost_minor), 0)
                     FROM inventory_movements
                     WHERE organization_id = ?1
                       AND movement_type IN ('sale', 'return')
                       AND julianday(created_at) >= julianday(?2)
                       AND julianday(created_at) < julianday(?3)
                     GROUP BY variant_id",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?;
                for row in rows {
                    let (variant_id, total_cost) = row?;
                    if let Some(entry) = products.get_mut(&variant_id) {
                        entry.cost_minor = -total_cost;
                    }
                }
            }
            let mut product_sales = products
                .into_values()
                .map(|entry| ProductSalesRecord {
                    variant_id: entry.variant_id,
                    product_name: entry.product_name,
                    variant_name: entry.variant_name,
                    sku: entry.sku,
                    unit: entry.unit,
                    sold_units: entry.sold_units,
                    refunded_units: entry.refunded_units,
                    net_units: entry.sold_units - entry.refunded_units,
                    gross_sales_minor: entry.gross_sales_minor,
                    refunded_minor: entry.refunded_minor,
                    net_sales_minor: entry.gross_sales_minor - entry.refunded_minor,
                    cost_minor: entry.cost_minor,
                    margin_minor: entry.gross_sales_minor - entry.refunded_minor - entry.cost_minor,
                })
                .collect::<Vec<_>>();
            product_sales.sort_by(|left, right| {
                right
                    .net_sales_minor
                    .cmp(&left.net_sales_minor)
                    .then_with(|| right.net_units.cmp(&left.net_units))
                    .then_with(|| left.product_name.cmp(&right.product_name))
                    .then_with(|| left.variant_name.cmp(&right.variant_name))
            });

            let mut statement = conn.prepare(
                "SELECT m.variant_id, p.name, v.name, v.sku,
                        SUM(m.quantity_delta), COALESCE(SUM(m.total_cost_minor), 0)
                 FROM inventory_movements m
                 JOIN catalog_variants v ON v.id = m.variant_id
                 JOIN catalog_products p ON p.id = v.product_id
                 WHERE m.organization_id = ?1
                   AND m.movement_type = 'adjustment'
                   AND julianday(m.created_at) >= julianday(?2)
                   AND julianday(m.created_at) < julianday(?3)
                 GROUP BY m.variant_id
                 ORDER BY p.name COLLATE NOCASE, v.name COLLATE NOCASE",
            )?;
            let inventory_adjustments = statement
                .query_map(params![organization_id, from, to], |row| {
                    Ok(InventoryAdjustmentRecord {
                        variant_id: row.get(0)?,
                        product_name: row.get(1)?,
                        variant_name: row.get(2)?,
                        sku: row.get(3)?,
                        units_delta: row.get(4)?,
                        cost_minor: row.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let (multi_item_sales, sold_units): (i64, i64) = conn.query_row(
                "SELECT COALESCE(SUM(CASE WHEN item_count >= 2 THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(units), 0)
                 FROM (
                     SELECT si.sale_id, COUNT(*) AS item_count, SUM(si.quantity) AS units
                     FROM sale_items si
                     JOIN sales s ON s.id = si.sale_id
                     WHERE s.organization_id = ?1
                       AND julianday(s.created_at) >= julianday(?2)
                       AND julianday(s.created_at) < julianday(?3)
                     GROUP BY si.sale_id
                 )",
                params![organization_id, from, to],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let inventory_value_before = |boundary: &str| -> Result<i64, DataError> {
                conn.query_row(
                    "SELECT COALESCE(SUM(total_cost_minor), 0)
                     FROM inventory_movements
                     WHERE organization_id = ?1
                       AND julianday(created_at) < julianday(?2)",
                    params![organization_id, boundary],
                    |row| row.get(0),
                )
                .map_err(DataError::from)
            };
            let inventory_value_start_minor = inventory_value_before(&from)?;
            let inventory_value_end_minor = inventory_value_before(&to)?;
            let inventory_value_average_minor =
                (inventory_value_start_minor + inventory_value_end_minor) / 2;

            let adjustments_units: i64 = inventory_adjustments
                .iter()
                .map(|adjustment| adjustment.units_delta)
                .sum();
            let adjustments_cost_minor: i64 = inventory_adjustments
                .iter()
                .map(|adjustment| adjustment.cost_minor)
                .sum();
            let cost_of_goods_minor: i64 =
                product_sales.iter().map(|product| product.cost_minor).sum();
            let net_sales_minor = gross_sales_minor - refunded_minor;
            let margin_minor = net_sales_minor - cost_of_goods_minor;
            let margin_bps = if net_sales_minor == 0 {
                0
            } else {
                margin_minor * 10_000 / net_sales_minor
            };
            let attach_rate_bps = if sales_count > 0 {
                (multi_item_sales * 10_000 + sales_count / 2) / sales_count
            } else {
                0
            };
            let items_per_sale_milli = if sales_count > 0 {
                (sold_units * 1_000 + sales_count / 2) / sales_count
            } else {
                0
            };
            let inventory_turnover_bps = if inventory_value_average_minor > 0 {
                cost_of_goods_minor * 10_000 / inventory_value_average_minor
            } else {
                0
            };

            let (
                cash_sessions_count,
                closed_cash_sessions,
                cash_difference_minor,
                cash_difference_abs_minor,
            ) = conn.query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN status = 'closed' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(difference_minor), 0),
                        COALESCE(SUM(ABS(difference_minor)), 0)
                 FROM cash_sessions
                 WHERE organization_id = ?1
                   AND julianday(opened_at) >= julianday(?2)
                   AND julianday(opened_at) < julianday(?3)",
                params![organization_id, from, to],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;

            Ok(FinancialReportRecord {
                from,
                to,
                generated_at: chrono::Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                currency,
                sales_count,
                gross_sales_minor,
                tax_minor,
                refunds_count,
                refunded_minor,
                net_sales_minor,
                average_sale_minor: if sales_count > 0 {
                    (gross_sales_minor + sales_count / 2) / sales_count
                } else {
                    0
                },
                attach_rate_bps,
                items_per_sale_milli,
                cost_of_goods_minor,
                margin_minor,
                margin_bps,
                inventory_value_start_minor,
                inventory_value_end_minor,
                inventory_turnover_bps,
                adjustments_units,
                adjustments_cost_minor,
                payment_methods,
                cash_sessions_count,
                open_cash_sessions: cash_sessions_count - closed_cash_sessions,
                closed_cash_sessions,
                cash_difference_minor,
                cash_difference_abs_minor,
                product_sales,
                inventory_adjustments,
            })
        })
    }

    pub fn list_payment_charges(
        &self,
        organization_id: &str,
        sale_id: Option<&str>,
        external_reference: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PaymentChargeRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, sale_id, sale_return_id, kind, method, provider, status,
                        amount_minor, currency, external_reference, created_at
                 FROM payment_charges
                 WHERE organization_id = ?1
                   AND (?2 IS NULL OR sale_id = ?2)
                   AND (?3 IS NULL OR external_reference = ?3)
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?4",
            )?;
            let rows = statement
                .query_map(
                    params![
                        organization_id,
                        sale_id,
                        external_reference,
                        limit.clamp(1, 100)
                    ],
                    |row| {
                        Ok(PaymentChargeRecord {
                            id: row.get(0)?,
                            sale_id: row.get(1)?,
                            sale_return_id: row.get(2)?,
                            kind: row.get(3)?,
                            method: row.get(4)?,
                            provider: row.get(5)?,
                            status: row.get(6)?,
                            amount_minor: row.get(7)?,
                            currency: row.get(8)?,
                            external_reference: row.get(9)?,
                            created_at: row.get(10)?,
                        })
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn load_sale_returns(
        &self,
        organization_id: &str,
        return_id: Option<&str>,
        sale_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SaleReturnRecord>, DataError> {
        self.with_connection(|conn| {
            let mut return_statement = conn.prepare(
                "SELECT r.id, r.sale_id, r.cash_session_id, r.currency,
                        r.refund_method, r.total_minor, r.reason, r.authorized_by,
                        u.name,
                        (
                            SELECT pc.external_reference FROM payment_charges pc
                            WHERE pc.sale_return_id = r.id AND pc.kind = 'refund'
                            LIMIT 1
                        ), r.created_at
                 FROM sale_returns r
                 LEFT JOIN users u ON u.id = r.authorized_by
                 WHERE r.organization_id = ?1
                   AND (?2 IS NULL OR r.id = ?2)
                   AND (?3 IS NULL OR r.sale_id = ?3)
                 ORDER BY r.created_at DESC, r.id DESC
                 LIMIT ?4",
            )?;
            let returns = return_statement
                .query_map(params![organization_id, return_id, sale_id, limit], |row| {
                    Ok(SaleReturnRecord {
                        id: row.get(0)?,
                        sale_id: row.get(1)?,
                        cash_session_id: row.get(2)?,
                        currency: row.get(3)?,
                        refund_method: row.get(4)?,
                        total_minor: row.get(5)?,
                        reason: row.get(6)?,
                        authorized_by: row.get(7)?,
                        authorized_by_name: row.get(8)?,
                        payment_reference: row.get(9)?,
                        created_at: row.get(10)?,
                        items: Vec::new(),
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut item_statement = conn.prepare(
                "SELECT ri.return_id, ri.id, ri.sale_item_id, si.product_name,
                        si.variant_name, si.sku, ri.quantity, ri.amount_minor
                 FROM sale_return_items ri
                 JOIN sale_items si ON si.id = ri.sale_item_id
                 WHERE ri.organization_id = ?1
                   AND (
                       (?2 IS NOT NULL AND ri.return_id = ?2)
                       OR
                       (?2 IS NULL AND ri.return_id IN (
                           SELECT id FROM sale_returns r
                           WHERE r.organization_id = ?1
                             AND (?3 IS NULL OR r.sale_id = ?3)
                           ORDER BY r.created_at DESC, r.id DESC
                           LIMIT ?4
                       ))
                   )
                 ORDER BY ri.return_id, ri.id",
            )?;
            let items = item_statement
                .query_map(params![organization_id, return_id, sale_id, limit], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        SaleReturnItemRecord {
                            id: row.get(1)?,
                            sale_item_id: row.get(2)?,
                            product_name: row.get(3)?,
                            variant_name: row.get(4)?,
                            sku: row.get(5)?,
                            quantity: row.get(6)?,
                            amount_minor: row.get(7)?,
                        },
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut returns = returns;
            let positions: HashMap<String, usize> = returns
                .iter()
                .enumerate()
                .map(|(index, sale_return)| (sale_return.id.clone(), index))
                .collect();
            for (return_id, item) in items {
                if let Some(index) = positions.get(&return_id) {
                    returns[*index].items.push(item);
                }
            }
            Ok(returns)
        })
    }

    pub fn create_sale_return(
        &self,
        organization_id: &str,
        authorized_by: &str,
        sale_return: NewSaleReturn,
    ) -> Result<SaleReturnRecord, DataError> {
        let reason = sale_return.reason.trim();
        if reason.is_empty() || reason.chars().count() > 300 {
            return Err(DataError::InvalidValue(
                "the return reason must be between 1 and 300 characters".into(),
            ));
        }
        if sale_return.items.is_empty() {
            return Err(DataError::InvalidValue(
                "a return needs at least one item".into(),
            ));
        }
        if sale_return.items.len() > 50 {
            return Err(DataError::InvalidValue(
                "a return accepts at most 50 items".into(),
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for item in &sale_return.items {
            if !(1..=999).contains(&item.quantity) {
                return Err(DataError::InvalidValue(
                    "return quantity must be between 1 and 999".into(),
                ));
            }
            if !seen.insert(item.sale_item_id.as_str()) {
                return Err(DataError::InvalidValue(
                    "the same sale item appears more than once".into(),
                ));
            }
        }
        let payment_reference = normalize_optional_text(
            sale_return.payment_reference.clone(),
            120,
            "payment reference",
        )?;
        let idempotency_key = sale_return.idempotency_key.trim();
        if idempotency_key.is_empty() || idempotency_key.len() > 128 {
            return Err(DataError::InvalidValue(
                "idempotency key must have between 1 and 128 characters".into(),
            ));
        }
        let request_hash = sale_return.request_hash();

        let replay_return_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let existing = conn
                .query_row(
                    "SELECT id, request_hash FROM sale_returns
                     WHERE organization_id = ?1 AND idempotency_key = ?2",
                    params![organization_id, idempotency_key],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;
            match existing {
                Some((return_id, stored_hash)) => {
                    if stored_hash != request_hash {
                        return Err(DataError::SaleIdempotencyConflict);
                    }
                    Some(return_id)
                }
                None => None,
            }
        };
        if let Some(return_id) = replay_return_id {
            return self.sale_return(organization_id, &return_id);
        }

        let return_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let sale = transaction
                .query_row(
                    "SELECT id, currency, payment_method
                     FROM sales
                     WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, sale_return.sale_id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    },
                )
                .optional()?
                .ok_or(DataError::SaleNotFound)?;
            let (_, currency, refund_method) = sale;
            let cash_session_id: String = transaction
                .query_row(
                    "SELECT id FROM cash_sessions
                     WHERE organization_id = ?1 AND status = 'open'",
                    params![organization_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(DataError::CashSessionRequired)?;

            let mut total_minor = 0_i64;
            let mut lines = Vec::with_capacity(sale_return.items.len());
            for item in &sale_return.items {
                let row = transaction
                    .query_row(
                        "SELECT si.id, si.quantity, si.total_minor,
                                COALESCE((
                                    SELECT SUM(ri.quantity) FROM sale_return_items ri
                                    WHERE ri.sale_item_id = si.id
                                ), 0) AS refunded_quantity,
                                COALESCE((
                                    SELECT SUM(ri.amount_minor) FROM sale_return_items ri
                                    WHERE ri.sale_item_id = si.id
                                ), 0) AS refunded_minor,
                                si.variant_id, si.unit
                         FROM sale_items si
                         WHERE si.organization_id = ?1 AND si.sale_id = ?2
                           AND si.id = ?3",
                        params![organization_id, sale_return.sale_id, item.sale_item_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, i64>(2)?,
                                row.get::<_, i64>(3)?,
                                row.get::<_, i64>(4)?,
                                row.get::<_, String>(5)?,
                                row.get::<_, String>(6)?,
                            ))
                        },
                    )
                    .optional()?;
                let Some((
                    sale_item_id,
                    quantity,
                    item_total,
                    refunded_quantity,
                    refunded_minor,
                    variant_id,
                    unit,
                )) = row
                else {
                    return Err(DataError::InvalidValue(
                        "the sale item does not belong to this sale".into(),
                    ));
                };
                let remaining_quantity = quantity - refunded_quantity;
                if remaining_quantity <= 0 {
                    return Err(DataError::InvalidValue(
                        "the sale item was already fully returned".into(),
                    ));
                }
                if item.quantity > remaining_quantity {
                    return Err(DataError::InvalidValue(
                        "the return quantity exceeds the refundable quantity".into(),
                    ));
                }
                let remaining_amount = item_total - refunded_minor;
                let amount_minor = if item.quantity == remaining_quantity {
                    remaining_amount
                } else {
                    remaining_amount
                        .checked_mul(item.quantity)
                        .ok_or_else(overflow)?
                        / remaining_quantity
                };
                if amount_minor < 0 {
                    return Err(DataError::InvalidValue(
                        "the refund amount is invalid".into(),
                    ));
                }
                total_minor = total_minor.checked_add(amount_minor).ok_or_else(overflow)?;
                lines.push((sale_item_id, item.quantity, amount_minor, variant_id, unit));
            }
            if total_minor <= 0 {
                return Err(DataError::InvalidValue(
                    "the return total must be greater than zero".into(),
                ));
            }
            if total_minor > MAX_AMOUNT_MINOR {
                return Err(DataError::InvalidValue(
                    "the return total is too large".into(),
                ));
            }

            transaction.execute(
                "INSERT INTO sale_returns
                     (id, organization_id, sale_id, cash_session_id, currency,
                      refund_method, total_minor, reason, authorized_by, created_at,
                      idempotency_key, request_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    return_id,
                    organization_id,
                    sale_return.sale_id,
                    cash_session_id,
                    currency,
                    refund_method,
                    total_minor,
                    reason,
                    authorized_by,
                    now,
                    idempotency_key,
                    request_hash
                ],
            )?;
            let refund_provider = if refund_method == "point" {
                "point"
            } else {
                "manual"
            };
            insert_payment_charge(
                &transaction,
                PaymentChargeInsert {
                    id: &uuid::Uuid::new_v4().to_string(),
                    organization_id,
                    sale_id: &sale_return.sale_id,
                    sale_return_id: Some(&return_id),
                    kind: "refund",
                    method: &refund_method,
                    provider: refund_provider,
                    status: "approved",
                    amount_minor: total_minor,
                    currency: &currency,
                    external_reference: payment_reference.as_deref(),
                    idempotency_key,
                    request_hash: &request_hash,
                    created_at: &now,
                },
            )?;
            for (sale_item_id, quantity, amount_minor, variant_id, unit) in &lines {
                transaction.execute(
                    "INSERT INTO sale_return_items
                         (id, organization_id, return_id, sale_item_id, quantity, amount_minor)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        organization_id,
                        return_id,
                        sale_item_id,
                        quantity,
                        amount_minor
                    ],
                )?;
                if unit == "piece" {
                    let unit_cost_minor = transaction
                        .query_row(
                            "SELECT unit_cost_minor
                             FROM inventory_movements
                             WHERE organization_id = ?1 AND source_type = 'sale'
                               AND source_id = ?2
                             ORDER BY created_at, id
                             LIMIT 1",
                            params![organization_id, sale_item_id],
                            |row| row.get::<_, i64>(0),
                        )
                        .optional()?
                        .map(Ok)
                        .unwrap_or_else(|| {
                            inventory_average_cost(&transaction, organization_id, variant_id)
                        })?;
                    let total_cost_minor =
                        quantity.checked_mul(unit_cost_minor).ok_or_else(overflow)?;
                    insert_inventory_movement(
                        &transaction,
                        InventoryMovementInsert {
                            id: &uuid::Uuid::new_v4().to_string(),
                            organization_id,
                            variant_id,
                            movement_type: "return",
                            quantity_delta: *quantity,
                            unit_cost_minor,
                            total_cost_minor,
                            source_type: "sale_return",
                            source_id: Some(&return_id),
                            reference: Some(&sale_return.sale_id),
                            note: Some(reason),
                            authorized_by: Some(authorized_by),
                            created_at: &now,
                        },
                    )?;
                }
            }
            transaction.execute(
                "INSERT INTO ledger_entries
                     (id, organization_id, entry_type, reference_id, sale_id,
                      currency, amount_minor, occurred_at, created_at)
                 VALUES (?1, ?2, 'refund', ?3, ?4, ?5, ?6, ?7, ?7)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    organization_id,
                    return_id,
                    sale_return.sale_id,
                    currency,
                    -total_minor,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "sale.returned",
                "sale_return",
                &return_id,
                &format!("sale-return:{return_id}:created"),
                &serde_json::json!({
                    "returnId": return_id,
                    "saleId": sale_return.sale_id,
                    "currency": currency,
                    "refundMethod": refund_method,
                    "totalMinor": total_minor,
                    "reason": reason,
                    "authorizedBy": authorized_by,
                    "paymentReference": payment_reference,
                    "createdAt": now,
                    "items": lines.iter().map(|(sale_item_id, quantity, amount_minor, variant_id, unit)| serde_json::json!({
                        "saleItemId": sale_item_id,
                        "variantId": variant_id,
                        "unit": unit,
                        "quantity": quantity,
                        "amountMinor": amount_minor,
                    })).collect::<Vec<_>>(),
                }),
                &now,
            )?;
            transaction.commit()?;
        }

        self.sale_return(organization_id, &return_id)
    }

    pub fn list_inventory(
        &self,
        organization_id: &str,
    ) -> Result<Vec<InventoryItemRecord>, DataError> {
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let mut statement = conn.prepare(
            "SELECT v.id, p.name, v.name, v.sku, v.unit,
                    COALESCE(SUM(b.quantity), 0) AS stock,
                    COALESCE(SUM(b.value_minor), 0) AS value_minor,
                    CASE
                        WHEN COALESCE(SUM(b.quantity), 0) > 0
                            THEN CAST(ROUND(CAST(COALESCE(SUM(b.value_minor), 0) AS REAL)
                                            / SUM(b.quantity)) AS INTEGER)
                        ELSE 0
                    END AS avg_unit_cost_minor,
                    COALESCE(pol.min_stock, 0) AS min_stock
             FROM catalog_variants v
             JOIN catalog_products p ON p.id = v.product_id
             LEFT JOIN inventory_balances b
               ON b.variant_id = v.id
              AND b.organization_id = v.organization_id
             LEFT JOIN inventory_policies pol
               ON pol.variant_id = v.id AND pol.organization_id = v.organization_id
             WHERE v.organization_id = ?1 AND v.unit = 'piece'
             GROUP BY v.id, p.name, v.name, v.sku, v.unit, pol.min_stock
             ORDER BY p.name COLLATE NOCASE, v.name COLLATE NOCASE, v.id",
        )?;
        let rows = statement
            .query_map(params![organization_id], |row| {
                let stock = row.get::<_, i64>(5)?;
                let min_stock = row.get::<_, i64>(8)?;
                Ok(InventoryItemRecord {
                    variant_id: row.get(0)?,
                    product_name: row.get(1)?,
                    variant_name: row.get(2)?,
                    sku: row.get(3)?,
                    unit: row.get(4)?,
                    stock,
                    value_minor: row.get(6)?,
                    avg_unit_cost_minor: row.get(7)?,
                    min_stock,
                    low_stock: stock <= min_stock,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_inventory_movements(
        &self,
        organization_id: &str,
        variant_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<InventoryMovementRecord>, DataError> {
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let mut statement = conn.prepare(
            "SELECT m.id, m.variant_id, m.movement_type,
                    m.quantity_delta, m.unit_cost_minor, m.total_cost_minor,
                    m.source_type, m.source_id, m.reference, m.note,
                    m.authorized_by, u.name, m.created_at
             FROM inventory_movements m
             LEFT JOIN users u ON u.id = m.authorized_by
             WHERE m.organization_id = ?1
               AND (?2 IS NULL OR m.variant_id = ?2)
             ORDER BY m.created_at DESC, m.id DESC
             LIMIT ?3",
        )?;
        let rows = statement
            .query_map(
                params![organization_id, variant_id, limit.clamp(1, 200)],
                map_inventory_movement,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn inventory_movement(
        &self,
        organization_id: &str,
        movement_id: &str,
    ) -> Result<InventoryMovementRecord, DataError> {
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        conn.query_row(
            "SELECT m.id, m.variant_id, m.movement_type,
                    m.quantity_delta, m.unit_cost_minor, m.total_cost_minor,
                    m.source_type, m.source_id, m.reference, m.note,
                    m.authorized_by, u.name, m.created_at
             FROM inventory_movements m
             LEFT JOIN users u ON u.id = m.authorized_by
             WHERE m.organization_id = ?1 AND m.id = ?2",
            params![organization_id, movement_id],
            map_inventory_movement,
        )
        .optional()?
        .ok_or(DataError::InventoryMovementNotFound)
    }

    pub fn record_inventory_entry(
        &self,
        organization_id: &str,
        authorized_by: &str,
        entry: NewInventoryEntry,
    ) -> Result<InventoryMovementRecord, DataError> {
        if !(1..=999_999).contains(&entry.quantity) {
            return Err(DataError::InvalidValue(
                "inventory quantity must be between 1 and 999999".into(),
            ));
        }
        if !(0..=MAX_AMOUNT_MINOR).contains(&entry.unit_cost_minor) {
            return Err(DataError::InvalidValue(
                "invalid inventory unit cost".into(),
            ));
        }
        let reference = normalize_optional_text(entry.reference, 120, "inventory reference")?;
        let note = normalize_optional_text(entry.note, 300, "inventory note")?;
        let movement_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            ensure_inventory_variant(&transaction, organization_id, &entry.variant_id)?;
            let total_cost_minor = entry
                .quantity
                .checked_mul(entry.unit_cost_minor)
                .ok_or_else(overflow)?;
            insert_inventory_movement(
                &transaction,
                InventoryMovementInsert {
                    id: &movement_id,
                    organization_id,
                    variant_id: &entry.variant_id,
                    movement_type: "entry",
                    quantity_delta: entry.quantity,
                    unit_cost_minor: entry.unit_cost_minor,
                    total_cost_minor,
                    source_type: "manual",
                    source_id: None,
                    reference: reference.as_deref(),
                    note: note.as_deref(),
                    authorized_by: Some(authorized_by),
                    created_at: &now,
                },
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "inventory.entry_recorded",
                "inventory_movement",
                &movement_id,
                &format!("inventory-movement:{movement_id}:recorded"),
                &serde_json::json!({
                    "movementId": movement_id,
                    "variantId": entry.variant_id,
                    "quantityDelta": entry.quantity,
                    "unitCostMinor": entry.unit_cost_minor,
                    "totalCostMinor": total_cost_minor,
                    "reference": reference,
                    "note": note,
                    "authorizedBy": authorized_by,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }

        self.inventory_movement(organization_id, &movement_id)
    }

    pub fn adjust_inventory(
        &self,
        organization_id: &str,
        authorized_by: &str,
        adjustment: NewInventoryAdjustment,
    ) -> Result<InventoryMovementRecord, DataError> {
        if !(1..=999_999).contains(&adjustment.quantity_delta.unsigned_abs()) {
            return Err(DataError::InvalidValue(
                "inventory adjustment must be between -999999 and 999999".into(),
            ));
        }
        if adjustment.quantity_delta == 0 {
            return Err(DataError::InvalidValue(
                "inventory adjustment cannot be zero".into(),
            ));
        }
        let reason = adjustment.reason.trim();
        if reason.is_empty() || reason.chars().count() > 300 {
            return Err(DataError::InvalidValue(
                "the inventory adjustment reason must be between 1 and 300 characters".into(),
            ));
        }
        if let Some(cost) = adjustment.unit_cost_minor {
            if !(0..=MAX_AMOUNT_MINOR).contains(&cost) {
                return Err(DataError::InvalidValue(
                    "invalid inventory unit cost".into(),
                ));
            }
        }
        let movement_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            ensure_inventory_variant(&transaction, organization_id, &adjustment.variant_id)?;
            let unit_cost_minor = match adjustment.unit_cost_minor {
                Some(cost) => cost,
                None => {
                    inventory_average_cost(&transaction, organization_id, &adjustment.variant_id)?
                }
            };
            let total_cost_minor = adjustment
                .quantity_delta
                .checked_mul(unit_cost_minor)
                .ok_or_else(overflow)?;
            insert_inventory_movement(
                &transaction,
                InventoryMovementInsert {
                    id: &movement_id,
                    organization_id,
                    variant_id: &adjustment.variant_id,
                    movement_type: "adjustment",
                    quantity_delta: adjustment.quantity_delta,
                    unit_cost_minor,
                    total_cost_minor,
                    source_type: "adjustment",
                    source_id: None,
                    reference: None,
                    note: Some(reason),
                    authorized_by: Some(authorized_by),
                    created_at: &now,
                },
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "inventory.adjustment_recorded",
                "inventory_movement",
                &movement_id,
                &format!("inventory-movement:{movement_id}:recorded"),
                &serde_json::json!({
                    "movementId": movement_id,
                    "variantId": adjustment.variant_id,
                    "quantityDelta": adjustment.quantity_delta,
                    "unitCostMinor": unit_cost_minor,
                    "totalCostMinor": total_cost_minor,
                    "reason": reason,
                    "authorizedBy": authorized_by,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }

        self.inventory_movement(organization_id, &movement_id)
    }

    pub fn update_inventory_threshold(
        &self,
        organization_id: &str,
        updated_by: &str,
        variant_id: &str,
        min_stock: i64,
    ) -> Result<InventoryItemRecord, DataError> {
        if !(0..=999_999).contains(&min_stock) {
            return Err(DataError::InvalidValue(
                "minimum stock must be between 0 and 999999".into(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            ensure_inventory_variant(&transaction, organization_id, variant_id)?;
            transaction.execute(
                "INSERT INTO inventory_policies
                     (variant_id, organization_id, min_stock, updated_at, updated_by)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (variant_id) DO UPDATE SET
                     min_stock = excluded.min_stock,
                     updated_at = excluded.updated_at,
                     updated_by = excluded.updated_by",
                params![variant_id, organization_id, min_stock, now, updated_by],
            )?;
            transaction.commit()?;
        }

        self.list_inventory(organization_id)?
            .into_iter()
            .find(|item| item.variant_id == variant_id)
            .ok_or(DataError::CatalogVariantNotFound)
    }

    /// Creates the first owner and its organization-wide membership atomically.
    /// There is no default user or universal password.
    pub fn create_owner(
        &self,
        organization_id: &str,
        name: &str,
        password_hash: &str,
        recovery_code_hash: Option<&str>,
    ) -> Result<UserRecord, DataError> {
        let name = name.trim();
        if name.is_empty() || password_hash.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "owner name and password hash are required".into(),
            ));
        }

        let user_id = uuid::Uuid::new_v4().to_string();
        let membership_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        let owner_exists: bool = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM staff_memberships
                 WHERE organization_id = ?1 AND role = 'owner' AND status = 'active'
             )",
            params![organization_id],
            |row| row.get(0),
        )?;
        if owner_exists {
            return Err(DataError::OwnerAlreadyExists);
        }

        let user_exists: bool = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM users
                 WHERE organization_id = ?1 AND name = ?2 COLLATE NOCASE
             )",
            params![organization_id, name],
            |row| row.get(0),
        )?;
        if user_exists {
            return Err(DataError::UserAlreadyExists);
        }

        transaction.execute(
            "INSERT INTO users
                 (id, organization_id, name, password_hash, recovery_code_hash,
                  status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?6)",
            params![
                user_id,
                organization_id,
                name,
                password_hash,
                recovery_code_hash,
                created_at
            ],
        )?;
        transaction.execute(
            "INSERT INTO staff_memberships
                 (id, organization_id, user_id, role, status, created_at)
             VALUES (?1, ?2, ?3, 'owner', 'active', ?4)",
            params![membership_id, organization_id, user_id, created_at],
        )?;
        transaction.commit()?;

        Ok(UserRecord {
            id: user_id,
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            role: "owner".into(),
        })
    }

    pub fn list_permission_overrides(
        &self,
        organization_id: &str,
        user_id: &str,
    ) -> Result<Vec<PermissionOverrideRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT permission, effect
                 FROM staff_permission_overrides
                 WHERE organization_id = ?1 AND user_id = ?2
                 ORDER BY permission",
            )?;
            let rows = statement.query_map(params![organization_id, user_id], |row| {
                Ok(PermissionOverrideRecord {
                    permission: row.get(0)?,
                    effect: row.get(1)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    pub fn replace_permission_overrides(
        &self,
        organization_id: &str,
        user_id: &str,
        overrides: &[(String, String)],
    ) -> Result<Vec<PermissionOverrideRecord>, DataError> {
        let mut unique = HashMap::new();
        for (permission, effect) in overrides {
            if permission.trim().is_empty() || !matches!(effect.as_str(), "allow" | "deny") {
                return Err(DataError::InvalidValue(
                    "invalid permission override".into(),
                ));
            }
            unique.insert(permission.trim().to_string(), effect.clone());
        }

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let role: Option<String> = transaction
            .query_row(
                "SELECT s.role
                 FROM staff_memberships s
                 WHERE s.organization_id = ?1 AND s.user_id = ?2",
                params![organization_id, user_id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(role) = role else {
            return Err(DataError::StaffNotFound);
        };
        if role == "owner" {
            return Err(DataError::InvalidValue(
                "the owner permissions cannot be modified".into(),
            ));
        }

        transaction.execute(
            "DELETE FROM staff_permission_overrides
             WHERE organization_id = ?1 AND user_id = ?2",
            params![organization_id, user_id],
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        for (permission, effect) in &unique {
            transaction.execute(
                "INSERT INTO staff_permission_overrides
                     (organization_id, user_id, permission, effect, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                params![organization_id, user_id, permission, effect, now],
            )?;
        }
        transaction.commit()?;

        let mut records: Vec<_> = unique
            .into_iter()
            .map(|(permission, effect)| PermissionOverrideRecord { permission, effect })
            .collect();
        records.sort_by(|left, right| left.permission.cmp(&right.permission));
        Ok(records)
    }

    /// Returns the active membership used for login. Lookup is scoped to the
    /// installation's organization and is case-insensitive for the name.
    pub fn find_user_credentials(
        &self,
        organization_id: &str,
        name: &str,
    ) -> Result<Option<UserCredentialRecord>, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT u.id, u.organization_id, o.name, u.name, u.password_hash, s.role
                 FROM users u
                 JOIN organizations o
                   ON o.id = u.organization_id
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1
                   AND u.name = ?2 COLLATE NOCASE
                   AND u.status = 'active'
                   AND s.status = 'active'
                 LIMIT 1",
                params![organization_id, name.trim()],
                |row| {
                    Ok(UserCredentialRecord {
                        user_id: row.get(0)?,
                        organization_id: row.get(1)?,
                        organization_name: row.get(2)?,
                        name: row.get(3)?,
                        password_hash: row.get(4)?,
                        role: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(DataError::from)
        })
    }

    /// Reads the owner's current recovery hash. The hash is never returned to
    /// the webview; core code uses it only to verify a submitted code.
    pub fn owner_recovery(
        &self,
        organization_id: &str,
    ) -> Result<Option<OwnerRecoveryRecord>, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT u.id, u.name, u.recovery_code_hash
                 FROM users u
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1
                   AND u.status = 'active'
                   AND s.role = 'owner'
                   AND s.status = 'active'
                 LIMIT 1",
                params![organization_id],
                |row| {
                    Ok(OwnerRecoveryRecord {
                        user_id: row.get(0)?,
                        name: row.get(1)?,
                        recovery_code_hash: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(DataError::from)
        })
    }

    /// Replaces the owner's recovery hash. Generating a new code invalidates
    /// every previously printed code.
    pub fn set_owner_recovery_hash(
        &self,
        organization_id: &str,
        owner_id: &str,
        recovery_code_hash: &str,
    ) -> Result<(), DataError> {
        if recovery_code_hash.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "recovery code hash is required".into(),
            ));
        }
        let updated = self.with_connection(|conn| {
            conn.execute(
                "UPDATE users
                 SET recovery_code_hash = ?3, updated_at = ?4
                 WHERE organization_id = ?1 AND id = ?2
                   AND EXISTS (
                       SELECT 1 FROM staff_memberships s
                       WHERE s.organization_id = users.organization_id
                         AND s.user_id = users.id
                         AND s.role = 'owner'
                         AND s.status = 'active'
                   )",
                params![
                    organization_id,
                    owner_id,
                    recovery_code_hash,
                    chrono::Utc::now().to_rfc3339()
                ],
            )
            .map_err(DataError::from)
        })?;
        if updated == 0 {
            return Err(DataError::StaffNotFound);
        }
        Ok(())
    }

    /// Sets a new owner password and recovery hash in one transaction. The
    /// caller has already verified the old recovery code.
    pub fn reset_owner_password(
        &self,
        organization_id: &str,
        owner_id: &str,
        expected_recovery_code_hash: &str,
        password_hash: &str,
        recovery_code_hash: &str,
    ) -> Result<(), DataError> {
        if expected_recovery_code_hash.trim().is_empty()
            || password_hash.trim().is_empty()
            || recovery_code_hash.trim().is_empty()
        {
            return Err(DataError::InvalidValue(
                "recovery and password hashes are required".into(),
            ));
        }
        let updated = self.with_connection(|conn| {
            conn.execute(
                "UPDATE users
                 SET password_hash = ?4, recovery_code_hash = ?5, updated_at = ?6
                 WHERE organization_id = ?1 AND id = ?2
                   AND recovery_code_hash = ?3
                   AND EXISTS (
                       SELECT 1 FROM staff_memberships s
                       WHERE s.organization_id = users.organization_id
                         AND s.user_id = users.id
                         AND s.role = 'owner'
                         AND s.status = 'active'
                   )",
                params![
                    organization_id,
                    owner_id,
                    expected_recovery_code_hash,
                    password_hash,
                    recovery_code_hash,
                    chrono::Utc::now().to_rfc3339()
                ],
            )
            .map_err(DataError::from)
        })?;
        if updated == 0 {
            return Err(DataError::RecoveryCodeConflict);
        }
        Ok(())
    }

    pub fn mark_login(&self, user_id: &str) -> Result<(), DataError> {
        self.with_connection(|conn| {
            conn.execute(
                "UPDATE users SET last_login_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![user_id, chrono::Utc::now().to_rfc3339()],
            )?;
            Ok(())
        })
    }

    /// Appends a tamper-evident audit event. The hash chain is computed inside
    /// the same transaction as the insert, so each row commits with its
    /// predecessor hash. Triggers in migration 4 reject updates and deletes.
    #[allow(clippy::too_many_arguments)]
    pub fn record_audit_event(
        &self,
        organization_id: &str,
        actor_id: Option<&str>,
        action: &str,
        resource: &str,
        result: &str,
        origin: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), DataError> {
        if !matches!(result, "allowed" | "denied" | "failed") {
            return Err(DataError::InvalidValue("invalid audit result".into()));
        }
        if action.trim().is_empty() || resource.trim().is_empty() || origin.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "audit action, resource and origin cannot be empty".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let previous: Option<(i64, Option<String>)> = transaction
            .query_row(
                "SELECT sequence, event_hash
                 FROM audit_events
                 WHERE organization_id = ?1
                 ORDER BY sequence DESC
                 LIMIT 1",
                params![organization_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let sequence = previous.as_ref().map_or(1, |(sequence, _)| sequence + 1);
        let prev_hash = previous.and_then(|(_, hash)| hash);
        let event_hash = audit_event_hash(
            sequence,
            organization_id,
            &id,
            actor_id,
            action,
            resource,
            result,
            origin,
            correlation_id,
            &created_at,
            prev_hash.as_deref(),
        );
        transaction.execute(
            "INSERT INTO audit_events
                 (id, organization_id, actor_id, action, resource, result,
                  origin, correlation_id, sequence, prev_hash, event_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id,
                organization_id,
                actor_id,
                action,
                resource,
                result,
                origin,
                correlation_id,
                sequence,
                prev_hash,
                event_hash,
                created_at
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    /// Returns newest audit rows first for the whole installation.
    pub fn list_audit_events(
        &self,
        organization_id: &str,
        limit: i64,
    ) -> Result<Vec<AuditEventRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT a.id, a.sequence, a.actor_id, u.name, a.action,
                        a.resource, a.result, a.origin, a.correlation_id, a.created_at
                 FROM audit_events a
                 LEFT JOIN users u ON u.id = a.actor_id
                 WHERE a.organization_id = ?1
                 ORDER BY a.sequence DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(params![organization_id, limit], |row| {
                Ok(AuditEventRecord {
                    id: row.get(0)?,
                    sequence: row.get(1)?,
                    actor_id: row.get(2)?,
                    actor_name: row.get(3)?,
                    action: row.get(4)?,
                    resource: row.get(5)?,
                    result: row.get(6)?,
                    origin: row.get(7)?,
                    correlation_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    /// Recomputes the hash chain from the oldest hashed event to the newest.
    /// Legacy rows migrated from schema 3 have no hash and are skipped; the
    /// chain starts at the first post-migration append.
    pub fn verify_audit_chain(
        &self,
        organization_id: &str,
    ) -> Result<AuditIntegrityRecord, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT sequence, id, actor_id, action, resource, result,
                        origin, correlation_id, created_at, prev_hash, event_hash
                 FROM audit_events
                 WHERE organization_id = ?1 AND event_hash IS NOT NULL
                 ORDER BY sequence ASC",
            )?;
            let mut rows = statement.query(params![organization_id])?;
            let mut checked_events = 0;
            let mut previous_sequence: Option<i64> = None;
            let mut previous_hash: Option<String> = None;
            while let Some(row) = rows.next()? {
                checked_events += 1;
                let sequence: i64 = row.get(0)?;
                let id: String = row.get(1)?;
                let actor_id: Option<String> = row.get(2)?;
                let action: String = row.get(3)?;
                let resource: String = row.get(4)?;
                let result: String = row.get(5)?;
                let origin: String = row.get(6)?;
                let correlation_id: Option<String> = row.get(7)?;
                let created_at: String = row.get(8)?;
                let prev_hash: Option<String> = row.get(9)?;
                let event_hash: String = row.get(10)?;
                let expected = audit_event_hash(
                    sequence,
                    organization_id,
                    &id,
                    actor_id.as_deref(),
                    &action,
                    &resource,
                    &result,
                    &origin,
                    correlation_id.as_deref(),
                    &created_at,
                    prev_hash.as_deref(),
                );
                let sequence_ok = previous_sequence.is_none_or(|value| sequence == value + 1);
                let hash_ok = expected == event_hash && prev_hash == previous_hash;
                if !sequence_ok || !hash_ok {
                    return Ok(AuditIntegrityRecord {
                        valid: false,
                        checked_events,
                        first_broken_sequence: Some(sequence),
                    });
                }
                previous_sequence = Some(sequence);
                previous_hash = Some(event_hash);
            }
            Ok(AuditIntegrityRecord {
                valid: true,
                checked_events,
                first_broken_sequence: None,
            })
        })
    }

    /// Inserts an outbox event once per `(organization, idempotency_key)` and
    /// returns the durable row, including when it already existed.
    #[allow(clippy::too_many_arguments)]
    pub fn record_module_event(
        &self,
        organization_id: &str,
        direction: &str,
        event_type: &str,
        aggregate: &str,
        idempotency_key: &str,
        payload_hash: &str,
    ) -> Result<RecordedModuleEvent, DataError> {
        if idempotency_key.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "idempotency key cannot be empty".into(),
            ));
        }

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let inserted = transaction.execute(
            "INSERT INTO module_events
                 (id, organization_id, direction, event_type, aggregate,
                  idempotency_key, payload_hash, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'applied', ?8)
             ON CONFLICT (organization_id, idempotency_key) DO NOTHING",
            params![
                id,
                organization_id,
                direction,
                event_type,
                aggregate,
                idempotency_key,
                payload_hash,
                created_at
            ],
        )?;

        let event = transaction.query_row(
            "SELECT id, event_type, idempotency_key, payload_hash, created_at
             FROM module_events
             WHERE organization_id = ?1 AND idempotency_key = ?2",
            params![organization_id, idempotency_key],
            |row| {
                Ok(ModuleEventRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    idempotency_key: row.get(2)?,
                    payload_hash: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )?;
        transaction.commit()?;

        Ok(RecordedModuleEvent {
            event,
            inserted: inserted == 1,
        })
    }

    pub fn list_module_events(
        &self,
        aggregate: &str,
        limit: usize,
    ) -> Result<Vec<ModuleEventRecord>, DataError> {
        let limit = limit.clamp(1, 100) as i64;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, event_type, idempotency_key, payload_hash, created_at
                 FROM module_events
                 WHERE aggregate = ?1
                 ORDER BY rowid DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(params![aggregate, limit], |row| {
                Ok(ModuleEventRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    idempotency_key: row.get(2)?,
                    payload_hash: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    pub fn pending_sync_events(
        &self,
        organization_id: &str,
        due_at: &str,
        limit: usize,
    ) -> Result<Vec<SyncOutboxRecord>, DataError> {
        let limit = limit.clamp(1, 100) as i64;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, event_type, aggregate_type, aggregate_id, idempotency_key,
                        payload_json, payload_hash, state, attempts, last_error,
                        next_attempt_at, sent_at, created_at
                 FROM sync_outbox
                 WHERE organization_id = ?1 AND state = 'pending'
                   AND next_attempt_at <= ?2
                 ORDER BY created_at, id
                 LIMIT ?3",
            )?;
            let rows = statement
                .query_map(
                    params![organization_id, due_at, limit],
                    map_sync_outbox_record,
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn list_sync_outbox(
        &self,
        organization_id: &str,
        limit: usize,
    ) -> Result<Vec<SyncOutboxRecord>, DataError> {
        let limit = limit.clamp(1, 200) as i64;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, event_type, aggregate_type, aggregate_id, idempotency_key,
                        payload_json, payload_hash, state, attempts, last_error,
                        next_attempt_at, sent_at, created_at
                 FROM sync_outbox
                 WHERE organization_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = statement
                .query_map(params![organization_id, limit], map_sync_outbox_record)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn mark_sync_event_sent(
        &self,
        organization_id: &str,
        event_id: &str,
        sent_at: &str,
    ) -> Result<bool, DataError> {
        self.with_connection(|conn| {
            let updated = conn.execute(
                "UPDATE sync_outbox
                 SET state = 'sent', sent_at = ?1, last_error = NULL
                 WHERE organization_id = ?2 AND id = ?3 AND state = 'pending'",
                params![sent_at, organization_id, event_id],
            )?;
            Ok(updated == 1)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fail_sync_event(
        &self,
        organization_id: &str,
        event_id: &str,
        error: &str,
        next_attempt_at: &str,
        max_attempts: i64,
    ) -> Result<SyncFailureRecord, DataError> {
        if max_attempts < 1 {
            return Err(DataError::InvalidValue(
                "sync attempts must be at least one".into(),
            ));
        }
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let current = transaction
            .query_row(
                "SELECT attempts, state FROM sync_outbox
                 WHERE organization_id = ?1 AND id = ?2",
                params![organization_id, event_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((attempts, state)) = current else {
            return Err(DataError::InvalidValue(
                "the sync event does not exist in this gym".into(),
            ));
        };
        if state != "pending" {
            return Ok(SyncFailureRecord { state, attempts });
        }
        let attempts = attempts + 1;
        let state = if attempts >= max_attempts {
            "dead_letter"
        } else {
            "pending"
        };
        let error: String = error.chars().take(500).collect();
        transaction.execute(
            "UPDATE sync_outbox
             SET attempts = ?1, state = ?2, last_error = ?3, next_attempt_at = ?4
             WHERE organization_id = ?5 AND id = ?6",
            params![
                attempts,
                state,
                error,
                next_attempt_at,
                organization_id,
                event_id
            ],
        )?;
        transaction.commit()?;
        Ok(SyncFailureRecord {
            state: state.into(),
            attempts,
        })
    }

    pub fn sync_cursor(&self, organization_id: &str) -> Result<i64, DataError> {
        self.with_connection(|conn| {
            let cursor = conn
                .query_row(
                    "SELECT last_server_sequence FROM sync_state WHERE organization_id = ?1",
                    params![organization_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?;
            Ok(cursor.unwrap_or(0))
        })
    }

    pub fn apply_inbox_event(
        &self,
        organization_id: &str,
        event: InboundSyncEvent,
    ) -> Result<bool, DataError> {
        if event.event_id.trim().is_empty()
            || event.idempotency_key.trim().is_empty()
            || event.server_sequence < 1
        {
            return Err(DataError::InvalidValue(
                "the inbound sync event is missing identifiers".into(),
            ));
        }
        let payload_hash = sha256_hex(event.payload_json.as_bytes());
        let received_at = chrono::Utc::now().to_rfc3339();
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let cursor: i64 = transaction
            .query_row(
                "SELECT last_server_sequence FROM sync_state WHERE organization_id = ?1",
                params![organization_id],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        if event.server_sequence <= cursor {
            transaction.commit()?;
            return Ok(false);
        }
        let inserted = transaction.execute(
            "INSERT INTO sync_inbox
                 (event_id, organization_id, server_sequence, event_type,
                  aggregate_type, aggregate_id, idempotency_key, payload_json,
                  payload_hash, occurred_at, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT DO NOTHING",
            params![
                event.event_id,
                organization_id,
                event.server_sequence,
                event.event_type,
                event.aggregate_type,
                event.aggregate_id,
                event.idempotency_key,
                event.payload_json,
                payload_hash,
                event.occurred_at,
                received_at
            ],
        )?;
        if inserted == 1 {
            transaction.execute(
                "INSERT INTO sync_state (organization_id, last_server_sequence, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (organization_id) DO UPDATE SET
                     last_server_sequence = MAX(last_server_sequence, excluded.last_server_sequence),
                     updated_at = excluded.updated_at",
                params![organization_id, event.server_sequence, received_at],
            )?;
        }
        transaction.commit()?;
        Ok(inserted == 1)
    }

    pub fn list_sync_inbox(
        &self,
        organization_id: &str,
        limit: usize,
    ) -> Result<Vec<SyncInboxRecord>, DataError> {
        let limit = limit.clamp(1, 200) as i64;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT event_id, server_sequence, event_type, aggregate_type,
                        aggregate_id, idempotency_key, payload_json, payload_hash,
                        occurred_at, received_at
                 FROM sync_inbox
                 WHERE organization_id = ?1
                 ORDER BY server_sequence DESC
                 LIMIT ?2",
            )?;
            let rows = statement
                .query_map(params![organization_id, limit], |row| {
                    Ok(SyncInboxRecord {
                        event_id: row.get(0)?,
                        server_sequence: row.get(1)?,
                        event_type: row.get(2)?,
                        aggregate_type: row.get(3)?,
                        aggregate_id: row.get(4)?,
                        idempotency_key: row.get(5)?,
                        payload_json: row.get(6)?,
                        payload_hash: row.get(7)?,
                        occurred_at: row.get(8)?,
                        received_at: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    /// Copies the open encrypted database to `destination` without exposing
    /// plaintext. SQLCipher re-encrypts with the current key.
    pub fn vacuum_into(&self, destination: &Path) -> Result<(), DataError> {
        if destination.exists() {
            fs::remove_file(destination)?;
        }

        self.with_connection(|conn| {
            conn.execute(
                "VACUUM INTO ?1",
                params![destination.to_string_lossy().as_ref()],
            )
            .map_err(DataError::from)
        })?;
        Ok(())
    }

    pub fn integrity_check(&self) -> Result<bool, DataError> {
        self.with_connection(|conn| {
            let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
            Ok(result.eq_ignore_ascii_case("ok"))
        })
    }

    /// Closes the connection and its WAL after a best-effort checkpoint. Every
    /// file handle is released so the database can be replaced on any platform.
    pub fn close(&self) -> Result<(), DataError> {
        let mut guard = self.lock()?;
        if let Some(conn) = guard.take() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            drop(conn);
        }
        Ok(())
    }

    /// Reopens the live connection after a restore. The database file is
    /// assumed to be already authenticated by the restore routine.
    pub fn reopen(&self, key: &DbKey) -> Result<(), DataError> {
        let conn = Self::open_connection(&self.path, key)?;
        {
            let mut guard = self.lock()?;
            *guard = Some(conn);
        }
        self.migrate()
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, DataError>,
    ) -> Result<T, DataError> {
        let guard = self.lock()?;
        let conn = guard.as_ref().ok_or(DataError::DatabaseClosed)?;
        operation(conn)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Option<Connection>>, DataError> {
        self.conn.lock().map_err(|_| DataError::LockPoisoned)
    }
}

fn normalized_report_range(from: &str, to: &str) -> Result<(String, String), DataError> {
    let parse = |value: &str| {
        chrono::DateTime::parse_from_rfc3339(value.trim())
            .map(|value| value.with_timezone(&chrono::Utc))
            .map_err(|_| DataError::InvalidValue("report dates must be RFC 3339 timestamps".into()))
    };
    let from = parse(from)?;
    let to = parse(to)?;
    if from >= to {
        return Err(DataError::InvalidValue(
            "the report start must be before the end".into(),
        ));
    }
    if to - from > chrono::TimeDelta::days(366) {
        return Err(DataError::InvalidValue(
            "the report range cannot exceed 366 days".into(),
        ));
    }
    Ok((
        from.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        to.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    ))
}

fn required_text(
    value: &str,
    field: &str,
    min_chars: usize,
    max_chars: usize,
) -> Result<String, DataError> {
    let value = value.trim();
    let length = value.chars().count();
    if length < min_chars || length > max_chars {
        return Err(DataError::InvalidValue(format!(
            "{field} must be between {min_chars} and {max_chars} characters"
        )));
    }
    Ok(value.to_string())
}

fn optional_text(
    value: Option<&str>,
    field: &str,
    min_chars: usize,
    max_chars: usize,
) -> Result<Option<String>, DataError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let length = value.chars().count();
    if length < min_chars || length > max_chars {
        return Err(DataError::InvalidValue(format!(
            "{field} must be between {min_chars} and {max_chars} characters"
        )));
    }
    Ok(Some(value.to_string()))
}

struct InventoryMovementInsert<'a> {
    id: &'a str,
    organization_id: &'a str,
    variant_id: &'a str,
    movement_type: &'a str,
    quantity_delta: i64,
    unit_cost_minor: i64,
    total_cost_minor: i64,
    source_type: &'a str,
    source_id: Option<&'a str>,
    reference: Option<&'a str>,
    note: Option<&'a str>,
    authorized_by: Option<&'a str>,
    created_at: &'a str,
}

fn map_inventory_movement(row: &Row<'_>) -> rusqlite::Result<InventoryMovementRecord> {
    Ok(InventoryMovementRecord {
        id: row.get(0)?,
        variant_id: row.get(1)?,
        movement_type: row.get(2)?,
        quantity_delta: row.get(3)?,
        unit_cost_minor: row.get(4)?,
        total_cost_minor: row.get(5)?,
        source_type: row.get(6)?,
        source_id: row.get(7)?,
        reference: row.get(8)?,
        note: row.get(9)?,
        authorized_by: row.get(10)?,
        authorized_by_name: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn normalize_optional_text(
    value: Option<String>,
    max_chars: usize,
    field: &str,
) -> Result<Option<String>, DataError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max_chars {
        return Err(DataError::InvalidValue(format!(
            "the {field} must have at most {max_chars} characters"
        )));
    }
    Ok(Some(value.to_owned()))
}

fn ensure_inventory_variant(
    transaction: &Transaction<'_>,
    organization_id: &str,
    variant_id: &str,
) -> Result<(), DataError> {
    let unit = transaction
        .query_row(
            "SELECT unit FROM catalog_variants
             WHERE organization_id = ?1 AND id = ?2",
            params![organization_id, variant_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(DataError::CatalogVariantNotFound)?;
    if unit != "piece" {
        return Err(DataError::InvalidValue(
            "inventory only applies to piece variants".into(),
        ));
    }
    Ok(())
}

fn inventory_average_cost(
    transaction: &Transaction<'_>,
    organization_id: &str,
    variant_id: &str,
) -> Result<i64, DataError> {
    if let Some(cost) = transaction
        .query_row(
            "SELECT avg_unit_cost_minor FROM inventory_balances
             WHERE organization_id = ?1 AND variant_id = ?2 AND quantity > 0",
            params![organization_id, variant_id],
            |row| row.get(0),
        )
        .optional()?
    {
        return Ok(cost);
    }
    Ok(transaction
        .query_row(
            "SELECT CAST(ROUND(CAST(SUM(total_cost_minor) AS REAL)
                                / SUM(quantity_delta)) AS INTEGER)
             FROM inventory_movements
             WHERE organization_id = ?1 AND variant_id = ?2
             GROUP BY organization_id, variant_id
             HAVING SUM(quantity_delta) > 0",
            params![organization_id, variant_id],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0))
}

fn insert_inventory_movement(
    transaction: &Transaction<'_>,
    movement: InventoryMovementInsert<'_>,
) -> Result<(), DataError> {
    transaction.execute(
        "INSERT INTO inventory_movements
             (id, organization_id, variant_id, movement_type,
              quantity_delta, unit_cost_minor, total_cost_minor, source_type,
              source_id, reference, note, authorized_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            movement.id,
            movement.organization_id,
            movement.variant_id,
            movement.movement_type,
            movement.quantity_delta,
            movement.unit_cost_minor,
            movement.total_cost_minor,
            movement.source_type,
            movement.source_id,
            movement.reference,
            movement.note,
            movement.authorized_by,
            movement.created_at
        ],
    )?;
    Ok(())
}

struct PaymentChargeInsert<'a> {
    id: &'a str,
    organization_id: &'a str,
    sale_id: &'a str,
    sale_return_id: Option<&'a str>,
    kind: &'a str,
    method: &'a str,
    provider: &'a str,
    status: &'a str,
    amount_minor: i64,
    currency: &'a str,
    external_reference: Option<&'a str>,
    idempotency_key: &'a str,
    request_hash: &'a str,
    created_at: &'a str,
}

fn insert_payment_charge(
    transaction: &Transaction<'_>,
    charge: PaymentChargeInsert<'_>,
) -> Result<(), DataError> {
    transaction.execute(
        "INSERT INTO payment_charges
             (id, organization_id, sale_id, sale_return_id, kind, method, provider,
              status, amount_minor, currency, external_reference, idempotency_key,
              request_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            charge.id,
            charge.organization_id,
            charge.sale_id,
            charge.sale_return_id,
            charge.kind,
            charge.method,
            charge.provider,
            charge.status,
            charge.amount_minor,
            charge.currency,
            charge.external_reference,
            charge.idempotency_key,
            charge.request_hash,
            charge.created_at
        ],
    )?;
    Ok(())
}

fn map_sync_outbox_record(row: &Row<'_>) -> rusqlite::Result<SyncOutboxRecord> {
    Ok(SyncOutboxRecord {
        id: row.get(0)?,
        event_type: row.get(1)?,
        aggregate_type: row.get(2)?,
        aggregate_id: row.get(3)?,
        idempotency_key: row.get(4)?,
        payload_json: row.get(5)?,
        payload_hash: row.get(6)?,
        state: row.get(7)?,
        attempts: row.get(8)?,
        last_error: row.get(9)?,
        next_attempt_at: row.get(10)?,
        sent_at: row.get(11)?,
        created_at: row.get(12)?,
    })
}

#[allow(clippy::too_many_arguments)]
fn enqueue_sync_event(
    transaction: &Transaction<'_>,
    organization_id: &str,
    event_type: &str,
    aggregate_type: &str,
    aggregate_id: &str,
    idempotency_key: &str,
    payload: &serde_json::Value,
    created_at: &str,
) -> Result<(), DataError> {
    let payload_json = serde_json::to_string(payload)?;
    let payload_hash = sha256_hex(payload_json.as_bytes());
    transaction.execute(
        "INSERT INTO sync_outbox
             (id, organization_id, event_type, aggregate_type, aggregate_id,
              idempotency_key, payload_json, payload_hash, state, attempts,
              next_attempt_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', 0, ?9, ?9)
         ON CONFLICT (organization_id, idempotency_key) DO NOTHING",
        params![
            uuid::Uuid::new_v4().to_string(),
            organization_id,
            event_type,
            aggregate_type,
            aggregate_id,
            idempotency_key,
            payload_json,
            payload_hash,
            created_at
        ],
    )?;
    Ok(())
}

fn validate_catalog_unit(value: &str) -> Result<(), DataError> {
    if matches!(value, "piece" | "service" | "session" | "package") {
        Ok(())
    } else {
        Err(DataError::InvalidValue("invalid catalog unit".into()))
    }
}

fn validate_catalog_status(value: &str) -> Result<(), DataError> {
    if matches!(value, "active" | "archived") {
        Ok(())
    } else {
        Err(DataError::InvalidValue("invalid catalog status".into()))
    }
}

fn overflow() -> DataError {
    DataError::InvalidValue("amount is too large".into())
}

fn line_totals(
    unit_amount_minor: i64,
    quantity: i64,
    tax_rate_bps: i64,
    price_includes_tax: bool,
) -> Result<(i64, i64, i64), DataError> {
    let gross = i128::from(unit_amount_minor)
        .checked_mul(i128::from(quantity))
        .ok_or_else(overflow)?;
    let (subtotal, tax, total) = if price_includes_tax {
        let denominator = 10_000 + i128::from(tax_rate_bps);
        let subtotal = (gross * 20_000 + denominator) / (denominator * 2);
        (subtotal, gross - subtotal, gross)
    } else {
        let tax = (gross * i128::from(tax_rate_bps) + 5_000) / 10_000;
        (gross, tax, gross + tax)
    };
    let limit = i128::from(MAX_AMOUNT_MINOR);
    if subtotal > limit || tax > limit || total > limit {
        return Err(overflow());
    }
    Ok((subtotal as i64, tax as i64, total as i64))
}

fn validate_new_price(price: NewCatalogPrice) -> Result<ValidatedCatalogPrice, DataError> {
    if price.amount_minor < 0 {
        return Err(DataError::InvalidValue(
            "price must be zero or greater".into(),
        ));
    }
    if !(0..=10_000).contains(&price.tax_rate_bps) {
        return Err(DataError::InvalidValue(
            "tax rate must be between 0 and 10000 basis points".into(),
        ));
    }
    let effective_from = price
        .effective_from
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|date| {
                    date.with_timezone(&chrono::Utc)
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                })
                .map_err(|_| {
                    DataError::InvalidValue("price effective date must be RFC 3339".into())
                })
        })
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    Ok(ValidatedCatalogPrice {
        amount_minor: price.amount_minor,
        tax_rate_bps: price.tax_rate_bps,
        price_includes_tax: price.price_includes_tax,
        effective_from,
    })
}

fn organization_currency(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
) -> Result<String, DataError> {
    conn.query_row(
        "SELECT currency FROM organizations WHERE id = ?1",
        params![organization_id],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| DataError::InvalidValue("the gym does not exist".into()))
}

fn ensure_unique_product_name(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM catalog_products
             WHERE organization_id = ?1
               AND name = ?2 COLLATE NOCASE
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, name, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(DataError::CatalogProductAlreadyExists);
    }
    Ok(())
}

fn ensure_unique_sku(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    sku: &str,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM catalog_variants
             WHERE organization_id = ?1
               AND sku = ?2 COLLATE NOCASE
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, sku, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(DataError::CatalogSkuAlreadyExists);
    }
    Ok(())
}

fn ensure_unique_barcode(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    barcode: Option<&str>,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let Some(barcode) = barcode else {
        return Ok(());
    };
    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM catalog_variants
             WHERE organization_id = ?1
               AND barcode = ?2
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, barcode, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(DataError::CatalogBarcodeAlreadyExists);
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn audit_event_hash(
    sequence: i64,
    organization_id: &str,
    id: &str,
    actor_id: Option<&str>,
    action: &str,
    resource: &str,
    result: &str,
    origin: &str,
    correlation_id: Option<&str>,
    created_at: &str,
    prev_hash: Option<&str>,
) -> String {
    fn push_field(buffer: &mut String, label: &str, value: Option<&str>) {
        buffer.push_str(label);
        buffer.push('=');
        match value {
            Some(value) => {
                buffer.push_str(&value.len().to_string());
                buffer.push(':');
                buffer.push_str(value);
            }
            None => buffer.push('-'),
        }
        buffer.push('\n');
    }

    let mut buffer = String::with_capacity(512);
    buffer.push_str("fitness-gym-audit-v2\n");
    push_field(&mut buffer, "sequence", Some(&sequence.to_string()));
    push_field(&mut buffer, "organization", Some(organization_id));
    push_field(&mut buffer, "id", Some(id));
    push_field(&mut buffer, "actor", actor_id);
    push_field(&mut buffer, "action", Some(action));
    push_field(&mut buffer, "resource", Some(resource));
    push_field(&mut buffer, "result", Some(result));
    push_field(&mut buffer, "origin", Some(origin));
    push_field(&mut buffer, "correlation", correlation_id);
    push_field(&mut buffer, "created_at", Some(created_at));
    push_field(&mut buffer, "prev_hash", prev_hash);
    crate::sha256_hex(buffer)
}

/// Opens an encrypted database with `key` and runs `PRAGMA integrity_check`.
/// Used by restore to validate a snapshot before replacing the live file.
pub fn verify_encrypted_database(path: &Path, key: &DbKey) -> Result<(), DataError> {
    let conn = Db::open_connection(path, key)?;
    let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result.eq_ignore_ascii_case("ok") {
        Ok(())
    } else {
        Err(DataError::InvalidValue(format!(
            "integrity check failed: {result}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp(dir: &tempfile::TempDir, key: &DbKey) -> Db {
        Db::open(dir.path().join("gym.db"), key).expect("opens database")
    }

    #[test]
    fn database_is_encrypted_on_disk() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        db.create_organization("Gimnasio Centro")
            .expect("creates org");

        let path = db.path().to_path_buf();
        drop(db);

        let header = &fs::read(&path).expect("reads file")[..16];
        assert_ne!(header, b"SQLite format 3\0");
    }

    #[test]
    fn migrations_are_reproducible() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let path = dir.path().join("gym.db");

        let first = Db::open(&path, &key).expect("first open");
        let first_status = first.status().expect("first status");
        drop(first);

        let second = Db::open(&path, &key).expect("second open");
        let second_status = second.status().expect("second status");

        assert_eq!(first_status.schema_version, second_status.schema_version);
        assert_eq!(second_status.schema_version, 21);
        assert_eq!(second_status.organizations, 0);
        assert!(second_status.encrypted);
        assert!(second_status.integrity_ok);
    }

    #[test]
    fn roles_permissions_migration_preserves_and_maps_legacy_staff() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let path = dir.path().join("gym.db");

        {
            let conn = Db::open_connection(&path, &key).expect("opens connection");
            conn.execute_batch(
                "CREATE TABLE schema_migrations (
                     version INTEGER PRIMARY KEY NOT NULL,
                     name TEXT NOT NULL,
                     checksum TEXT NOT NULL,
                     applied_at TEXT NOT NULL
                 ) STRICT;",
            )
            .expect("creates migration table");
            for migration in MIGRATIONS
                .iter()
                .filter(|migration| migration.version <= 19)
            {
                conn.execute_batch(migration.up)
                    .expect("applies legacy migration");
                conn.execute(
                    "INSERT INTO schema_migrations (version, name, checksum, applied_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        migration.version,
                        migration.name,
                        hex::encode(Sha256::digest(migration.up.as_bytes())),
                        chrono::Utc::now().to_rfc3339()
                    ],
                )
                .expect("records legacy migration");
            }
            conn.pragma_update(None, "user_version", 19)
                .expect("sets legacy version");
            conn.execute_batch(
                "INSERT INTO organizations (id, name, created_at)
                 VALUES ('org', 'Gimnasio Centro', '2026-01-01T00:00:00Z');
                 INSERT INTO users
                     (id, organization_id, name, password_hash, status, created_at, updated_at)
                 VALUES
                     ('owner', 'org', 'Ana', 'hash', 'active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                     ('cashier', 'org', 'Carlos', 'hash', 'active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                     ('accounting', 'org', 'Luisa', 'hash', 'active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO staff_memberships
                     (id, organization_id, user_id, role, status, created_at)
                 VALUES
                     ('m-owner', 'org', 'owner', 'owner', 'active', '2026-01-01T00:00:00Z'),
                     ('m-cashier', 'org', 'cashier', 'cashier', 'active', '2026-01-01T00:00:00Z'),
                     ('m-accounting', 'org', 'accounting', 'accounting', 'active', '2026-01-01T00:00:00Z');",
            )
            .expect("seeds legacy staff");
        }

        let db = Db::open(&path, &key).expect("migrates to current schema");
        assert_eq!(db.status().expect("status").schema_version, 21);
        let staff = db.list_staff("org").expect("lists migrated staff");
        assert_eq!(staff.len(), 3);
        assert_eq!(
            staff
                .iter()
                .find(|member| member.user_id == "owner")
                .expect("owner")
                .role,
            "owner"
        );
        assert_eq!(
            staff
                .iter()
                .find(|member| member.user_id == "cashier")
                .expect("cashier")
                .role,
            "receptionist"
        );
        assert_eq!(
            staff
                .iter()
                .find(|member| member.user_id == "accounting")
                .expect("accounting")
                .role,
            "advisor"
        );
        assert_eq!(
            staff
                .iter()
                .find(|member| member.user_id == "accounting")
                .expect("accounting")
                .status,
            "revoked"
        );
        assert!(db
            .find_user_credentials("org", "Luisa")
            .expect("looks up disabled user")
            .is_none());
    }

    #[test]
    fn only_one_organization_per_installation() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);

        db.create_organization("Gimnasio Centro")
            .expect("creates the first organization");
        let error = db
            .create_organization("Gimnasio Duplicado")
            .expect_err("must reject a second organization");
        assert!(matches!(error, DataError::OrganizationAlreadyExists));
        assert_eq!(db.status().expect("status").organizations, 1);
    }

    #[test]
    fn owner_bootstrap_is_atomic_and_unique() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");

        assert!(!db.has_owner().expect("owner query"));
        let owner = db
            .create_owner(
                &organization_id,
                "Ana Pérez",
                "$argon2id$dummy",
                Some("$argon2id$recovery"),
            )
            .expect("creates owner");
        assert_eq!(owner.name, "Ana Pérez");
        assert_eq!(owner.role, "owner");
        assert!(db.has_owner().expect("owner query"));

        let duplicate = db
            .create_owner(&organization_id, "Luis", "$argon2id$dummy", None)
            .expect_err("must reject a second owner");
        assert!(matches!(duplicate, DataError::OwnerAlreadyExists));

        let credentials = db
            .find_user_credentials(&organization_id, "ana pérez")
            .expect("looks up user")
            .expect("user exists");
        assert_eq!(credentials.user_id, owner.id);
        assert_eq!(credentials.role, "owner");
        assert_eq!(credentials.password_hash, "$argon2id$dummy");

        let recovery = db
            .owner_recovery(&organization_id)
            .expect("reads owner recovery")
            .expect("owner recovery exists");
        assert_eq!(recovery.user_id, owner.id);
        assert_eq!(
            recovery.recovery_code_hash.as_deref(),
            Some("$argon2id$recovery")
        );

        db.set_owner_recovery_hash(&organization_id, &owner.id, "$argon2id$new-recovery")
            .expect("rotates recovery code");
        db.reset_owner_password(
            &organization_id,
            &owner.id,
            "$argon2id$new-recovery",
            "$argon2id$new-password",
            "$argon2id$reset-recovery",
        )
        .expect("resets owner password");

        let reused = db
            .reset_owner_password(
                &organization_id,
                &owner.id,
                "$argon2id$new-recovery",
                "$argon2id$reused-password",
                "$argon2id$reused-recovery",
            )
            .expect_err("must reject a reused recovery hash");
        assert!(matches!(reused, DataError::RecoveryCodeConflict));

        let credentials = db
            .find_user_credentials(&organization_id, "Ana Pérez")
            .expect("looks up user")
            .expect("user exists");
        assert_eq!(credentials.password_hash, "$argon2id$new-password");
        assert_eq!(
            db.owner_recovery(&organization_id)
                .expect("reads owner recovery")
                .expect("owner recovery exists")
                .recovery_code_hash
                .as_deref(),
            Some("$argon2id$reset-recovery")
        );
    }

    #[test]
    fn staff_memberships_are_updatable() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        db.create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");

        let staff = db
            .create_staff(
                &organization_id,
                "Carlos Caja",
                "$argon2id$staff",
                "receptionist",
            )
            .expect("creates staff");
        assert_eq!(staff.role, "receptionist");
        assert_eq!(staff.status, "active");
        assert_eq!(db.list_staff(&organization_id).expect("lists").len(), 2);
        assert!(matches!(
            db.create_staff(
                &organization_id,
                "carlos caja",
                "$argon2id$x",
                "receptionist"
            )
            .expect_err("duplicate user"),
            DataError::UserAlreadyExists
        ));

        let credentials = db
            .find_user_credentials(&organization_id, "carlos caja")
            .expect("looks up staff")
            .expect("staff exists");
        assert_eq!(credentials.role, "receptionist");

        let updated = db
            .update_staff(&organization_id, &staff.user_id, "advisor", "revoked")
            .expect("revokes staff");
        assert_eq!(updated.role, "advisor");
        assert_eq!(updated.status, "revoked");
        assert!(db
            .find_user_credentials(&organization_id, "Carlos Caja")
            .expect("looks up revoked staff")
            .is_none());

        db.update_staff(&organization_id, &staff.user_id, "manager", "active")
            .expect("reactivates staff");
        assert_eq!(
            db.find_user_credentials(&organization_id, "Carlos Caja")
                .expect("looks up staff")
                .expect("staff exists")
                .role,
            "manager"
        );
        assert!(matches!(
            db.update_staff(&organization_id, "missing", "receptionist", "active")
                .expect_err("missing staff"),
            DataError::StaffNotFound
        ));
    }

    #[test]
    fn permission_overrides_are_replaced_and_owner_is_protected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        let staff = db
            .create_staff(
                &organization_id,
                "Carlos Recepción",
                "$argon2id$staff",
                "receptionist",
            )
            .expect("creates staff");

        let records = db
            .replace_permission_overrides(
                &organization_id,
                &staff.user_id,
                &[
                    ("viewFinance".into(), "allow".into()),
                    ("operatePos".into(), "deny".into()),
                ],
            )
            .expect("replaces overrides");
        assert_eq!(
            records,
            vec![
                PermissionOverrideRecord {
                    permission: "operatePos".into(),
                    effect: "deny".into(),
                },
                PermissionOverrideRecord {
                    permission: "viewFinance".into(),
                    effect: "allow".into(),
                },
            ]
        );
        assert_eq!(
            db.list_permission_overrides(&organization_id, &staff.user_id)
                .expect("lists overrides"),
            records
        );

        assert!(db
            .replace_permission_overrides(&organization_id, &staff.user_id, &[])
            .expect("clears overrides")
            .is_empty());
        assert!(db
            .list_permission_overrides(&organization_id, &staff.user_id)
            .expect("lists empty overrides")
            .is_empty());

        assert!(matches!(
            db.replace_permission_overrides(
                &organization_id,
                &owner.id,
                &[("viewFinance".into(), "allow".into())],
            )
            .expect_err("owner permissions are immutable"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.replace_permission_overrides(
                &organization_id,
                "missing",
                &[("viewFinance".into(), "allow".into())],
            )
            .expect_err("missing staff"),
            DataError::StaffNotFound
        ));
        assert!(matches!(
            db.replace_permission_overrides(
                &organization_id,
                &staff.user_id,
                &[("viewFinance".into(), "maybe".into())],
            )
            .expect_err("invalid effect"),
            DataError::InvalidValue(_)
        ));
    }

    #[test]
    fn catalog_prices_are_versioned_and_append_only() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");

        let product = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Membresía mensual".into(),
                    category: Some("Membresías".into()),
                    variant_name: "Mensual".into(),
                    sku: "MEM-MEN".into(),
                    barcode: Some("750000000001".into()),
                    unit: "service".into(),
                    cost_minor: 0,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates product");
        assert_eq!(product.variants.len(), 1);
        let variant = &product.variants[0];
        assert_eq!(variant.sku, "MEM-MEN");
        assert_eq!(
            variant
                .current_price
                .as_ref()
                .map(|price| price.amount_minor),
            Some(10_000)
        );
        let original_price_id = variant
            .current_price
            .as_ref()
            .expect("initial price")
            .id
            .clone();

        db.update_catalog_product(
            &organization_id,
            &product.id,
            NewCatalogProductUpdate {
                name: "Membresía mensual premium".into(),
                category: Some("Membresías".into()),
                status: "active".into(),
            },
        )
        .expect("renames product");
        let after_rename = db
            .catalog_product(&organization_id, &product.id)
            .expect("reads product");
        assert_eq!(after_rename.name, "Membresía mensual premium");
        assert_eq!(
            after_rename.variants[0]
                .current_price
                .as_ref()
                .map(|price| price.amount_minor),
            Some(10_000)
        );

        db.add_catalog_price(
            &organization_id,
            &owner.id,
            &variant.id,
            NewCatalogPrice {
                amount_minor: 12_000,
                tax_rate_bps: 1_600,
                price_includes_tax: true,
                effective_from: Some("2020-06-01T02:00:00+02:00".into()),
            },
        )
        .expect("adds a new price version");
        let current = db
            .catalog_product(&organization_id, &product.id)
            .expect("reads product");
        let current_price = current.variants[0]
            .current_price
            .as_ref()
            .expect("current price");
        assert_eq!(current_price.amount_minor, 12_000);
        assert_ne!(current_price.id, original_price_id);
        let stored_effective: String = db
            .with_connection(|conn| {
                conn.query_row(
                    "SELECT effective_from FROM catalog_price_versions
                     WHERE amount_minor = 12000",
                    [],
                    |row| row.get(0),
                )
                .map_err(DataError::from)
            })
            .expect("reads normalized effective date");
        assert_eq!(stored_effective, "2020-06-01T00:00:00Z");

        db.add_catalog_price(
            &organization_id,
            &owner.id,
            &variant.id,
            NewCatalogPrice {
                amount_minor: 13_000,
                tax_rate_bps: 1_600,
                price_includes_tax: true,
                effective_from: Some("2999-01-01T00:00:00Z".into()),
            },
        )
        .expect("adds a future price version");
        let future = db
            .catalog_product(&organization_id, &product.id)
            .expect("reads product");
        assert_eq!(
            future.variants[0]
                .current_price
                .as_ref()
                .map(|price| price.amount_minor),
            Some(12_000),
            "a future price must not become current yet"
        );

        let version_count: i64 = db
            .with_connection(|conn| {
                conn.query_row("SELECT count(*) FROM catalog_price_versions", [], |row| {
                    row.get(0)
                })
                .map_err(DataError::from)
            })
            .expect("counts versions");
        assert_eq!(version_count, 3);

        let update = db.with_connection(|conn| {
            conn.execute("UPDATE catalog_price_versions SET amount_minor = 1", [])
                .map_err(DataError::from)
        });
        assert!(update.is_err(), "price versions must be immutable");
        let delete = db.with_connection(|conn| {
            conn.execute("DELETE FROM catalog_price_versions", [])
                .map_err(DataError::from)
        });
        assert!(delete.is_err(), "price versions must not be deleted");
    }

    #[test]
    fn catalog_rejects_duplicate_identifiers_and_archived_products() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        let item = || NewCatalogItem {
            name: "Agua".into(),
            category: Some("Bebidas".into()),
            variant_name: "600 ml".into(),
            sku: "AGUA-600".into(),
            barcode: Some("750000000002".into()),
            unit: "piece".into(),
            cost_minor: 500,
            price: NewCatalogPrice {
                amount_minor: 1_500,
                tax_rate_bps: 1_600,
                price_includes_tax: true,
                effective_from: Some("2020-01-01T00:00:00Z".into()),
            },
        };
        let product = db
            .create_catalog_item(&organization_id, &owner.id, item())
            .expect("creates product");

        let mut duplicate_sku = item();
        duplicate_sku.name = "Agua mineral".into();
        duplicate_sku.barcode = None;
        assert!(matches!(
            db.create_catalog_item(&organization_id, &owner.id, duplicate_sku)
                .expect_err("rejects duplicate SKU"),
            DataError::CatalogSkuAlreadyExists
        ));

        let mut duplicate_barcode = item();
        duplicate_barcode.name = "Agua gasificada".into();
        duplicate_barcode.sku = "AGUA-600-GAS".into();
        assert!(matches!(
            db.create_catalog_item(&organization_id, &owner.id, duplicate_barcode)
                .expect_err("rejects duplicate barcode"),
            DataError::CatalogBarcodeAlreadyExists
        ));

        db.update_catalog_product(
            &organization_id,
            &product.id,
            NewCatalogProductUpdate {
                name: product.name.clone(),
                category: product.category.clone(),
                status: "archived".into(),
            },
        )
        .expect("archives product");
        assert!(matches!(
            db.add_catalog_variant(
                &organization_id,
                &owner.id,
                &product.id,
                NewCatalogVariant {
                    name: "1 L".into(),
                    sku: "AGUA-1L".into(),
                    barcode: None,
                    unit: "piece".into(),
                    cost_minor: 800,
                    price: NewCatalogPrice {
                        amount_minor: 2_500,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: None,
                    },
                }
            )
            .expect_err("archived product rejects variants"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.add_catalog_price(
                &organization_id,
                &owner.id,
                &product.variants[0].id,
                NewCatalogPrice {
                    amount_minor: 1_800,
                    tax_rate_bps: 1_600,
                    price_includes_tax: true,
                    effective_from: None,
                },
            )
            .expect_err("archived product rejects prices"),
            DataError::InvalidValue(_)
        ));
    }

    #[test]
    fn audit_chain_is_append_only_and_tamper_evident() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");

        db.record_audit_event(
            &organization_id,
            Some("user-1"),
            "auth.login",
            "session",
            "allowed",
            "tauri",
            None,
        )
        .expect("records login");
        db.record_audit_event(
            &organization_id,
            Some("user-1"),
            "staff.updated",
            "staff_membership",
            "allowed",
            "tauri",
            Some("corr-1"),
        )
        .expect("records update");
        db.record_audit_event(
            &organization_id,
            None,
            "auth.login",
            "session",
            "failed",
            "tauri",
            None,
        )
        .expect("records failure");

        let events = db
            .list_audit_events(&organization_id, 10)
            .expect("lists audit");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence, 3);
        assert_eq!(events[2].sequence, 1);
        assert_eq!(events[1].correlation_id.as_deref(), Some("corr-1"));

        let integrity = db
            .verify_audit_chain(&organization_id)
            .expect("verifies chain");
        assert!(integrity.valid);
        assert_eq!(integrity.checked_events, 3);

        let update = db.with_connection(|conn| {
            conn.execute("UPDATE audit_events SET result = 'denied'", [])
                .map_err(DataError::from)
        });
        assert!(update.is_err(), "updates must be rejected");
        let delete = db.with_connection(|conn| {
            conn.execute("DELETE FROM audit_events", [])
                .map_err(DataError::from)
        });
        assert!(delete.is_err(), "deletes must be rejected");

        db.with_connection(|conn| {
            conn.execute("DROP TRIGGER audit_events_no_update", [])?;
            conn.execute(
                "UPDATE audit_events SET result = 'denied' WHERE sequence = 2",
                [],
            )?;
            Ok(())
        })
        .expect("simulates local tampering");
        let tampered = db
            .verify_audit_chain(&organization_id)
            .expect("verifies tampered chain");
        assert!(!tampered.valid);
        assert_eq!(tampered.first_broken_sequence, Some(2));
    }

    #[test]
    fn module_events_are_idempotent_and_listed_once() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        assert_eq!(db.organization().expect("organization query"), None);
        db.create_organization("Gimnasio Centro")
            .expect("creates organization");
        let organization = db
            .organization()
            .expect("organization query")
            .expect("organization exists");

        let first = db
            .record_module_event(
                &organization.id,
                "outbound",
                "access.allowed",
                "access_decision",
                "req-1",
                "hash-a",
            )
            .expect("records first event");
        assert!(first.inserted);

        let replay = db
            .record_module_event(
                &organization.id,
                "outbound",
                "access.allowed",
                "access_decision",
                "req-1",
                "hash-a",
            )
            .expect("replays event");
        assert!(!replay.inserted);
        assert_eq!(replay.event.id, first.event.id);

        let conflict = db
            .record_module_event(
                &organization.id,
                "outbound",
                "access.denied",
                "access_decision",
                "req-1",
                "hash-b",
            )
            .expect("returns existing event");
        assert!(!conflict.inserted);
        assert_eq!(conflict.event.payload_hash, "hash-a");

        let events = db
            .list_module_events("access_decision", 20)
            .expect("lists events");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn financial_report_reconciles_ledger_payments_products_and_cash() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        let tracked = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Playera".into(),
                    category: None,
                    variant_name: "M".into(),
                    sku: "PLAY-M".into(),
                    barcode: None,
                    unit: "piece".into(),
                    cost_minor: 4_000,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates tracked product");
        let service = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Visita".into(),
                    category: None,
                    variant_name: "Día".into(),
                    sku: "VIS-DIA".into(),
                    barcode: None,
                    unit: "service".into(),
                    cost_minor: 0,
                    price: NewCatalogPrice {
                        amount_minor: 5_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates service product");
        let tracked_variant = tracked.variants[0].id.clone();
        db.record_inventory_entry(
            &organization_id,
            &owner.id,
            NewInventoryEntry {
                variant_id: tracked_variant.clone(),
                quantity: 5,
                unit_cost_minor: 4_000,
                reference: None,
                note: None,
            },
        )
        .expect("records stock");
        let cash_session = db
            .open_cash_session(&organization_id, &owner.id, 5_000)
            .expect("opens session");

        let first_sale = db
            .create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![
                        NewSaleItem {
                            variant_id: tracked_variant.clone(),
                            quantity: 2,
                        },
                        NewSaleItem {
                            variant_id: service.variants[0].id.clone(),
                            quantity: 1,
                        },
                    ],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(25_000),
                    payment_reference: Some("CASH-1".into()),
                    idempotency_key: "report-sale-1".into(),
                },
            )
            .expect("creates cash sale");
        let tracked_sale_item = first_sale
            .items
            .iter()
            .find(|item| item.variant_id == tracked_variant)
            .expect("tracked item")
            .id
            .clone();
        db.create_sale_return(
            &organization_id,
            &owner.id,
            NewSaleReturn {
                sale_id: first_sale.id.clone(),
                reason: "Talla incorrecta".into(),
                items: vec![NewSaleReturnItem {
                    sale_item_id: tracked_sale_item,
                    quantity: 1,
                }],
                payment_reference: Some("REFUND-1".into()),
                idempotency_key: "report-return-1".into(),
            },
        )
        .expect("returns one shirt");
        db.create_sale(
            &organization_id,
            &owner.id,
            NewSale {
                items: vec![NewSaleItem {
                    variant_id: tracked_variant.clone(),
                    quantity: 1,
                }],
                payment_method: "transfer".into(),
                cash_received_minor: None,
                payment_reference: Some("SPEI-1".into()),
                idempotency_key: "report-sale-2".into(),
            },
        )
        .expect("creates transfer sale");
        db.adjust_inventory(
            &organization_id,
            &owner.id,
            NewInventoryAdjustment {
                variant_id: tracked_variant.clone(),
                quantity_delta: -1,
                unit_cost_minor: None,
                reason: "Producto dañado".into(),
            },
        )
        .expect("adjusts stock");
        db.close_cash_session(
            &organization_id,
            &cash_session.id,
            &owner.id,
            19_000,
            Some("Faltante"),
        )
        .expect("closes session");

        let from = (chrono::Utc::now() - chrono::TimeDelta::days(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let to = (chrono::Utc::now() + chrono::TimeDelta::days(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let report = db
            .financial_report(&organization_id, &from, &to)
            .expect("builds report");

        assert_eq!(report.sales_count, 2);
        assert_eq!(report.gross_sales_minor, 35_000);
        assert_eq!(report.refunds_count, 1);
        assert_eq!(report.refunded_minor, 10_000);
        assert_eq!(report.net_sales_minor, 25_000);
        assert_eq!(report.average_sale_minor, 17_500);
        assert_eq!(report.attach_rate_bps, 5_000);
        assert_eq!(report.items_per_sale_milli, 2_000);
        assert_eq!(report.cost_of_goods_minor, 8_000);
        assert_eq!(report.margin_minor, 17_000);
        assert_eq!(report.margin_bps, 6_800);
        assert_eq!(report.inventory_value_start_minor, 0);
        assert_eq!(report.inventory_value_end_minor, 8_000);
        assert_eq!(report.inventory_turnover_bps, 20_000);
        assert_eq!(report.adjustments_units, -1);
        assert_eq!(report.adjustments_cost_minor, -4_000);

        let payment_sales: i64 = report
            .payment_methods
            .iter()
            .map(|method| method.gross_sales_minor)
            .sum();
        let payment_refunds: i64 = report
            .payment_methods
            .iter()
            .map(|method| method.refunded_minor)
            .sum();
        assert_eq!(payment_sales, report.gross_sales_minor);
        assert_eq!(payment_refunds, report.refunded_minor);
        assert_eq!(
            report
                .payment_methods
                .iter()
                .map(|method| method.sales_count)
                .sum::<i64>(),
            report.sales_count
        );

        let cash = report
            .payment_methods
            .iter()
            .find(|method| method.payment_method == "cash")
            .expect("cash breakdown");
        assert_eq!(cash.sales_count, 1);
        assert_eq!(cash.gross_sales_minor, 25_000);
        assert_eq!(cash.refunded_minor, 10_000);
        assert_eq!(cash.net_sales_minor, 15_000);
        let transfer = report
            .payment_methods
            .iter()
            .find(|method| method.payment_method == "transfer")
            .expect("transfer breakdown");
        assert_eq!(transfer.sales_count, 1);
        assert_eq!(transfer.net_sales_minor, 10_000);
        let point = report
            .payment_methods
            .iter()
            .find(|method| method.payment_method == "point")
            .expect("point breakdown");
        assert_eq!(point.sales_count, 0);
        assert_eq!(point.net_sales_minor, 0);

        assert_eq!(report.cash_sessions_count, 1);
        assert_eq!(report.open_cash_sessions, 0);
        assert_eq!(report.closed_cash_sessions, 1);
        assert_eq!(report.cash_difference_minor, -1_000);
        assert_eq!(report.cash_difference_abs_minor, 1_000);

        let product_gross: i64 = report
            .product_sales
            .iter()
            .map(|product| product.gross_sales_minor)
            .sum();
        let product_refunds: i64 = report
            .product_sales
            .iter()
            .map(|product| product.refunded_minor)
            .sum();
        assert_eq!(product_gross, report.gross_sales_minor);
        assert_eq!(product_refunds, report.refunded_minor);

        let shirt = report
            .product_sales
            .iter()
            .find(|product| product.variant_id == tracked_variant)
            .expect("shirt totals");
        assert_eq!(shirt.sold_units, 3);
        assert_eq!(shirt.refunded_units, 1);
        assert_eq!(shirt.net_units, 2);
        assert_eq!(shirt.gross_sales_minor, 30_000);
        assert_eq!(shirt.refunded_minor, 10_000);
        assert_eq!(shirt.net_sales_minor, 20_000);
        assert_eq!(shirt.cost_minor, 8_000);
        assert_eq!(shirt.margin_minor, 12_000);

        let visit = report
            .product_sales
            .iter()
            .find(|product| product.variant_id == service.variants[0].id)
            .expect("service totals");
        assert_eq!(visit.net_sales_minor, 5_000);
        assert_eq!(visit.cost_minor, 0);
        assert_eq!(visit.margin_minor, 5_000);

        assert_eq!(report.inventory_adjustments.len(), 1);
        let adjustment = &report.inventory_adjustments[0];
        assert_eq!(adjustment.variant_id, tracked_variant);
        assert_eq!(adjustment.units_delta, -1);
        assert_eq!(adjustment.cost_minor, -4_000);

        let future = db
            .financial_report(
                &organization_id,
                "2999-01-01T00:00:00Z",
                "2999-01-02T00:00:00Z",
            )
            .expect("empty report");
        assert_eq!(future.sales_count, 0);
        assert_eq!(future.net_sales_minor, 0);
        assert_eq!(future.attach_rate_bps, 0);
        assert_eq!(future.items_per_sale_milli, 0);
        assert_eq!(future.cost_of_goods_minor, 0);
        assert_eq!(future.margin_minor, 0);
        assert_eq!(future.margin_bps, 0);
        assert_eq!(future.inventory_value_start_minor, 8_000);
        assert_eq!(future.inventory_value_end_minor, 8_000);
        assert_eq!(future.inventory_turnover_bps, 0);
        assert_eq!(future.adjustments_units, 0);
        assert_eq!(future.adjustments_cost_minor, 0);
        assert_eq!(future.product_sales.len(), 0);
        assert_eq!(future.payment_methods.len(), 3);

        assert!(matches!(
            db.financial_report(&organization_id, &from, &from)
                .expect_err("empty range"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.financial_report(
                &organization_id,
                "2020-01-01T00:00:00Z",
                "2022-01-01T00:00:00Z"
            )
            .expect_err("range too wide"),
            DataError::InvalidValue(_)
        ));
    }

    #[test]
    #[ignore = "latency smoke; run with --ignored --nocapture"]
    fn access_event_latency_p95_is_under_300ms() {
        use std::time::{Duration, Instant};

        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        db.create_organization("Gimnasio Centro")
            .expect("creates organization");
        let organization = db
            .organization()
            .expect("organization query")
            .expect("organization exists");

        let mut samples = Vec::with_capacity(100);
        for index in 0..100 {
            let started = Instant::now();
            db.record_module_event(
                &organization.id,
                "outbound",
                "access.allowed",
                "access_decision",
                &format!("req-{index}"),
                "hash",
            )
            .expect("records event");
            db.list_module_events("access_decision", 20)
                .expect("refreshes event list");
            samples.push(started.elapsed());
        }
        samples.sort_unstable();
        let p95 = samples[94];
        println!("access p95 = {p95:?}");
        assert!(p95 < Duration::from_millis(300));
    }

    #[test]
    fn cash_sessions_track_sales_and_close_once() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        let product = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Membresía mensual".into(),
                    category: None,
                    variant_name: "Mensual".into(),
                    sku: "MEM-MEN".into(),
                    barcode: None,
                    unit: "service".into(),
                    cost_minor: 0,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates product");
        let variant_id = product.variants[0].id.clone();

        assert!(db
            .current_cash_session(&organization_id)
            .expect("queries current session")
            .is_none());
        let opened = db
            .open_cash_session(&organization_id, &owner.id, 5_000)
            .expect("opens session");
        assert_eq!(opened.opening_amount_minor, 5_000);
        assert_eq!(opened.expected_cash_minor, 5_000);
        assert_eq!(opened.sales_count, 0);
        assert_eq!(opened.status, "open");
        assert!(matches!(
            db.open_cash_session(&organization_id, &owner.id, 1_000)
                .expect_err("second open session"),
            DataError::CashSessionAlreadyOpen
        ));

        db.create_sale(
            &organization_id,
            &owner.id,
            NewSale {
                items: vec![NewSaleItem {
                    variant_id: variant_id.clone(),
                    quantity: 1,
                }],
                payment_method: "cash".into(),
                cash_received_minor: Some(10_000),
                payment_reference: None,
                idempotency_key: "cash-session-sale".into(),
            },
        )
        .expect("creates cash sale");
        db.create_sale(
            &organization_id,
            &owner.id,
            NewSale {
                items: vec![NewSaleItem {
                    variant_id,
                    quantity: 2,
                }],
                payment_method: "transfer".into(),
                cash_received_minor: None,
                payment_reference: None,
                idempotency_key: "cash-session-transfer".into(),
            },
        )
        .expect("creates transfer sale");

        let current = db
            .current_cash_session(&organization_id)
            .expect("queries current session")
            .expect("session is open");
        assert_eq!(current.cash_sales_minor, 10_000);
        assert_eq!(current.expected_cash_minor, 15_000);
        assert_eq!(current.sales_count, 2);
        assert_eq!(current.movements_count, 0);

        let income = db
            .record_cash_movement(
                &organization_id,
                &owner.id,
                NewCashMovement {
                    cash_session_id: current.id.clone(),
                    movement_type: "income".into(),
                    category: "Ajuste de caja".into(),
                    amount_minor: 2_000,
                    payment_method: "cash".into(),
                    evidence: "Recibo A-100".into(),
                },
            )
            .expect("records cash income");
        assert_eq!(income.category, "Ajuste de caja");
        assert_eq!(income.authorized_by_name.as_deref(), Some("Ana Pérez"));
        db.record_cash_movement(
            &organization_id,
            &owner.id,
            NewCashMovement {
                cash_session_id: current.id.clone(),
                movement_type: "expense".into(),
                category: "Limpieza".into(),
                amount_minor: 500,
                payment_method: "cash".into(),
                evidence: "Nota 42".into(),
            },
        )
        .expect("records cash expense");
        db.record_cash_movement(
            &organization_id,
            &owner.id,
            NewCashMovement {
                cash_session_id: current.id.clone(),
                movement_type: "income".into(),
                category: "Transferencia".into(),
                amount_minor: 1_000,
                payment_method: "transfer".into(),
                evidence: "SPEI 123".into(),
            },
        )
        .expect("records transfer income");

        let with_movements = db
            .current_cash_session(&organization_id)
            .expect("queries current session")
            .expect("session is open");
        assert_eq!(with_movements.cash_income_minor, 2_000);
        assert_eq!(with_movements.income_minor, 3_000);
        assert_eq!(with_movements.cash_expense_minor, 500);
        assert_eq!(with_movements.expense_minor, 500);
        assert_eq!(with_movements.expected_cash_minor, 16_500);
        assert_eq!(with_movements.movements_count, 3);
        assert_eq!(
            db.list_cash_movements(&organization_id, &current.id, 50)
                .expect("lists movements")
                .len(),
            3
        );
        assert!(matches!(
            db.close_cash_session(&organization_id, &current.id, &owner.id, 16_000, None)
                .expect_err("difference without reason"),
            DataError::InvalidValue(_)
        ));

        let closed = db
            .close_cash_session(
                &organization_id,
                &current.id,
                &owner.id,
                16_000,
                Some("Faltante de cambio"),
            )
            .expect("closes session");
        assert_eq!(closed.status, "closed");
        assert_eq!(closed.counted_amount_minor, Some(16_000));
        assert_eq!(closed.difference_minor, Some(-500));
        assert_eq!(
            closed.difference_reason.as_deref(),
            Some("Faltante de cambio")
        );
        assert_eq!(closed.closed_by_name.as_deref(), Some("Ana Pérez"));
        assert!(closed.closed_at.is_some());
        assert!(db
            .current_cash_session(&organization_id)
            .expect("queries current session")
            .is_none());
        assert!(matches!(
            db.close_cash_session(&organization_id, &current.id, &owner.id, 16_000, None)
                .expect_err("closing twice"),
            DataError::CashSessionNotOpen
        ));
        assert!(matches!(
            db.close_cash_session(&organization_id, "missing", &owner.id, 16_000, None)
                .expect_err("missing session"),
            DataError::CashSessionNotFound
        ));
        assert!(matches!(
            db.record_cash_movement(
                &organization_id,
                &owner.id,
                NewCashMovement {
                    cash_session_id: current.id.clone(),
                    movement_type: "income".into(),
                    category: "Tarde".into(),
                    amount_minor: 100,
                    payment_method: "cash".into(),
                    evidence: "Nota".into(),
                },
            )
            .expect_err("movement on closed session"),
            DataError::CashSessionNotOpen
        ));
        assert!(matches!(
            db.with_connection(|conn| {
                conn.execute("DELETE FROM cash_movements", [])?;
                Ok(())
            })
            .expect_err("cash movements cannot be deleted"),
            DataError::Sqlite(_)
        ));
        assert!(matches!(
            db.with_connection(|conn| {
                conn.execute("UPDATE cash_movements SET amount_minor = 0", [])?;
                Ok(())
            })
            .expect_err("cash movements are immutable"),
            DataError::Sqlite(_)
        ));
        assert!(matches!(
            db.create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: product.variants[0].id.clone(),
                        quantity: 1,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: None,
                    payment_reference: None,
                    idempotency_key: "sale-without-session".into(),
                },
            )
            .expect_err("sale without an open session"),
            DataError::CashSessionRequired
        ));
        assert!(matches!(
            db.with_connection(|conn| {
                conn.execute("DELETE FROM cash_sessions", [])?;
                Ok(())
            })
            .expect_err("cash sessions cannot be deleted"),
            DataError::Sqlite(_)
        ));

        db.open_cash_session(&organization_id, &owner.id, 0)
            .expect("opens a new session after closing");
        assert_eq!(
            db.list_cash_sessions(&organization_id, 50)
                .expect("lists sessions")
                .len(),
            2
        );
    }

    #[test]
    fn sale_returns_are_referenced_append_only_and_rebuild_cash() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        let product = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Membresía mensual".into(),
                    category: None,
                    variant_name: "Mensual".into(),
                    sku: "MEM-MEN".into(),
                    barcode: None,
                    unit: "service".into(),
                    cost_minor: 0,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates product");
        let variant_id = product.variants[0].id.clone();
        db.open_cash_session(&organization_id, &owner.id, 5_000)
            .expect("opens session");
        let sale = db
            .create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id,
                        quantity: 3,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(30_000),
                    payment_reference: None,
                    idempotency_key: "sale-return".into(),
                },
            )
            .expect("creates sale");
        let sale_item_id = sale.items[0].id.clone();
        assert_eq!(sale.total_minor, 30_000);
        assert_eq!(sale.refunded_minor, 0);
        assert_eq!(sale.items[0].refunded_quantity, 0);

        let first = db
            .create_sale_return(
                &organization_id,
                &owner.id,
                NewSaleReturn {
                    sale_id: sale.id.clone(),
                    reason: "Cliente canceló una sesión".into(),
                    items: vec![NewSaleReturnItem {
                        sale_item_id: sale_item_id.clone(),
                        quantity: 1,
                    }],
                    payment_reference: Some("REF-REEMBOLSO-1".into()),
                    idempotency_key: "partial-return".into(),
                },
            )
            .expect("creates partial return");
        assert_eq!(first.total_minor, 10_000);
        assert_eq!(first.refund_method, "cash");
        assert_eq!(first.payment_reference.as_deref(), Some("REF-REEMBOLSO-1"));
        assert_eq!(first.items.len(), 1);
        assert_eq!(first.items[0].quantity, 1);

        let refreshed = db
            .sale(&organization_id, &sale.id)
            .expect("loads updated sale");
        assert_eq!(refreshed.refunded_minor, 10_000);
        assert_eq!(refreshed.items[0].refunded_quantity, 1);
        assert_eq!(refreshed.items[0].refunded_minor, 10_000);

        let session = db
            .current_cash_session(&organization_id)
            .expect("queries session")
            .expect("session is open");
        assert_eq!(session.cash_refunds_minor, 10_000);
        assert_eq!(session.refunded_minor, 10_000);
        assert_eq!(session.refunds_count, 1);
        assert_eq!(session.expected_cash_minor, 25_000);

        assert!(matches!(
            db.create_sale_return(
                &organization_id,
                &owner.id,
                NewSaleReturn {
                    sale_id: sale.id.clone(),
                    reason: " ".into(),
                    items: vec![NewSaleReturnItem {
                        sale_item_id: sale_item_id.clone(),
                        quantity: 1,
                    }],
                    payment_reference: None,
                    idempotency_key: "return-invalid-reason".into(),
                },
            )
            .expect_err("reason is required"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.create_sale_return(
                &organization_id,
                &owner.id,
                NewSaleReturn {
                    sale_id: sale.id.clone(),
                    reason: "Cobro duplicado".into(),
                    items: vec![NewSaleReturnItem {
                        sale_item_id: sale_item_id.clone(),
                        quantity: 3,
                    }],
                    payment_reference: None,
                    idempotency_key: "return-invalid-quantity".into(),
                },
            )
            .expect_err("quantity exceeds remaining"),
            DataError::InvalidValue(_)
        ));

        let second = db
            .create_sale_return(
                &organization_id,
                &owner.id,
                NewSaleReturn {
                    sale_id: sale.id.clone(),
                    reason: "Cancelación total".into(),
                    items: vec![NewSaleReturnItem {
                        sale_item_id: sale_item_id.clone(),
                        quantity: 2,
                    }],
                    payment_reference: None,
                    idempotency_key: "return-remainder".into(),
                },
            )
            .expect("returns the remainder");
        assert_eq!(second.total_minor, 20_000);

        let fully_returned = db
            .sale(&organization_id, &sale.id)
            .expect("loads fully returned sale");
        assert_eq!(fully_returned.refunded_minor, 30_000);
        assert_eq!(fully_returned.items[0].refunded_quantity, 3);
        assert_eq!(
            db.list_sale_returns(&organization_id, Some(&sale.id), 50)
                .expect("lists returns")
                .len(),
            2
        );
        let charges = db
            .list_payment_charges(&organization_id, Some(&sale.id), None, 50)
            .expect("lists sale payment charges");
        assert_eq!(charges.len(), 3);
        assert_eq!(
            charges
                .iter()
                .filter(|charge| charge.kind == "charge")
                .count(),
            1
        );
        assert_eq!(
            charges
                .iter()
                .filter(|charge| charge.kind == "refund")
                .count(),
            2
        );
        let replay = db
            .create_sale_return(
                &organization_id,
                &owner.id,
                NewSaleReturn {
                    sale_id: sale.id.clone(),
                    reason: "Cliente canceló una sesión".into(),
                    items: vec![NewSaleReturnItem {
                        sale_item_id: sale_item_id.clone(),
                        quantity: 1,
                    }],
                    payment_reference: Some("REF-REEMBOLSO-1".into()),
                    idempotency_key: "partial-return".into(),
                },
            )
            .expect("replays the original return");
        assert_eq!(replay.id, first.id);
        assert!(matches!(
            db.create_sale_return(
                &organization_id,
                &owner.id,
                NewSaleReturn {
                    sale_id: sale.id.clone(),
                    reason: "Otro motivo".into(),
                    items: vec![NewSaleReturnItem {
                        sale_item_id: sale_item_id.clone(),
                        quantity: 1,
                    }],
                    payment_reference: Some("REF-REEMBOLSO-1".into()),
                    idempotency_key: "partial-return".into(),
                },
            )
            .expect_err("idempotency key reused with another return"),
            DataError::SaleIdempotencyConflict
        ));

        let session = db
            .current_cash_session(&organization_id)
            .expect("queries session")
            .expect("session is open");
        assert_eq!(session.refunds_count, 2);
        assert_eq!(session.expected_cash_minor, 5_000);

        let ledger_total: i64 = db
            .with_connection(|conn| {
                conn.query_row(
                    "SELECT COALESCE(SUM(amount_minor), 0) FROM ledger_entries
                     WHERE organization_id = ?1",
                    params![organization_id],
                    |row| row.get(0),
                )
                .map_err(DataError::from)
            })
            .expect("reconstructs ledger total");
        assert_eq!(ledger_total, 0);

        for statement in [
            "UPDATE sale_returns SET total_minor = 0",
            "DELETE FROM sale_returns",
            "UPDATE sale_return_items SET quantity = 0",
            "DELETE FROM sale_return_items",
            "UPDATE ledger_entries SET amount_minor = 0",
            "DELETE FROM ledger_entries",
            "UPDATE payment_charges SET amount_minor = 0",
            "DELETE FROM payment_charges",
        ] {
            assert!(
                db.with_connection(|conn| {
                    conn.execute(statement, [])?;
                    Ok(())
                })
                .is_err(),
                "{statement} must be rejected"
            );
        }
    }

    #[test]
    fn inventory_movements_drive_stock_value() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        let product = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Playera".into(),
                    category: None,
                    variant_name: "M".into(),
                    sku: "PLAY-M".into(),
                    barcode: None,
                    unit: "piece".into(),
                    cost_minor: 4_000,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates product");
        let variant_id = product.variants[0].id.clone();
        let initial = db
            .list_inventory(&organization_id)
            .expect("lists inventory");
        assert_eq!(initial.len(), 1);
        assert_eq!(initial[0].stock, 0);
        assert_eq!(initial[0].value_minor, 0);
        assert!(initial[0].low_stock);

        db.record_inventory_entry(
            &organization_id,
            &owner.id,
            NewInventoryEntry {
                variant_id: variant_id.clone(),
                quantity: 10,
                unit_cost_minor: 4_000,
                reference: Some("OC-1".into()),
                note: Some("Compra inicial".into()),
            },
        )
        .expect("records entry");

        let item = db
            .list_inventory(&organization_id)
            .expect("lists inventory")
            .remove(0);
        assert_eq!(item.stock, 10);
        assert_eq!(item.value_minor, 40_000);
        assert_eq!(item.avg_unit_cost_minor, 4_000);
        assert!(!item.low_stock);

        let item = db
            .update_inventory_threshold(&organization_id, &owner.id, &variant_id, 5)
            .expect("sets threshold");
        assert_eq!(item.min_stock, 5);

        db.open_cash_session(&organization_id, &owner.id, 0)
            .expect("opens session");
        let sale = db
            .create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: variant_id.clone(),
                        quantity: 2,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(20_000),
                    payment_reference: None,
                    idempotency_key: "inventory-sale".into(),
                },
            )
            .expect("creates sale");
        assert_eq!(
            db.list_inventory(&organization_id)
                .expect("lists inventory")[0]
                .stock,
            8
        );

        db.create_sale_return(
            &organization_id,
            &owner.id,
            NewSaleReturn {
                sale_id: sale.id,
                reason: "Talla incorrecta".into(),
                items: vec![NewSaleReturnItem {
                    sale_item_id: sale.items[0].id.clone(),
                    quantity: 1,
                }],
                payment_reference: None,
                idempotency_key: "inventory-return".into(),
            },
        )
        .expect("records return");

        db.adjust_inventory(
            &organization_id,
            &owner.id,
            NewInventoryAdjustment {
                variant_id: variant_id.clone(),
                quantity_delta: -1,
                unit_cost_minor: None,
                reason: "Producto dañado".into(),
            },
        )
        .expect("records adjustment");

        let item = db
            .list_inventory(&organization_id)
            .expect("lists inventory")
            .remove(0);
        assert_eq!(item.stock, 8);
        assert_eq!(item.value_minor, 32_000);

        let movements = db
            .list_inventory_movements(&organization_id, Some(&variant_id), 50)
            .expect("lists movements");
        assert_eq!(movements.len(), 4);
        assert!(movements
            .iter()
            .any(|movement| movement.movement_type == "sale" && movement.quantity_delta == -2));
        assert!(movements
            .iter()
            .any(|movement| movement.movement_type == "return" && movement.quantity_delta == 1));

        for statement in [
            "UPDATE inventory_movements SET quantity_delta = 0",
            "DELETE FROM inventory_movements",
        ] {
            assert!(
                db.with_connection(|conn| {
                    conn.execute(statement, [])?;
                    Ok(())
                })
                .is_err(),
                "{statement} must be rejected"
            );
        }
    }

    #[test]
    fn sales_snapshot_prices_and_reject_invalid_payments() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner", None)
            .expect("creates owner");
        db.open_cash_session(&organization_id, &owner.id, 5_000)
            .expect("opens cash session");
        let product = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Membresía mensual".into(),
                    category: None,
                    variant_name: "Mensual".into(),
                    sku: "MEM-MEN".into(),
                    barcode: None,
                    unit: "service".into(),
                    cost_minor: 0,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates product");
        let variant_id = product.variants[0].id.clone();
        let price_id = product.variants[0]
            .current_price
            .as_ref()
            .expect("price")
            .id
            .clone();

        let sale = db
            .create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: variant_id.clone(),
                        quantity: 1,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(20_000),
                    payment_reference: None,
                    idempotency_key: "snapshot-cash".into(),
                },
            )
            .expect("creates sale");
        assert_eq!(sale.subtotal_minor, 8_621);
        assert_eq!(sale.tax_minor, 1_379);
        assert_eq!(sale.total_minor, 10_000);
        assert_eq!(sale.cash_received_minor, Some(20_000));
        assert_eq!(sale.change_minor, Some(10_000));
        assert_eq!(sale.sold_by_name.as_deref(), Some("Ana Pérez"));
        assert!(sale.cash_session_id.is_some());
        assert_eq!(sale.items.len(), 1);
        assert_eq!(sale.items[0].price_version_id, price_id);
        assert_eq!(sale.items[0].sku, "MEM-MEN");

        let transfer = db
            .create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: variant_id.clone(),
                        quantity: 2,
                    }],
                    payment_method: "transfer".into(),
                    cash_received_minor: None,
                    payment_reference: Some("SPEI-123".into()),
                    idempotency_key: "snapshot-transfer".into(),
                },
            )
            .expect("creates transfer sale");
        assert_eq!(transfer.total_minor, 20_000);
        assert_eq!(transfer.cash_received_minor, None);
        assert_eq!(transfer.payment_reference.as_deref(), Some("SPEI-123"));
        assert_eq!(db.list_sales(&organization_id, 50).expect("lists").len(), 2);
        let charges = db
            .list_payment_charges(&organization_id, None, None, 50)
            .expect("lists payment charges");
        assert_eq!(charges.len(), 2);
        let transfer_charge = charges
            .iter()
            .find(|charge| charge.sale_id == transfer.id)
            .expect("transfer charge exists");
        assert_eq!(transfer_charge.kind, "charge");
        assert_eq!(transfer_charge.method, "transfer");
        assert_eq!(transfer_charge.provider, "manual");
        assert_eq!(
            transfer_charge.external_reference.as_deref(),
            Some("SPEI-123")
        );
        assert_eq!(transfer_charge.amount_minor, 20_000);

        let replay = db
            .create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: variant_id.clone(),
                        quantity: 1,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(20_000),
                    payment_reference: None,
                    idempotency_key: "snapshot-cash".into(),
                },
            )
            .expect("replays the original sale");
        assert_eq!(replay.id, sale.id);
        assert_eq!(db.list_sales(&organization_id, 50).expect("lists").len(), 2);

        assert!(matches!(
            db.create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: variant_id.clone(),
                        quantity: 2,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(20_000),
                    payment_reference: None,
                    idempotency_key: "snapshot-cash".into(),
                },
            )
            .expect_err("idempotency key reused with different contents"),
            DataError::SaleIdempotencyConflict
        ));

        assert!(matches!(
            db.create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![NewSaleItem {
                        variant_id: variant_id.clone(),
                        quantity: 1,
                    }],
                    payment_method: "cash".into(),
                    cash_received_minor: Some(1),
                    payment_reference: None,
                    idempotency_key: "underpaid-sale".into(),
                },
            )
            .expect_err("cash below total"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.create_sale(
                &organization_id,
                &owner.id,
                NewSale {
                    items: vec![
                        NewSaleItem {
                            variant_id: variant_id.clone(),
                            quantity: 1,
                        },
                        NewSaleItem {
                            variant_id: variant_id.clone(),
                            quantity: 1,
                        },
                    ],
                    payment_method: "cash".into(),
                    cash_received_minor: None,
                    payment_reference: None,
                    idempotency_key: "duplicate-variant".into(),
                },
            )
            .expect_err("duplicate variant"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.with_connection(|conn| {
                conn.execute("UPDATE sales SET total_minor = 0", [])?;
                Ok(())
            })
            .expect_err("sales are append-only"),
            DataError::Sqlite(_)
        ));
    }

    #[test]
    fn wrong_key_is_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("gym.db");
        let key = DbKey::generate();
        drop(Db::open(&path, &key).expect("opens"));

        let error = Db::open(&path, &DbKey::generate()).expect_err("must fail");
        assert!(matches!(error, DataError::InvalidKey(_)));
    }
}
