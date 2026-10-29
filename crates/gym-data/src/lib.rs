//! Local persistence: encrypted SQLite (SQLCipher), migrations and repos.
//!
//! The encryption key never lives inside the database. `FileKeyProvider` is a
//! bootstrap custodian with `0600` permissions; the OS keychain/Stronghold
//! adapter plugs into `KeyProvider` without touching repositories.

mod db;
mod error;
mod key;
mod migrations;

pub use db::{
    verify_encrypted_database, AccessConflictRecord, AccessEventRecord, AccessEventWriteOutcome,
    AccessReconciliationRecord, AccessSnapshotRecord, AttendanceDayRecord, AttendanceHourRecord,
    AttendanceMemberRecord, AttendancePlanRecord, AttendanceReportRecord, AttendanceZoneRecord,
    AuditEventRecord, AuditIntegrityRecord, CampaignRecord, CashMovementRecord, CashSessionRecord,
    CatalogPriceRecord, CatalogProductRecord, CatalogVariantRecord, ClassBookingRecord,
    ClassRecord, CredentialAccessRuleRecord, Db, EquipmentRecord, FinancialReportRecord,
    FunnelStageRecord, GymSettingsRecord, InboundSyncEvent, IntegrationJournalSummary,
    IntegrationLinkRecord, InventoryAdjustmentRecord, InventoryItemRecord, InventoryMovementRecord,
    LocalDatabaseStatus, MaintenanceOrderRecord, MaintenanceOrderUpdate, MarketingFunnelRecord,
    MemberAccountRecord, MemberCredentialRecord, MemberRecord, MembershipChargeRecord,
    MembershipDashboardRecord, MembershipDashboardTrendRecord, MembershipPaymentReceiptRecord,
    MembershipPaymentRecord, MembershipPlanRecord, MembershipRecord, ModuleEventRecord,
    NewAccessEvent, NewCampaign, NewCashMovement, NewCatalogItem, NewCatalogPrice,
    NewCatalogProductUpdate, NewCatalogVariant, NewCatalogVariantUpdate, NewClass, NewEquipment,
    NewIntegrationLink, NewInventoryAdjustment, NewInventoryEntry, NewMaintenanceOrder, NewMember,
    NewMembershipPayment, NewMembershipPlan, NewProspect, NewReportSchedule, NewSale, NewSaleItem,
    NewSaleReturn, NewSaleReturnItem, OrganizationRecord, OwnerRecoveryRecord, PaymentChargeRecord,
    PaymentMethodTotalRecord, PermissionOverrideRecord, ProductSalesRecord, ProspectRecord,
    RecordedModuleEvent, ReportRunRecord, ReportScheduleRecord, SaleItemRecord, SaleRecord,
    SaleReturnItemRecord, SaleReturnRecord, StaffRecord, SyncFailureRecord, SyncInboxRecord,
    SyncOutboxRecord, UpdateClass, UpdateEquipment, UpdateGymSettings, UpdateMember,
    UpdateMembershipPlan, UpdateProspect, UserCredentialRecord, UserRecord,
};
pub use error::DataError;
pub use key::{DbKey, EnvKeyProvider, FileKeyProvider, KeyProvider, KeyringKeyProvider};
pub use migrations::{Migration, MIGRATIONS};

/// Stable hex digest for event payload de-duplication.
pub fn sha256_hex(value: impl AsRef<[u8]>) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(value.as_ref()))
}
