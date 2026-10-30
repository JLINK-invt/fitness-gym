//! Versioned contracts shared by the Rust core and the Tauri frontend.
//!
//! Every payload is serialized in `camelCase` so the TypeScript client in
//! `packages/contracts` stays aligned without an adapter layer.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Version of the local command/event contract. Bump it on breaking changes.
pub const CONTRACT_VERSION: &str = "3.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ModuleHealth {
    pub fn ok(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Ok,
            detail: None,
        }
    }

    pub fn degraded(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub path: String,
    pub encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher_version: Option<String>,
    pub schema_version: i64,
    pub integrity_ok: bool,
    pub organizations: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub service: String,
    pub version: String,
    pub contract_version: String,
    pub status: HealthStatus,
    pub modules: Vec<ModuleHealth>,
    pub database: Option<DatabaseStatus>,
    pub checked_at: String,
}

/// Request sent by the simulated access module.
///
/// `credential_id` is a public badge identifier, not a secret. The real module
/// will replace it with an authenticated, signed envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessRequest {
    pub credential_id: String,
    pub device_id: String,
    pub idempotency_key: String,
}

/// Local decision returned to the access module and rendered by the desktop UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessDecision {
    pub event_id: String,
    pub allowed: bool,
    pub reason: String,
    pub duplicate: bool,
    pub idempotency_key: String,
    pub latency_ms: u64,
    pub decided_at: String,
}

/// Durable event summary used to prove that a decision is visible exactly once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessEventSummary {
    pub event_id: String,
    pub allowed: bool,
    pub idempotency_key: String,
    pub occurred_at: String,
}

/// Signed snapshot published to the access app. `payload_json` is the exact
/// bytes covered by `checksum` and `signature`; consumers must verify both
/// before applying the revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessSnapshot {
    pub revision: i64,
    pub generated_at: String,
    pub member_count: i64,
    pub credential_count: i64,
    pub checksum: String,
    pub signature: String,
    pub payload_json: String,
}

/// Credential exposed to reception. It is a public badge code, never a secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberCredential {
    pub id: String,
    pub member_id: String,
    pub member_number: String,
    pub member_name: String,
    pub code: String,
    pub status: String,
    pub revision: i64,
    pub issued_at: String,
    pub revoked_at: Option<String>,
}

/// Event produced by a reader. The same idempotency key with different content
/// is rejected instead of overwriting the original decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessEventInput {
    pub credential_code: String,
    pub device_id: String,
    pub zone: Option<String>,
    pub allowed: bool,
    pub reason: String,
    pub snapshot_revision: i64,
    pub occurred_at: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessEvent {
    pub id: String,
    pub credential_code: String,
    pub member_id: Option<String>,
    pub member_number: Option<String>,
    pub member_name: Option<String>,
    pub device_id: String,
    pub zone: Option<String>,
    pub allowed: bool,
    pub reason: String,
    pub snapshot_revision: i64,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub reconciliation_status: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessReconciliationInput {
    pub snapshot_revision: i64,
    pub events: Vec<AccessEventInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConflict {
    pub idempotency_key: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessReconciliationSummary {
    pub snapshot_revision: i64,
    pub inserted: i64,
    pub duplicates: i64,
    pub stale: i64,
    pub conflicts: Vec<AccessConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceInput {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceHour {
    pub hour: i64,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceDay {
    pub date: String,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendancePlan {
    pub plan_id: Option<String>,
    pub plan_name: Option<String>,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceZone {
    pub zone: Option<String>,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceMember {
    pub member_id: String,
    pub member_number: String,
    pub member_name: String,
    pub visits: i64,
    pub last_visit_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceReport {
    pub from: String,
    pub to: String,
    pub generated_at: String,
    pub total_visits: i64,
    pub unique_members: i64,
    pub peak_hour: i64,
    pub low_demand_hours: Vec<AttendanceHour>,
    pub hours: Vec<AttendanceHour>,
    pub days: Vec<AttendanceDay>,
    pub plans: Vec<AttendancePlan>,
    pub zones: Vec<AttendanceZone>,
    pub members: Vec<AttendanceMember>,
}

/// Roles enabled for local staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StaffRole {
    Owner,
    Manager,
    Receptionist,
    Advisor,
    Trainer,
}

/// Public bootstrap state used by the login/setup screen. It never exposes
/// password hashes or internal membership identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupState {
    pub configured: bool,
    pub gym_name: Option<String>,
    pub owner_ready: bool,
}

/// Authenticated actor derived from Rust-side session state. The webview may
/// render it but cannot submit it as an authorization claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub user_id: String,
    pub organization_id: String,
    /// Display name of the gym this installation serves. Used as the window
    /// title in the desktop shell.
    pub organization_name: String,
    pub name: String,
    pub role: StaffRole,
    pub permissions: Vec<String>,
}

/// Returned exactly once after creating or recovering the owner account. The
/// plaintext recovery code must be stored outside the app because only its
/// Argon2id hash is persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryKit {
    pub session: SessionSummary,
    pub recovery_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCode {
    pub recovery_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StaffStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionOverride {
    pub permission: String,
    pub effect: PermissionEffect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStaffPermissionsInput {
    pub user_id: String,
    pub overrides: Vec<PermissionOverride>,
}

/// Staff member visible to owner and manager screens. Password hashes are
/// never part of this contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffSummary {
    pub user_id: String,
    pub name: String,
    pub role: StaffRole,
    pub status: StaffStatus,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub permission_overrides: Vec<PermissionOverride>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CatalogStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CatalogUnit {
    Piece,
    Service,
    Session,
    Package,
}

/// Immutable price snapshot. Sales must reference the version id so later
/// catalog edits never rewrite historical totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPrice {
    pub id: String,
    pub currency: String,
    pub amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub effective_from: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogVariant {
    pub id: String,
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: CatalogUnit,
    pub cost_minor: i64,
    pub status: CatalogStatus,
    pub current_price: Option<CatalogPrice>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProduct {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub status: CatalogStatus,
    pub variants: Vec<CatalogVariant>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPriceInput {
    pub amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    #[serde(default)]
    pub effective_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogVariantInput {
    pub name: String,
    pub sku: String,
    #[serde(default)]
    pub barcode: Option<String>,
    pub unit: CatalogUnit,
    pub cost_minor: i64,
    pub price: CatalogPriceInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCatalogItemInput {
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    pub variant: CatalogVariantInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCatalogVariantInput {
    pub product_id: String,
    pub variant: CatalogVariantInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCatalogProductInput {
    pub product_id: String,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    pub status: CatalogStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCatalogVariantInput {
    pub variant_id: String,
    pub name: String,
    pub sku: String,
    #[serde(default)]
    pub barcode: Option<String>,
    pub unit: CatalogUnit,
    pub cost_minor: i64,
    pub status: CatalogStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCatalogPriceInput {
    pub variant_id: String,
    pub price: CatalogPriceInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaymentMethod {
    Cash,
    Transfer,
    Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemberStatus {
    Active,
    Inactive,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProspectStatus {
    New,
    Contacted,
    Trial,
    Converted,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MembershipStatus {
    Scheduled,
    Active,
    Paused,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MembershipChargeStatus {
    Pending,
    Paid,
    Void,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipPlan {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub currency: String,
    pub grace_days: i64,
    pub version: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub member_number: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<String>,
    pub status: MemberStatus,
    pub joined_at: String,
    pub notes: Option<String>,
    pub active_membership: Option<Membership>,
    pub debt_minor: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub id: String,
    pub member_id: String,
    pub member_name: String,
    pub plan_id: String,
    pub plan_name: String,
    pub plan_version: i64,
    pub plan_version_id: String,
    pub status: MembershipStatus,
    pub start_date: String,
    pub end_date: String,
    pub price_minor: i64,
    pub currency: String,
    pub grace_days: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
    pub paused_at: Option<String>,
    pub pause_until: Option<String>,
    pub cancellation_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prospect {
    pub id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub source: Option<String>,
    pub status: ProspectStatus,
    pub notes: Option<String>,
    pub converted_member_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipCharge {
    pub id: String,
    pub membership_id: String,
    pub member_id: String,
    pub description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub due_date: String,
    pub status: MembershipChargeStatus,
    pub paid_minor: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipPayment {
    pub id: String,
    pub charge_id: String,
    pub membership_id: String,
    pub member_id: String,
    pub amount_minor: i64,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub receipt_number: String,
    pub cash_session_id: Option<String>,
    pub received_by: Option<String>,
    pub received_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipPaymentReceipt {
    pub payment_id: String,
    pub receipt_number: String,
    pub gym_name: String,
    pub member_number: String,
    pub member_name: String,
    pub charge_description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub received_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipDashboardTrendPoint {
    pub month: String,
    pub new_members: i64,
    pub renewals: i64,
    pub cancellations: i64,
    pub income_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberAccount {
    pub member_id: String,
    pub currency: String,
    pub charges: Vec<MembershipCharge>,
    pub payments: Vec<MembershipPayment>,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipDashboard {
    pub period_start: String,
    pub period_end: String,
    pub timezone: String,
    pub active_members: i64,
    pub active_memberships: i64,
    pub expiring_soon: i64,
    pub past_due: i64,
    pub debt_minor: i64,
    pub new_members_this_month: i64,
    pub income_this_month_minor: i64,
    pub renewals_this_month: i64,
    pub cancellations_this_month: i64,
    pub trend: Vec<MembershipDashboardTrendPoint>,
    pub recent_payments: Vec<MembershipPaymentReceipt>,
    pub expiring_memberships: Vec<Membership>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemberInput {
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub birth_date: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberInput {
    pub member_id: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub birth_date: Option<String>,
    pub status: MemberStatus,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMembershipPlanInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub grace_days: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    #[serde(default)]
    pub min_age: Option<i64>,
    #[serde(default)]
    pub max_age: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMembershipPlanInput {
    pub plan_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub grace_days: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    #[serde(default)]
    pub min_age: Option<i64>,
    #[serde(default)]
    pub max_age: Option<i64>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProspectInput {
    pub name: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProspectInput {
    pub prospect_id: String,
    pub name: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    pub status: ProspectStatus,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMembershipInput {
    pub member_id: String,
    pub plan_id: String,
    #[serde(default)]
    pub start_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewMembershipInput {
    pub membership_id: String,
    #[serde(default)]
    pub plan_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseMembershipInput {
    pub membership_id: String,
    pub until_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeMembershipInput {
    pub membership_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelMembershipInput {
    pub membership_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordMembershipPaymentInput {
    pub charge_id: String,
    pub amount_minor: i64,
    pub payment_method: PaymentMethod,
    #[serde(default)]
    pub reference: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItemInput {
    pub variant_id: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSaleInput {
    pub items: Vec<SaleItemInput>,
    pub payment_method: PaymentMethod,
    #[serde(default)]
    pub cash_received_minor: Option<i64>,
    #[serde(default)]
    pub payment_reference: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItem {
    pub id: String,
    pub variant_id: String,
    pub price_version_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub unit: CatalogUnit,
    pub quantity: i64,
    pub unit_amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub subtotal_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub refunded_quantity: i64,
    pub refunded_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sale {
    pub id: String,
    pub currency: String,
    pub subtotal_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub refunded_minor: i64,
    pub payment_method: PaymentMethod,
    pub cash_received_minor: Option<i64>,
    pub change_minor: Option<i64>,
    pub sold_by: Option<String>,
    pub sold_by_name: Option<String>,
    pub cash_session_id: Option<String>,
    pub payment_reference: Option<String>,
    pub items: Vec<SaleItem>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleReturnItemInput {
    pub sale_item_id: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSaleReturnInput {
    pub sale_id: String,
    pub reason: String,
    pub items: Vec<SaleReturnItemInput>,
    #[serde(default)]
    pub payment_reference: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleReturnItem {
    pub id: String,
    pub sale_item_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub quantity: i64,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleReturn {
    pub id: String,
    pub sale_id: String,
    pub cash_session_id: String,
    pub currency: String,
    pub refund_method: PaymentMethod,
    pub total_minor: i64,
    pub reason: String,
    pub authorized_by: Option<String>,
    pub authorized_by_name: Option<String>,
    pub payment_reference: Option<String>,
    pub created_at: String,
    pub items: Vec<SaleReturnItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentCharge {
    pub id: String,
    pub sale_id: String,
    pub sale_return_id: Option<String>,
    pub kind: String,
    pub method: PaymentMethod,
    pub provider: String,
    pub status: String,
    pub amount_minor: i64,
    pub currency: String,
    pub external_reference: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPaymentChargesInput {
    #[serde(default)]
    pub sale_id: Option<String>,
    #[serde(default)]
    pub external_reference: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportInput {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethodTotal {
    pub payment_method: PaymentMethod,
    pub sales_count: i64,
    pub gross_sales_minor: i64,
    pub refunds_count: i64,
    pub refunded_minor: i64,
    pub net_sales_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSalesTotal {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryAdjustmentTotal {
    pub variant_id: String,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub units_delta: i64,
    pub cost_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReport {
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
    pub payment_methods: Vec<PaymentMethodTotal>,
    pub cash_sessions_count: i64,
    pub open_cash_sessions: i64,
    pub closed_cash_sessions: i64,
    pub cash_difference_minor: i64,
    pub cash_difference_abs_minor: i64,
    pub product_sales: Vec<ProductSalesTotal>,
    pub inventory_adjustments: Vec<InventoryAdjustmentTotal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportExport {
    pub file_name: String,
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncClientSettings {
    pub base_url: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub ca_cert_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSyncClientSettingsInput {
    pub base_url: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub ca_cert_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunReport {
    pub pushed: u64,
    pub accepted: u64,
    pub retried: u64,
    pub dead_lettered: u64,
    pub pulled: u64,
    pub applied: u64,
    pub duplicates: u64,
    pub transport_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditExport {
    pub file_name: String,
    pub path: String,
    pub bytes: u64,
    pub checksum: String,
    pub events: u64,
    pub integrity_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReportScheduleInput {
    pub cadence: String,
    pub lookback_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSchedule {
    pub id: String,
    pub report_type: String,
    pub cadence: String,
    pub lookback_days: i64,
    pub enabled: bool,
    pub next_run_at: String,
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetReportScheduleEnabledInput {
    pub schedule_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRun {
    pub id: String,
    pub schedule_id: Option<String>,
    pub report_type: String,
    pub period_from: String,
    pub period_to: String,
    pub generated_at: String,
    pub file_name: Option<String>,
    pub checksum: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryMovement {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordInventoryEntryInput {
    pub variant_id: String,
    pub quantity: i64,
    pub unit_cost_minor: i64,
    pub reference: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustInventoryInput {
    pub variant_id: String,
    pub quantity_delta: i64,
    pub unit_cost_minor: Option<i64>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInventoryThresholdInput {
    pub variant_id: String,
    pub min_stock: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CashSessionStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashSession {
    pub id: String,
    pub status: CashSessionStatus,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CashMovementType {
    Income,
    Expense,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashMovement {
    pub id: String,
    pub cash_session_id: String,
    pub movement_type: CashMovementType,
    pub category: String,
    pub amount_minor: i64,
    pub payment_method: PaymentMethod,
    pub evidence: String,
    pub authorized_by: Option<String>,
    pub authorized_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCashSessionInput {
    pub opening_amount_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseCashSessionInput {
    pub session_id: String,
    pub counted_amount_minor: i64,
    pub difference_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordCashMovementInput {
    pub session_id: String,
    pub movement_type: CashMovementType,
    pub category: String,
    pub amount_minor: i64,
    pub payment_method: PaymentMethod,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditResult {
    Allowed,
    Denied,
    Failed,
}

/// Immutable audit row rendered in the local read-only audit screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEventSummary {
    pub id: String,
    pub sequence: i64,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub origin: String,
    pub correlation_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditIntegrity {
    pub valid: bool,
    pub checked_events: i64,
    pub first_broken_sequence: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub id: String,
    pub file_name: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub schema_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOutcome {
    pub backup_id: String,
    pub restored_at: String,
    pub schema_version: i64,
    pub integrity_ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    InvalidRequest,
    NotFound,
    Storage,
    Backup,
    Unauthorized,
    Internal,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[error("{code:?}: {message}")]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRequest, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GymSettings {
    pub id: String,
    pub business_name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub locale: String,
    pub opening_time: String,
    pub closing_time: String,
    pub slot_minutes: i64,
    pub grace_minutes: i64,
    pub inactivity_days: i64,
    pub frozen_days: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGymSettingsInput {
    pub business_name: String,
    #[serde(default)]
    pub legal_name: Option<String>,
    #[serde(default)]
    pub tax_id: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub locale: String,
    pub opening_time: String,
    pub closing_time: String,
    pub slot_minutes: i64,
    pub grace_minutes: i64,
    pub inactivity_days: i64,
    pub frozen_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChangeRequest {
    pub id: String,
    pub requested_by: String,
    pub requested_by_name: String,
    pub status: String,
    pub proposed: UpdateGymSettingsInput,
    pub created_at: String,
    pub reviewed_by: Option<String>,
    pub reviewed_by_name: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSettingsChangeInput {
    pub id: String,
    pub approve: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationLink {
    pub id: String,
    pub module_id: String,
    pub display_name: String,
    pub contract_version: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub last_seen_at: Option<String>,
    pub last_event_at: Option<String>,
    pub last_snapshot_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertIntegrationLinkInput {
    pub module_id: String,
    pub display_name: String,
    pub contract_version: String,
    pub status: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationJournalSummary {
    pub pending: i64,
    pub sent: i64,
    pub dead_letter: i64,
    pub inbox_events: i64,
    pub last_server_sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassSession {
    pub id: String,
    pub name: String,
    pub coach_user_id: Option<String>,
    pub coach_name: Option<String>,
    pub weekday: i64,
    pub start_time: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub room: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub booked: i64,
    pub attended: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClassInput {
    pub name: String,
    #[serde(default)]
    pub coach_user_id: Option<String>,
    pub weekday: i64,
    pub start_time: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    #[serde(default)]
    pub room: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClassInput {
    pub class_id: String,
    pub name: String,
    #[serde(default)]
    pub coach_user_id: Option<String>,
    pub weekday: i64,
    pub start_time: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    #[serde(default)]
    pub room: Option<String>,
    pub status: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassBooking {
    pub id: String,
    pub class_id: String,
    pub class_name: String,
    pub member_id: String,
    pub member_name: String,
    pub status: String,
    pub note: Option<String>,
    pub booked_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookClassInput {
    pub class_id: String,
    pub member_id: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClassBookingInput {
    pub booking_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub audience: String,
    pub status: String,
    pub starts_on: Option<String>,
    pub ends_on: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampaignInput {
    pub name: String,
    pub channel: String,
    pub audience: String,
    pub status: String,
    #[serde(default)]
    pub starts_on: Option<String>,
    #[serde(default)]
    pub ends_on: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCampaignInput {
    pub campaign_id: String,
    pub name: String,
    pub channel: String,
    pub audience: String,
    pub status: String,
    #[serde(default)]
    pub starts_on: Option<String>,
    #[serde(default)]
    pub ends_on: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelStage {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketingFunnel {
    pub prospects_total: i64,
    pub prospects_converted: i64,
    pub conversion_bps: i64,
    pub stages: Vec<FunnelStage>,
    pub active_campaigns: i64,
    pub expiring_memberships: i64,
    pub inactive_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub area: Option<String>,
    pub status: String,
    pub acquired_on: Option<String>,
    pub notes: Option<String>,
    pub open_orders: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEquipmentInput {
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub acquired_on: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEquipmentInput {
    pub equipment_id: String,
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    pub status: String,
    #[serde(default)]
    pub acquired_on: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceOrder {
    pub id: String,
    pub equipment_id: Option<String>,
    pub equipment_name: Option<String>,
    pub kind: String,
    pub status: String,
    pub priority: String,
    pub summary: String,
    pub detail: Option<String>,
    pub due_on: Option<String>,
    pub reported_by: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMaintenanceOrderInput {
    #[serde(default)]
    pub equipment_id: Option<String>,
    pub kind: String,
    pub priority: String,
    pub summary: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub due_on: Option<String>,
    #[serde(default)]
    pub reported_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMaintenanceOrderInput {
    pub order_id: String,
    pub status: String,
    pub priority: String,
    pub summary: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub due_on: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_report_serializes_in_camel_case() {
        let report = HealthReport {
            service: "fitness-gym".into(),
            version: "0.1.0".into(),
            contract_version: CONTRACT_VERSION.into(),
            status: HealthStatus::Ok,
            modules: vec![ModuleHealth::ok("identity")],
            database: Some(DatabaseStatus {
                path: "/tmp/gym.db".into(),
                encrypted: true,
                cipher_version: Some("4.6.1".into()),
                schema_version: 1,
                integrity_ok: true,
                organizations: 1,
            }),
            checked_at: "2026-10-20T00:00:00Z".into(),
        };

        let json = serde_json::to_value(&report).expect("serializes");
        assert_eq!(json["contractVersion"], CONTRACT_VERSION);
        assert_eq!(json["database"]["cipherVersion"], "4.6.1");
        assert!(json.get("checked_at").is_none());
    }

    #[test]
    fn access_contract_serializes_in_camel_case() {
        let request = AccessRequest {
            credential_id: "CRED-ALLOW".into(),
            device_id: "recepcion-1".into(),
            idempotency_key: "req-1".into(),
        };
        let json = serde_json::to_value(request).expect("serializes");
        assert_eq!(json["credentialId"], "CRED-ALLOW");
        assert_eq!(json["deviceId"], "recepcion-1");
        assert_eq!(json["idempotencyKey"], "req-1");
    }

    #[test]
    fn setup_and_session_contracts_serialize_in_camel_case() {
        let setup = SetupState {
            configured: true,
            gym_name: Some("Gimnasio Centro".into()),
            owner_ready: false,
        };
        let json = serde_json::to_value(setup).expect("serializes");
        assert_eq!(json["gymName"], "Gimnasio Centro");
        assert_eq!(json["ownerReady"], false);

        let session = SessionSummary {
            user_id: "user-1".into(),
            organization_id: "org-1".into(),
            organization_name: "Gimnasio Centro".into(),
            name: "Ana".into(),
            role: StaffRole::Owner,
            permissions: vec!["manageSettings".into()],
        };
        let json = serde_json::to_value(session).expect("serializes");
        assert_eq!(json["role"], "owner");
        assert_eq!(json["permissions"][0], "manageSettings");

        let kit = RecoveryKit {
            session: SessionSummary {
                user_id: "user-1".into(),
                organization_id: "org-1".into(),
                organization_name: "Gimnasio Centro".into(),
                name: "Ana".into(),
                role: StaffRole::Owner,
                permissions: vec!["manageSettings".into()],
            },
            recovery_code: "AAAAA-BBBBB-CCCCC-DDDDD".into(),
        };
        let json = serde_json::to_value(kit).expect("serializes");
        assert_eq!(json["recoveryCode"], "AAAAA-BBBBB-CCCCC-DDDDD");
        assert_eq!(json["session"]["role"], "owner");
    }

    #[test]
    fn api_error_omits_empty_details() {
        let error = ApiError::invalid("bad input");
        let json = serde_json::to_value(&error).expect("serializes");
        assert_eq!(json["code"], "invalidRequest");
        assert!(json.get("details").is_none());
    }
}
