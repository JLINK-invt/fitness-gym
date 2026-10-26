use chrono::{Timelike, Utc};
use gym_backup::{
    create_backup as write_backup, list_backups as read_backups, restore_backup as apply_restore,
};
use gym_contracts::{
    AccessConflict, AccessDecision, AccessEvent, AccessEventInput, AccessEventSummary,
    AccessReconciliationInput, AccessReconciliationSummary, AccessRequest, AccessSnapshot,
    AddCatalogPriceInput, AddCatalogVariantInput, AdjustInventoryInput, ApiError, AttendanceDay,
    AttendanceHour, AttendanceInput, AttendanceMember, AttendancePlan, AttendanceReport,
    AttendanceZone, AuditEventSummary, AuditIntegrity, AuditResult, BackupSummary,
    CancelMembershipInput, CashMovement, CashMovementType, CashSession, CashSessionStatus,
    CatalogPrice, CatalogPriceInput, CatalogProduct, CatalogStatus, CatalogUnit, CatalogVariant,
    CloseCashSessionInput, CreateCatalogItemInput, CreateMemberInput, CreateMembershipInput,
    CreateMembershipPlanInput, CreateProspectInput, CreateSaleInput, CreateSaleReturnInput,
    ErrorCode, FinancialReport, FinancialReportExport, FinancialReportInput, HealthReport,
    HealthStatus, InventoryAdjustmentTotal, InventoryItem, InventoryMovement,
    ListPaymentChargesInput, Member, MemberAccount, MemberCredential, MemberStatus, Membership,
    MembershipCharge, MembershipChargeStatus, MembershipDashboard, MembershipDashboardTrendPoint,
    MembershipPayment, MembershipPaymentReceipt, MembershipPlan, MembershipStatus, ModuleHealth,
    NewReportScheduleInput, OpenCashSessionInput, PauseMembershipInput, PaymentCharge,
    PaymentMethod, PaymentMethodTotal, ProductSalesTotal, Prospect, ProspectStatus,
    RecordCashMovementInput, RecordInventoryEntryInput, RecordMembershipPaymentInput, RecoveryCode,
    RecoveryKit, RenewMembershipInput, ReportRun, ReportSchedule, RestoreOutcome,
    ResumeMembershipInput, Sale, SaleItem, SaleReturn, SaleReturnItem, SessionSummary,
    SetReportScheduleEnabledInput, SetupState, StaffRole, StaffStatus, StaffSummary,
    UpdateCatalogProductInput, UpdateCatalogVariantInput, UpdateInventoryThresholdInput,
    UpdateMemberInput, UpdateMembershipPlanInput, UpdateProspectInput, CONTRACT_VERSION,
};
use gym_core::{
    access::{decide_member_access, decide_simulated_access, MemberAccessContext},
    auth::{
        generate_recovery_code, hash_password, hash_recovery_code, normalize_person_name,
        verify_password, verify_recovery_code, IDLE_TIMEOUT_MINUTES, LOGIN_LOCK_MINUTES,
        MAX_LOGIN_ATTEMPTS, OWNER_MIN_PASSWORD_CHARS, STAFF_MIN_PASSWORD_CHARS,
    },
    rbac::{self, Permission, Role},
    CORE_VERSION, SERVICE_NAME,
};
use gym_data::{
    sha256_hex, AccessEventRecord, AccessEventWriteOutcome, AccessSnapshotRecord,
    AttendanceReportRecord, CashMovementRecord, CashSessionRecord, CatalogPriceRecord,
    CatalogProductRecord, CatalogVariantRecord, DataError, Db, DbKey, EnvKeyProvider,
    FileKeyProvider, FinancialReportRecord, InventoryAdjustmentRecord, InventoryItemRecord,
    InventoryMovementRecord, KeyProvider, KeyringKeyProvider, MemberAccountRecord,
    MemberCredentialRecord, MemberRecord, MembershipChargeRecord, MembershipDashboardRecord,
    MembershipDashboardTrendRecord, MembershipPaymentReceiptRecord, MembershipPaymentRecord,
    MembershipPlanRecord, MembershipRecord, NewCashMovement, NewCatalogItem, NewCatalogPrice,
    NewCatalogProductUpdate, NewCatalogVariant, NewCatalogVariantUpdate, NewInventoryAdjustment,
    NewInventoryEntry, NewMember, NewMembershipPayment, NewMembershipPlan, NewProspect,
    NewReportSchedule, NewSale, NewSaleItem, NewSaleReturn, NewSaleReturnItem, PaymentChargeRecord,
    PaymentMethodTotalRecord, ProductSalesRecord, ProspectRecord, ReportRunRecord,
    ReportScheduleRecord, SaleItemRecord, SaleRecord, SaleReturnItemRecord, SaleReturnRecord,
    UpdateMember, UpdateMembershipPlan, UpdateProspect,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{Manager, State};
use tracing_subscriber::EnvFilter;

const KEYRING_SERVICE: &str = "com.fitnessgym.desktop";
const KEYRING_ACCOUNT: &str = "database-key";

struct AppPaths {
    db_path: PathBuf,
    backups_dir: PathBuf,
    exports_dir: PathBuf,
}

fn load_or_migrate_key(data_dir: &Path) -> Result<DbKey, String> {
    if std::env::var_os("GYM_DB_KEY").is_some() {
        return EnvKeyProvider::default()
            .load_or_create()
            .map_err(|error| format!("clave GYM_DB_KEY inválida: {error}"));
    }

    let file_provider = FileKeyProvider::new(data_dir.join("keys").join("db.key"));
    let keyring = KeyringKeyProvider::new(KEYRING_SERVICE, KEYRING_ACCOUNT);

    match keyring.get() {
        Ok(Some(key)) => {
            remove_legacy_key(file_provider.path());
            Ok(key)
        }
        Ok(None) => {
            let had_legacy_key = file_provider.path().exists();
            let key = if had_legacy_key {
                file_provider
                    .load_or_create()
                    .map_err(|error| format!("no se pudo leer la clave existente: {error}"))?
            } else {
                DbKey::generate()
            };

            match keyring.store(&key) {
                Ok(()) => {
                    remove_legacy_key(file_provider.path());
                    Ok(key)
                }
                Err(error) if cfg!(debug_assertions) => {
                    tracing::warn!(%error, "keychain no disponible; usando archivo de desarrollo");
                    if !had_legacy_key {
                        file_provider
                            .store(&key)
                            .map_err(|error| format!("no se pudo guardar la clave: {error}"))?;
                    }
                    Ok(key)
                }
                Err(error) => Err(format!(
                    "no se pudo guardar la clave en el keychain del sistema: {error}"
                )),
            }
        }
        Err(error) if cfg!(debug_assertions) => {
            tracing::warn!(%error, "keychain no disponible; usando archivo de desarrollo");
            file_provider
                .load_or_create()
                .map_err(|error| format!("no se pudo leer la clave: {error}"))
        }
        Err(error) => Err(format!(
            "no se pudo acceder al keychain del sistema: {error}"
        )),
    }
}

fn remove_legacy_key(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => tracing::info!(
            path = %path.display(),
            "clave migrada al keychain; archivo heredado eliminado"
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!(
            path = %path.display(),
            %error,
            "no se pudo eliminar el archivo de clave heredado"
        ),
    }
}

#[derive(Debug, Clone)]
struct ActiveSession {
    session: gym_core::auth::Session,
    last_seen: Instant,
}

#[derive(Debug, Default)]
struct LoginFailures {
    attempts: Vec<Instant>,
    blocked_until: Option<Instant>,
}

#[derive(Debug, Default)]
struct AuthState {
    session: Option<ActiveSession>,
    failures: HashMap<String, LoginFailures>,
}

impl AuthState {
    fn current_session(&mut self) -> Option<SessionSummary> {
        if self.session_expired() {
            self.session = None;
            return None;
        }
        self.session.as_ref().map(session_summary)
    }

    fn authorize(&mut self, permission: Permission) -> Result<gym_core::auth::Session, ApiError> {
        self.authorize_any(&[permission])
    }

    fn authorize_any(
        &mut self,
        permissions: &[Permission],
    ) -> Result<gym_core::auth::Session, ApiError> {
        if self.session_expired() {
            self.session = None;
            return Err(ApiError::new(
                ErrorCode::Unauthorized,
                "la sesión expiró por inactividad; vuelve a entrar",
            ));
        }

        let active = self.session.as_mut().ok_or_else(|| {
            ApiError::new(ErrorCode::Unauthorized, "inicia sesión para continuar")
        })?;
        active.last_seen = Instant::now();
        let session = active.session.clone();
        let allowed = permissions
            .iter()
            .any(|permission| rbac::authorize(session.role, *permission).is_ok());
        if !allowed {
            return Err(ApiError::new(
                ErrorCode::Unauthorized,
                "tu rol no permite realizar esta acción",
            ));
        }
        Ok(session)
    }

    fn session_expired(&self) -> bool {
        self.session.as_ref().is_some_and(|active| {
            active.last_seen.elapsed() >= Duration::from_secs(60 * IDLE_TIMEOUT_MINUTES as u64)
        })
    }

    fn is_blocked(&self, key: &str, now: Instant) -> bool {
        self.failures
            .get(key)
            .and_then(|failures| failures.blocked_until)
            .is_some_and(|blocked_until| blocked_until > now)
    }

    /// Records a failed attempt and returns `true` when the account is locked.
    fn record_failure(&mut self, key: &str, now: Instant) -> bool {
        let failures = self.failures.entry(key.to_string()).or_default();
        if failures
            .blocked_until
            .is_some_and(|blocked_until| blocked_until > now)
        {
            return true;
        }

        let window = Duration::from_secs(60 * LOGIN_LOCK_MINUTES as u64);
        failures
            .attempts
            .retain(|attempt| now.duration_since(*attempt) < window);
        failures.attempts.push(now);
        if failures.attempts.len() >= MAX_LOGIN_ATTEMPTS as usize {
            failures.attempts.clear();
            failures.blocked_until = Some(now + window);
            return true;
        }
        false
    }

    fn clear_failures(&mut self, key: &str) {
        self.failures.remove(key);
    }
}

struct AppState {
    db: Result<Arc<Db>, String>,
    key: DbKey,
    paths: AppPaths,
    auth: Mutex<AuthState>,
}

impl AppState {
    fn db(&self) -> Result<&Db, ApiError> {
        match &self.db {
            Ok(db) => Ok(db.as_ref()),
            Err(error) => Err(ApiError::new(
                ErrorCode::Storage,
                "la base de datos local no está disponible",
            )
            .with_details(error.clone())),
        }
    }

    fn authorize(&self, permission: Permission) -> Result<gym_core::auth::Session, ApiError> {
        let mut auth = self.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        auth.authorize(permission)
    }

    fn authorize_any(
        &self,
        permissions: &[Permission],
    ) -> Result<gym_core::auth::Session, ApiError> {
        let mut auth = self.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        auth.authorize_any(permissions)
    }

    fn clear_session(&self) {
        if let Ok(mut auth) = self.auth.lock() {
            auth.session = None;
        }
    }
}

fn staff_role(role: Role) -> StaffRole {
    match role {
        Role::Owner => StaffRole::Owner,
        Role::Manager => StaffRole::Manager,
        Role::Cashier => StaffRole::Cashier,
        Role::Accounting => StaffRole::Accounting,
    }
}

fn role_from_str(value: &str) -> Option<Role> {
    match value {
        "owner" => Some(Role::Owner),
        "manager" => Some(Role::Manager),
        "cashier" => Some(Role::Cashier),
        "accounting" => Some(Role::Accounting),
        _ => None,
    }
}

fn session_summary(active: &ActiveSession) -> SessionSummary {
    let session = &active.session;
    SessionSummary {
        user_id: session.user_id.clone(),
        organization_id: session.organization_id.clone(),
        name: session.name.clone(),
        role: staff_role(session.role),
        permissions: rbac::permissions_for(session.role)
            .iter()
            .map(|permission| permission.as_str().to_string())
            .collect(),
    }
}

fn build_session(
    credentials: gym_data::UserCredentialRecord,
) -> Result<gym_core::auth::Session, ApiError> {
    let role = role_from_str(&credentials.role).ok_or_else(|| {
        ApiError::new(
            ErrorCode::Internal,
            "la membresía tiene un rol desconocido; revisa la base local",
        )
    })?;
    Ok(gym_core::auth::Session {
        user_id: credentials.user_id,
        organization_id: credentials.organization_id,
        name: credentials.name,
        role,
    })
}

fn ensure_staff_role_allowed(
    session: &gym_core::auth::Session,
    current_role: Option<&str>,
    role: Role,
) -> Result<(), ApiError> {
    if role == Role::Owner || current_role == Some("owner") {
        return Err(ApiError::invalid(
            "la cuenta del dueño no se puede crear ni modificar",
        ));
    }
    if session.role == Role::Manager && (role == Role::Manager || current_role == Some("manager")) {
        return Err(ApiError::new(
            ErrorCode::Unauthorized,
            "solo puedes administrar cajeros y contabilidad",
        ));
    }
    Ok(())
}

fn staff_status(value: &str) -> StaffStatus {
    match value {
        "revoked" => StaffStatus::Revoked,
        _ => StaffStatus::Active,
    }
}

fn staff_status_str(value: StaffStatus) -> &'static str {
    match value {
        StaffStatus::Active => "active",
        StaffStatus::Revoked => "revoked",
    }
}

fn role_from_contract(role: StaffRole) -> Role {
    match role {
        StaffRole::Owner => Role::Owner,
        StaffRole::Manager => Role::Manager,
        StaffRole::Cashier => Role::Cashier,
        StaffRole::Accounting => Role::Accounting,
    }
}

fn staff_summary(record: gym_data::StaffRecord) -> StaffSummary {
    StaffSummary {
        user_id: record.user_id,
        name: record.name,
        role: role_from_str(&record.role)
            .map(staff_role)
            .unwrap_or(StaffRole::Cashier),
        status: staff_status(&record.status),
        last_login_at: record.last_login_at,
        created_at: record.created_at,
    }
}

fn member_status(value: &str) -> MemberStatus {
    match value {
        "inactive" => MemberStatus::Inactive,
        "archived" => MemberStatus::Archived,
        _ => MemberStatus::Active,
    }
}

fn member_status_str(value: MemberStatus) -> &'static str {
    match value {
        MemberStatus::Active => "active",
        MemberStatus::Inactive => "inactive",
        MemberStatus::Archived => "archived",
    }
}

fn prospect_status(value: &str) -> ProspectStatus {
    match value {
        "contacted" => ProspectStatus::Contacted,
        "trial" => ProspectStatus::Trial,
        "converted" => ProspectStatus::Converted,
        "lost" => ProspectStatus::Lost,
        _ => ProspectStatus::New,
    }
}

fn prospect_status_str(value: ProspectStatus) -> &'static str {
    match value {
        ProspectStatus::New => "new",
        ProspectStatus::Contacted => "contacted",
        ProspectStatus::Trial => "trial",
        ProspectStatus::Converted => "converted",
        ProspectStatus::Lost => "lost",
    }
}

fn membership_status(value: &str) -> MembershipStatus {
    match value {
        "scheduled" => MembershipStatus::Scheduled,
        "paused" => MembershipStatus::Paused,
        "expired" => MembershipStatus::Expired,
        "cancelled" => MembershipStatus::Cancelled,
        _ => MembershipStatus::Active,
    }
}

fn membership_charge_status(value: &str) -> MembershipChargeStatus {
    match value {
        "paid" => MembershipChargeStatus::Paid,
        "void" => MembershipChargeStatus::Void,
        _ => MembershipChargeStatus::Pending,
    }
}

fn membership_plan_summary(record: MembershipPlanRecord) -> MembershipPlan {
    MembershipPlan {
        id: record.id,
        name: record.name,
        description: record.description,
        duration_days: record.duration_days,
        price_minor: record.price_minor,
        currency: record.currency,
        grace_days: record.grace_days,
        version: record.version,
        access_days: record.access_days,
        access_start_minute: record.access_start_minute,
        access_end_minute: record.access_end_minute,
        min_age: record.min_age,
        max_age: record.max_age,
        active: record.active,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn membership_summary(record: MembershipRecord) -> Membership {
    Membership {
        id: record.id,
        member_id: record.member_id,
        member_name: record.member_name,
        plan_id: record.plan_id,
        plan_name: record.plan_name,
        plan_version: record.plan_version,
        plan_version_id: record.plan_version_id,
        status: membership_status(&record.status),
        start_date: record.start_date,
        end_date: record.end_date,
        price_minor: record.price_minor,
        currency: record.currency,
        grace_days: record.grace_days,
        access_days: record.access_days,
        access_start_minute: record.access_start_minute,
        access_end_minute: record.access_end_minute,
        min_age: record.min_age,
        max_age: record.max_age,
        paused_at: record.paused_at,
        pause_until: record.pause_until,
        cancellation_reason: record.cancellation_reason,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn member_summary(record: MemberRecord) -> Member {
    Member {
        id: record.id,
        member_number: record.member_number,
        first_name: record.first_name,
        last_name: record.last_name,
        phone: record.phone,
        email: record.email,
        birth_date: record.birth_date,
        status: member_status(&record.status),
        joined_at: record.joined_at,
        notes: record.notes,
        active_membership: record.active_membership.map(membership_summary),
        debt_minor: record.debt_minor,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn prospect_summary(record: ProspectRecord) -> Prospect {
    Prospect {
        id: record.id,
        name: record.name,
        phone: record.phone,
        email: record.email,
        source: record.source,
        status: prospect_status(&record.status),
        notes: record.notes,
        converted_member_id: record.converted_member_id,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn membership_charge_summary(record: MembershipChargeRecord) -> MembershipCharge {
    MembershipCharge {
        id: record.id,
        membership_id: record.membership_id,
        member_id: record.member_id,
        description: record.description,
        amount_minor: record.amount_minor,
        currency: record.currency,
        due_date: record.due_date,
        status: membership_charge_status(&record.status),
        paid_minor: record.paid_minor,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn membership_payment_summary(record: MembershipPaymentRecord) -> MembershipPayment {
    MembershipPayment {
        id: record.id,
        charge_id: record.charge_id,
        membership_id: record.membership_id,
        member_id: record.member_id,
        amount_minor: record.amount_minor,
        payment_method: payment_method(&record.payment_method),
        reference: record.reference,
        receipt_number: record.receipt_number,
        cash_session_id: record.cash_session_id,
        received_by: record.received_by,
        received_by_name: record.received_by_name,
        created_at: record.created_at,
    }
}

fn membership_payment_receipt_summary(
    record: MembershipPaymentReceiptRecord,
) -> MembershipPaymentReceipt {
    MembershipPaymentReceipt {
        payment_id: record.payment_id,
        receipt_number: record.receipt_number,
        gym_name: record.gym_name,
        member_number: record.member_number,
        member_name: record.member_name,
        charge_description: record.charge_description,
        amount_minor: record.amount_minor,
        currency: record.currency,
        payment_method: payment_method(&record.payment_method),
        reference: record.reference,
        received_by_name: record.received_by_name,
        created_at: record.created_at,
    }
}

fn membership_trend_summary(
    record: MembershipDashboardTrendRecord,
) -> MembershipDashboardTrendPoint {
    MembershipDashboardTrendPoint {
        month: record.month,
        new_members: record.new_members,
        renewals: record.renewals,
        cancellations: record.cancellations,
        income_minor: record.income_minor,
    }
}

fn member_account_summary(record: MemberAccountRecord) -> MemberAccount {
    MemberAccount {
        member_id: record.member_id,
        currency: record.currency,
        charges: record
            .charges
            .into_iter()
            .map(membership_charge_summary)
            .collect(),
        payments: record
            .payments
            .into_iter()
            .map(membership_payment_summary)
            .collect(),
        balance_minor: record.balance_minor,
    }
}

fn membership_dashboard_summary(record: MembershipDashboardRecord) -> MembershipDashboard {
    MembershipDashboard {
        period_start: record.period_start,
        period_end: record.period_end,
        timezone: record.timezone,
        active_members: record.active_members,
        active_memberships: record.active_memberships,
        expiring_soon: record.expiring_soon,
        past_due: record.past_due,
        debt_minor: record.debt_minor,
        new_members_this_month: record.new_members_this_month,
        income_this_month_minor: record.income_this_month_minor,
        renewals_this_month: record.renewals_this_month,
        cancellations_this_month: record.cancellations_this_month,
        trend: record
            .trend
            .into_iter()
            .map(membership_trend_summary)
            .collect(),
        recent_payments: record
            .recent_payments
            .into_iter()
            .map(membership_payment_receipt_summary)
            .collect(),
        expiring_memberships: record
            .expiring_memberships
            .into_iter()
            .map(membership_summary)
            .collect(),
    }
}

fn catalog_status(value: &str) -> CatalogStatus {
    match value {
        "archived" => CatalogStatus::Archived,
        _ => CatalogStatus::Active,
    }
}

fn catalog_status_str(value: CatalogStatus) -> &'static str {
    match value {
        CatalogStatus::Active => "active",
        CatalogStatus::Archived => "archived",
    }
}

fn catalog_unit(value: &str) -> CatalogUnit {
    match value {
        "piece" => CatalogUnit::Piece,
        "session" => CatalogUnit::Session,
        "package" => CatalogUnit::Package,
        _ => CatalogUnit::Service,
    }
}

fn catalog_unit_str(value: CatalogUnit) -> &'static str {
    match value {
        CatalogUnit::Piece => "piece",
        CatalogUnit::Service => "service",
        CatalogUnit::Session => "session",
        CatalogUnit::Package => "package",
    }
}

fn catalog_price(record: CatalogPriceRecord) -> CatalogPrice {
    CatalogPrice {
        id: record.id,
        currency: record.currency,
        amount_minor: record.amount_minor,
        tax_rate_bps: record.tax_rate_bps,
        price_includes_tax: record.price_includes_tax,
        effective_from: record.effective_from,
        created_at: record.created_at,
    }
}

fn catalog_variant(record: CatalogVariantRecord) -> CatalogVariant {
    CatalogVariant {
        id: record.id,
        name: record.name,
        sku: record.sku,
        barcode: record.barcode,
        unit: catalog_unit(&record.unit),
        cost_minor: record.cost_minor,
        status: catalog_status(&record.status),
        current_price: record.current_price.map(catalog_price),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn catalog_product(record: CatalogProductRecord) -> CatalogProduct {
    CatalogProduct {
        id: record.id,
        name: record.name,
        category: record.category,
        status: catalog_status(&record.status),
        variants: record.variants.into_iter().map(catalog_variant).collect(),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn payment_method(value: &str) -> PaymentMethod {
    match value {
        "transfer" => PaymentMethod::Transfer,
        "point" => PaymentMethod::Point,
        _ => PaymentMethod::Cash,
    }
}

fn payment_method_str(value: PaymentMethod) -> &'static str {
    match value {
        PaymentMethod::Cash => "cash",
        PaymentMethod::Transfer => "transfer",
        PaymentMethod::Point => "point",
    }
}

fn sale_item(record: SaleItemRecord) -> SaleItem {
    SaleItem {
        id: record.id,
        variant_id: record.variant_id,
        price_version_id: record.price_version_id,
        product_name: record.product_name,
        variant_name: record.variant_name,
        sku: record.sku,
        unit: catalog_unit(&record.unit),
        quantity: record.quantity,
        unit_amount_minor: record.unit_amount_minor,
        tax_rate_bps: record.tax_rate_bps,
        price_includes_tax: record.price_includes_tax,
        subtotal_minor: record.subtotal_minor,
        tax_minor: record.tax_minor,
        total_minor: record.total_minor,
        refunded_quantity: record.refunded_quantity,
        refunded_minor: record.refunded_minor,
    }
}

fn sale(record: SaleRecord) -> Sale {
    Sale {
        id: record.id,
        currency: record.currency,
        subtotal_minor: record.subtotal_minor,
        tax_minor: record.tax_minor,
        total_minor: record.total_minor,
        refunded_minor: record.refunded_minor,
        payment_method: payment_method(&record.payment_method),
        cash_received_minor: record.cash_received_minor,
        change_minor: record.change_minor,
        sold_by: record.sold_by,
        sold_by_name: record.sold_by_name,
        cash_session_id: record.cash_session_id,
        payment_reference: record.payment_reference,
        items: record.items.into_iter().map(sale_item).collect(),
        created_at: record.created_at,
    }
}

fn sale_return_item(record: SaleReturnItemRecord) -> SaleReturnItem {
    SaleReturnItem {
        id: record.id,
        sale_item_id: record.sale_item_id,
        product_name: record.product_name,
        variant_name: record.variant_name,
        sku: record.sku,
        quantity: record.quantity,
        amount_minor: record.amount_minor,
    }
}

fn sale_return(record: SaleReturnRecord) -> SaleReturn {
    SaleReturn {
        id: record.id,
        sale_id: record.sale_id,
        cash_session_id: record.cash_session_id,
        currency: record.currency,
        refund_method: payment_method(&record.refund_method),
        total_minor: record.total_minor,
        reason: record.reason,
        authorized_by: record.authorized_by,
        authorized_by_name: record.authorized_by_name,
        payment_reference: record.payment_reference,
        created_at: record.created_at,
        items: record.items.into_iter().map(sale_return_item).collect(),
    }
}

fn payment_charge(record: PaymentChargeRecord) -> PaymentCharge {
    PaymentCharge {
        id: record.id,
        sale_id: record.sale_id,
        sale_return_id: record.sale_return_id,
        kind: record.kind,
        method: payment_method(&record.method),
        provider: record.provider,
        status: record.status,
        amount_minor: record.amount_minor,
        currency: record.currency,
        external_reference: record.external_reference,
        created_at: record.created_at,
    }
}

fn payment_method_total(record: PaymentMethodTotalRecord) -> PaymentMethodTotal {
    PaymentMethodTotal {
        payment_method: payment_method(&record.payment_method),
        sales_count: record.sales_count,
        gross_sales_minor: record.gross_sales_minor,
        refunds_count: record.refunds_count,
        refunded_minor: record.refunded_minor,
        net_sales_minor: record.net_sales_minor,
    }
}

fn product_sales_total(record: ProductSalesRecord) -> ProductSalesTotal {
    ProductSalesTotal {
        variant_id: record.variant_id,
        product_name: record.product_name,
        variant_name: record.variant_name,
        sku: record.sku,
        unit: record.unit,
        sold_units: record.sold_units,
        refunded_units: record.refunded_units,
        net_units: record.net_units,
        gross_sales_minor: record.gross_sales_minor,
        refunded_minor: record.refunded_minor,
        net_sales_minor: record.net_sales_minor,
        cost_minor: record.cost_minor,
        margin_minor: record.margin_minor,
    }
}

fn inventory_adjustment_total(record: InventoryAdjustmentRecord) -> InventoryAdjustmentTotal {
    InventoryAdjustmentTotal {
        variant_id: record.variant_id,
        product_name: record.product_name,
        variant_name: record.variant_name,
        sku: record.sku,
        units_delta: record.units_delta,
        cost_minor: record.cost_minor,
    }
}

fn financial_report_contract(record: FinancialReportRecord) -> FinancialReport {
    FinancialReport {
        from: record.from,
        to: record.to,
        generated_at: record.generated_at,
        currency: record.currency,
        sales_count: record.sales_count,
        gross_sales_minor: record.gross_sales_minor,
        tax_minor: record.tax_minor,
        refunds_count: record.refunds_count,
        refunded_minor: record.refunded_minor,
        net_sales_minor: record.net_sales_minor,
        average_sale_minor: record.average_sale_minor,
        attach_rate_bps: record.attach_rate_bps,
        items_per_sale_milli: record.items_per_sale_milli,
        cost_of_goods_minor: record.cost_of_goods_minor,
        margin_minor: record.margin_minor,
        margin_bps: record.margin_bps,
        inventory_value_start_minor: record.inventory_value_start_minor,
        inventory_value_end_minor: record.inventory_value_end_minor,
        inventory_turnover_bps: record.inventory_turnover_bps,
        adjustments_units: record.adjustments_units,
        adjustments_cost_minor: record.adjustments_cost_minor,
        payment_methods: record
            .payment_methods
            .into_iter()
            .map(payment_method_total)
            .collect(),
        cash_sessions_count: record.cash_sessions_count,
        open_cash_sessions: record.open_cash_sessions,
        closed_cash_sessions: record.closed_cash_sessions,
        cash_difference_minor: record.cash_difference_minor,
        cash_difference_abs_minor: record.cash_difference_abs_minor,
        product_sales: record
            .product_sales
            .into_iter()
            .map(product_sales_total)
            .collect(),
        inventory_adjustments: record
            .inventory_adjustments
            .into_iter()
            .map(inventory_adjustment_total)
            .collect(),
    }
}

fn inventory_item(record: InventoryItemRecord) -> InventoryItem {
    InventoryItem {
        variant_id: record.variant_id,
        product_name: record.product_name,
        variant_name: record.variant_name,
        sku: record.sku,
        unit: record.unit,
        stock: record.stock,
        value_minor: record.value_minor,
        avg_unit_cost_minor: record.avg_unit_cost_minor,
        min_stock: record.min_stock,
        low_stock: record.low_stock,
    }
}

fn inventory_movement(record: InventoryMovementRecord) -> InventoryMovement {
    InventoryMovement {
        id: record.id,
        variant_id: record.variant_id,
        movement_type: record.movement_type,
        quantity_delta: record.quantity_delta,
        unit_cost_minor: record.unit_cost_minor,
        total_cost_minor: record.total_cost_minor,
        source_type: record.source_type,
        source_id: record.source_id,
        reference: record.reference,
        note: record.note,
        authorized_by: record.authorized_by,
        authorized_by_name: record.authorized_by_name,
        created_at: record.created_at,
    }
}

fn cash_session_status(value: &str) -> CashSessionStatus {
    match value {
        "closed" => CashSessionStatus::Closed,
        _ => CashSessionStatus::Open,
    }
}

fn cash_session(record: CashSessionRecord) -> CashSession {
    CashSession {
        id: record.id,
        status: cash_session_status(&record.status),
        currency: record.currency,
        opening_amount_minor: record.opening_amount_minor,
        expected_cash_minor: record.expected_cash_minor,
        cash_sales_minor: record.cash_sales_minor,
        sales_count: record.sales_count,
        cash_income_minor: record.cash_income_minor,
        income_minor: record.income_minor,
        cash_expense_minor: record.cash_expense_minor,
        expense_minor: record.expense_minor,
        cash_refunds_minor: record.cash_refunds_minor,
        refunded_minor: record.refunded_minor,
        refunds_count: record.refunds_count,
        movements_count: record.movements_count,
        counted_amount_minor: record.counted_amount_minor,
        difference_minor: record.difference_minor,
        difference_reason: record.difference_reason,
        opened_by: record.opened_by,
        opened_by_name: record.opened_by_name,
        opened_at: record.opened_at,
        closed_by: record.closed_by,
        closed_by_name: record.closed_by_name,
        closed_at: record.closed_at,
    }
}

fn cash_movement_type(value: &str) -> CashMovementType {
    match value {
        "expense" => CashMovementType::Expense,
        _ => CashMovementType::Income,
    }
}

fn cash_movement_type_str(value: CashMovementType) -> &'static str {
    match value {
        CashMovementType::Income => "income",
        CashMovementType::Expense => "expense",
    }
}

fn cash_movement(record: CashMovementRecord) -> CashMovement {
    CashMovement {
        id: record.id,
        cash_session_id: record.cash_session_id,
        movement_type: cash_movement_type(&record.movement_type),
        category: record.category,
        amount_minor: record.amount_minor,
        payment_method: payment_method(&record.payment_method),
        evidence: record.evidence,
        authorized_by: record.authorized_by,
        authorized_by_name: record.authorized_by_name,
        created_at: record.created_at,
    }
}

fn new_catalog_price(input: CatalogPriceInput) -> NewCatalogPrice {
    NewCatalogPrice {
        amount_minor: input.amount_minor,
        tax_rate_bps: input.tax_rate_bps,
        price_includes_tax: input.price_includes_tax,
        effective_from: input.effective_from,
    }
}

fn catalog_api_error(error: DataError) -> ApiError {
    match error {
        DataError::CatalogProductAlreadyExists => {
            ApiError::invalid("ya existe un producto con ese nombre")
        }
        DataError::CatalogSkuAlreadyExists => {
            ApiError::invalid("ya existe una variante con ese SKU")
        }
        DataError::CatalogBarcodeAlreadyExists => {
            ApiError::invalid("ya existe una variante con ese código de barras")
        }
        DataError::CatalogProductNotFound => {
            ApiError::new(ErrorCode::NotFound, "el producto no existe")
        }
        DataError::CatalogVariantNotFound => {
            ApiError::new(ErrorCode::NotFound, "la variante no existe")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo guardar el catálogo")
            .with_details(other.to_string()),
    }
}

fn cash_api_error(error: DataError) -> ApiError {
    match error {
        DataError::CashSessionAlreadyOpen => {
            ApiError::invalid("ya hay un turno de caja abierto en este gimnasio")
        }
        DataError::CashSessionNotFound => {
            ApiError::new(ErrorCode::NotFound, "el turno de caja no existe")
        }
        DataError::CashSessionRequired => {
            ApiError::invalid("abre la caja antes de registrar una venta")
        }
        DataError::CashSessionNotOpen => ApiError::invalid("el turno de caja ya está cerrado"),
        DataError::CashMovementNotFound => {
            ApiError::new(ErrorCode::NotFound, "el movimiento de caja no existe")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo guardar el turno de caja")
            .with_details(other.to_string()),
    }
}

fn inventory_api_error(error: DataError) -> ApiError {
    match error {
        DataError::CatalogVariantNotFound => {
            ApiError::new(ErrorCode::NotFound, "la variante no existe")
        }
        DataError::InventoryMovementNotFound => {
            ApiError::new(ErrorCode::NotFound, "el movimiento de inventario no existe")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo guardar el inventario")
            .with_details(other.to_string()),
    }
}

fn sale_api_error(error: DataError) -> ApiError {
    match error {
        DataError::CatalogProductNotFound => {
            ApiError::new(ErrorCode::NotFound, "el producto no existe")
        }
        DataError::CatalogVariantNotFound => {
            ApiError::new(ErrorCode::NotFound, "la variante no existe")
        }
        DataError::CashSessionRequired => {
            ApiError::invalid("abre la caja antes de registrar una venta o devolución")
        }
        DataError::SaleIdempotencyConflict => ApiError::invalid(
            "la clave de idempotencia ya se usó con otra operación; inicia un intento nuevo",
        ),
        DataError::SaleNotFound => ApiError::new(ErrorCode::NotFound, "la venta no existe"),
        DataError::SaleReturnNotFound => {
            ApiError::new(ErrorCode::NotFound, "la devolución no existe")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(
            ErrorCode::Storage,
            "no se pudo registrar la operación de venta",
        )
        .with_details(other.to_string()),
    }
}

fn audit_result(value: &str) -> AuditResult {
    match value {
        "denied" => AuditResult::Denied,
        "failed" => AuditResult::Failed,
        _ => AuditResult::Allowed,
    }
}

fn audit_summary(record: gym_data::AuditEventRecord) -> AuditEventSummary {
    AuditEventSummary {
        id: record.id,
        sequence: record.sequence,
        actor_id: record.actor_id,
        actor_name: record.actor_name,
        action: record.action,
        resource: record.resource,
        result: audit_result(&record.result),
        origin: record.origin,
        correlation_id: record.correlation_id,
        created_at: record.created_at,
    }
}

fn staff_api_error(error: DataError) -> ApiError {
    match error {
        DataError::UserAlreadyExists => ApiError::invalid("ya existe una persona con ese nombre"),
        DataError::StaffNotFound => ApiError::new(ErrorCode::NotFound, "el empleado no existe"),
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo guardar al empleado")
            .with_details(other.to_string()),
    }
}

fn members_api_error(error: DataError) -> ApiError {
    match error {
        DataError::MembershipPlanAlreadyExists => {
            ApiError::invalid("ya existe un plan de membresía con ese nombre")
        }
        DataError::MembershipPlanNotFound => {
            ApiError::new(ErrorCode::NotFound, "el plan de membresía no existe")
        }
        DataError::MemberNotFound => ApiError::new(ErrorCode::NotFound, "el socio no existe"),
        DataError::MembershipAlreadyCurrent => {
            ApiError::invalid("el socio ya tiene una membresía vigente")
        }
        DataError::MembershipNotFound => {
            ApiError::new(ErrorCode::NotFound, "la membresía no existe")
        }
        DataError::MembershipStateConflict => {
            ApiError::invalid("la membresía no permite esa operación en su estado actual")
        }
        DataError::ProspectNotFound => ApiError::new(ErrorCode::NotFound, "el prospecto no existe"),
        DataError::ProspectAlreadyConverted => {
            ApiError::invalid("el prospecto ya fue convertido en socio")
        }
        DataError::MembershipChargeNotFound => {
            ApiError::new(ErrorCode::NotFound, "el cargo no existe")
        }
        DataError::MembershipChargeNotPayable => {
            ApiError::invalid("el cargo no está pendiente de pago")
        }
        DataError::MembershipPaymentExceedsBalance => {
            ApiError::invalid("el pago supera el saldo pendiente del cargo")
        }
        DataError::MembershipPaymentIdempotencyConflict => ApiError::invalid(
            "la clave de idempotencia ya se usó con otro pago; inicia un intento nuevo",
        ),
        DataError::MembershipPaymentNotFound => {
            ApiError::new(ErrorCode::NotFound, "el pago no existe")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(
            ErrorCode::Storage,
            "no se pudo guardar la operación de socios",
        )
        .with_details(other.to_string()),
    }
}

/// Fixed Argon2id hash used to keep unknown-user logins as slow as known ones.
fn dummy_password_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| {
        hash_password("not-a-real-user-password", OWNER_MIN_PASSWORD_CHARS)
            .unwrap_or_else(|_| String::new())
    })
}

fn setup_state(db: &Db) -> Result<SetupState, ApiError> {
    let organization = db.organization().map_err(|error| {
        ApiError::new(ErrorCode::Storage, "no se pudo leer la configuración local")
            .with_details(error.to_string())
    })?;
    let owner_ready = if organization.is_some() {
        db.has_owner().map_err(|error| {
            ApiError::new(ErrorCode::Storage, "no se pudo leer el personal local")
                .with_details(error.to_string())
        })?
    } else {
        false
    };
    Ok(SetupState {
        configured: organization.is_some(),
        gym_name: organization.map(|organization| organization.name),
        owner_ready,
    })
}

#[tauri::command]
fn get_setup_state(state: State<'_, AppState>) -> Result<SetupState, ApiError> {
    setup_state(state.db()?)
}

#[tauri::command]
fn bootstrap_owner(
    name: String,
    password: String,
    gym_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<RecoveryKit, ApiError> {
    let db = state.db()?;
    let name = normalize_person_name(&name)
        .map_err(|error| ApiError::invalid(format!("nombre del dueño inválido: {error}")))?;
    let password_hash = hash_password(&password, OWNER_MIN_PASSWORD_CHARS)
        .map_err(|error| ApiError::invalid(error.to_string()))?;
    let recovery_code = generate_recovery_code();
    let recovery_code_hash = hash_recovery_code(&recovery_code).map_err(|error| {
        ApiError::new(
            ErrorCode::Internal,
            "no se pudo crear el código de recuperación",
        )
        .with_details(error.to_string())
    })?;

    let organization = match db.organization().map_err(|error| {
        ApiError::new(ErrorCode::Storage, "no se pudo leer la configuración local")
            .with_details(error.to_string())
    })? {
        Some(organization) => organization,
        None => {
            let gym_name = gym_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ApiError::invalid("escribe el nombre del gimnasio"))?;
            db.create_organization(gym_name)
                .map_err(|error| match error {
                    DataError::OrganizationAlreadyExists => {
                        ApiError::invalid("este equipo ya está configurado para un gimnasio")
                    }
                    other => ApiError::new(ErrorCode::Storage, "no se pudo configurar el gimnasio")
                        .with_details(other.to_string()),
                })?;
            db.organization()
                .map_err(|error| {
                    ApiError::new(ErrorCode::Storage, "no se pudo leer la configuración local")
                        .with_details(error.to_string())
                })?
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::Internal,
                        "no se encontró el gimnasio recién creado",
                    )
                })?
        }
    };

    let owner = db
        .create_owner(
            &organization.id,
            &name,
            &password_hash,
            Some(&recovery_code_hash),
        )
        .map_err(|error| match error {
            DataError::OwnerAlreadyExists => {
                ApiError::invalid("este gimnasio ya tiene una cuenta de dueño")
            }
            DataError::UserAlreadyExists => {
                ApiError::invalid("ya existe una persona con ese nombre")
            }
            other => ApiError::new(ErrorCode::Storage, "no se pudo crear la cuenta del dueño")
                .with_details(other.to_string()),
        })?;

    let _ = db.record_audit_event(
        &organization.id,
        Some(&owner.id),
        "auth.owner_created",
        "staff_membership",
        "allowed",
        "tauri",
        None,
    );

    let session = gym_core::auth::Session {
        user_id: owner.id,
        organization_id: owner.organization_id,
        name: owner.name,
        role: Role::Owner,
    };
    let active = ActiveSession {
        session,
        last_seen: Instant::now(),
    };
    let summary = session_summary(&active);
    {
        let mut auth = state.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        auth.session = Some(active);
    }
    Ok(RecoveryKit {
        session: summary,
        recovery_code,
    })
}

#[tauri::command]
fn generate_owner_recovery(state: State<'_, AppState>) -> Result<RecoveryCode, ApiError> {
    let session = state.authorize(Permission::ManageSettings)?;
    if session.role != Role::Owner {
        return Err(ApiError::new(
            ErrorCode::Unauthorized,
            "solo la cuenta del dueño puede generar el código de recuperación",
        ));
    }

    let db = state.db()?;
    let recovery_code = generate_recovery_code();
    let recovery_code_hash = hash_recovery_code(&recovery_code).map_err(|error| {
        ApiError::new(
            ErrorCode::Internal,
            "no se pudo crear el código de recuperación",
        )
        .with_details(error.to_string())
    })?;
    db.set_owner_recovery_hash(
        &session.organization_id,
        &session.user_id,
        &recovery_code_hash,
    )
    .map_err(|error| {
        ApiError::new(
            ErrorCode::Storage,
            "no se pudo guardar el código de recuperación",
        )
        .with_details(error.to_string())
    })?;

    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "auth.owner_recovery_generated",
        "staff_membership",
        "allowed",
        "tauri",
        None,
    );
    Ok(RecoveryCode { recovery_code })
}

#[tauri::command]
fn recover_owner(
    recovery_code: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<RecoveryKit, ApiError> {
    const RECOVERY_KEY: &str = "__owner_recovery__";

    let db = state.db()?;
    let organization = db
        .organization()
        .map_err(|error| {
            ApiError::new(ErrorCode::Storage, "no se pudo leer la configuración local")
                .with_details(error.to_string())
        })?
        .ok_or_else(|| ApiError::invalid("configura el gimnasio antes de recuperar la cuenta"))?;
    let owner = db
        .owner_recovery(&organization.id)
        .map_err(|error| {
            ApiError::new(
                ErrorCode::Storage,
                "no se pudo consultar la cuenta del dueño",
            )
            .with_details(error.to_string())
        })?
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "la cuenta del dueño no existe"))?;

    let now = Instant::now();
    {
        let auth = state.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        if auth.is_blocked(RECOVERY_KEY, now) {
            return Err(ApiError::new(
                ErrorCode::Unauthorized,
                "demasiados intentos fallidos; espera 5 minutos",
            ));
        }
    }

    let valid = owner
        .recovery_code_hash
        .as_deref()
        .is_some_and(|hash| verify_recovery_code(hash, &recovery_code))
        || (owner.recovery_code_hash.is_none()
            && verify_password(dummy_password_hash(), &recovery_code));

    if !valid {
        let blocked = {
            let mut auth = state.auth.lock().map_err(|_| {
                ApiError::new(
                    ErrorCode::Internal,
                    "el estado de autenticación no está disponible",
                )
            })?;
            auth.record_failure(RECOVERY_KEY, now)
        };
        let _ = db.record_audit_event(
            &organization.id,
            None,
            "auth.owner_recovered",
            "staff_membership",
            "failed",
            "tauri",
            None,
        );
        return Err(ApiError::new(
            ErrorCode::Unauthorized,
            if blocked {
                "demasiados intentos fallidos; espera 5 minutos"
            } else {
                "código de recuperación incorrecto o no configurado"
            },
        ));
    }

    let password_hash = hash_password(&new_password, OWNER_MIN_PASSWORD_CHARS)
        .map_err(|error| ApiError::invalid(error.to_string()))?;
    let next_recovery_code = generate_recovery_code();
    let next_recovery_hash = hash_recovery_code(&next_recovery_code).map_err(|error| {
        ApiError::new(
            ErrorCode::Internal,
            "no se pudo crear el código de recuperación",
        )
        .with_details(error.to_string())
    })?;

    db.reset_owner_password(
        &organization.id,
        &owner.user_id,
        owner.recovery_code_hash.as_deref().unwrap_or_default(),
        &password_hash,
        &next_recovery_hash,
    )
    .map_err(|error| match error {
        DataError::RecoveryCodeConflict => ApiError::new(
            ErrorCode::Unauthorized,
            "el código de recuperación ya fue usado o reemplazado",
        ),
        other => ApiError::new(
            ErrorCode::Storage,
            "no se pudo actualizar la cuenta del dueño",
        )
        .with_details(other.to_string()),
    })?;

    {
        let mut auth = state.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        auth.clear_failures(RECOVERY_KEY);
        auth.session = Some(ActiveSession {
            session: gym_core::auth::Session {
                user_id: owner.user_id.clone(),
                organization_id: organization.id.clone(),
                name: owner.name.clone(),
                role: Role::Owner,
            },
            last_seen: Instant::now(),
        });
    }
    let _ = db.mark_login(&owner.user_id);
    let _ = db.record_audit_event(
        &organization.id,
        Some(&owner.user_id),
        "auth.owner_recovered",
        "staff_membership",
        "allowed",
        "tauri",
        None,
    );

    let active = ActiveSession {
        session: gym_core::auth::Session {
            user_id: owner.user_id,
            organization_id: organization.id,
            name: owner.name,
            role: Role::Owner,
        },
        last_seen: Instant::now(),
    };
    Ok(RecoveryKit {
        session: session_summary(&active),
        recovery_code: next_recovery_code,
    })
}

#[tauri::command]
fn login(
    name: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<SessionSummary, ApiError> {
    let db = state.db()?;
    let organization = db
        .organization()
        .map_err(|error| {
            ApiError::new(ErrorCode::Storage, "no se pudo leer la configuración local")
                .with_details(error.to_string())
        })?
        .ok_or_else(|| ApiError::invalid("configura el gimnasio antes de entrar"))?;

    let name = name.trim();
    if name.is_empty() || password.is_empty() {
        return Err(ApiError::new(
            ErrorCode::Unauthorized,
            "nombre o contraseña incorrectos",
        ));
    }
    let key = name.to_lowercase();
    let now = Instant::now();
    {
        let auth = state.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        if auth.is_blocked(&key, now) {
            return Err(ApiError::new(
                ErrorCode::Unauthorized,
                "demasiados intentos fallidos; espera 5 minutos",
            ));
        }
    }

    let credentials = db
        .find_user_credentials(&organization.id, name)
        .map_err(|error| {
            ApiError::new(ErrorCode::Storage, "no se pudo consultar el personal local")
                .with_details(error.to_string())
        })?;

    let valid = credentials
        .as_ref()
        .is_some_and(|credentials| verify_password(&credentials.password_hash, &password))
        || (credentials.is_none() && verify_password(dummy_password_hash(), &password));

    let Some(credentials) = credentials.filter(|_| valid) else {
        let blocked = {
            let mut auth = state.auth.lock().map_err(|_| {
                ApiError::new(
                    ErrorCode::Internal,
                    "el estado de autenticación no está disponible",
                )
            })?;
            auth.record_failure(&key, now)
        };
        let _ = db.record_audit_event(
            &organization.id,
            None,
            "auth.login",
            "session",
            "failed",
            "tauri",
            None,
        );
        return Err(ApiError::new(
            ErrorCode::Unauthorized,
            if blocked {
                "demasiados intentos fallidos; espera 5 minutos"
            } else {
                "nombre o contraseña incorrectos"
            },
        ));
    };

    let session = build_session(credentials)?;
    {
        let mut auth = state.auth.lock().map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "el estado de autenticación no está disponible",
            )
        })?;
        auth.clear_failures(&key);
        auth.session = Some(ActiveSession {
            session: session.clone(),
            last_seen: Instant::now(),
        });
    }
    let _ = db.mark_login(&session.user_id);
    let _ = db.record_audit_event(
        &organization.id,
        Some(&session.user_id),
        "auth.login",
        "session",
        "allowed",
        "tauri",
        None,
    );

    Ok(session_summary(&ActiveSession {
        session,
        last_seen: Instant::now(),
    }))
}

#[tauri::command]
fn logout(state: State<'_, AppState>) {
    let session = state
        .auth
        .lock()
        .ok()
        .and_then(|mut auth| auth.session.take())
        .map(|active| active.session);
    if let Some(session) = session {
        if let Ok(db) = state.db() {
            let _ = db.record_audit_event(
                &session.organization_id,
                Some(&session.user_id),
                "auth.logout",
                "session",
                "allowed",
                "tauri",
                None,
            );
        }
    }
}

#[tauri::command]
fn current_session(state: State<'_, AppState>) -> Result<Option<SessionSummary>, ApiError> {
    let mut auth = state.auth.lock().map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "el estado de autenticación no está disponible",
        )
    })?;
    Ok(auth.current_session())
}

#[tauri::command]
fn health(state: State<'_, AppState>) -> HealthReport {
    let (database, data_module, overall) = match state.db() {
        Ok(db) => match db.status() {
            Ok(status) => (Some(status), ModuleHealth::ok("data"), HealthStatus::Ok),
            Err(error) => (
                None,
                ModuleHealth::degraded("data", error.to_string()),
                HealthStatus::Degraded,
            ),
        },
        Err(error) => (
            None,
            ModuleHealth::degraded("data", error.message),
            HealthStatus::Down,
        ),
    };

    HealthReport {
        service: SERVICE_NAME.into(),
        version: CORE_VERSION.into(),
        contract_version: CONTRACT_VERSION.into(),
        status: overall,
        modules: vec![
            ModuleHealth::ok("core"),
            data_module,
            ModuleHealth::ok("backup"),
            ModuleHealth::ok("tauri"),
        ],
        database,
        checked_at: Utc::now().to_rfc3339(),
    }
}

const ACCESS_AGGREGATE: &str = "access_decision";

fn storage_error(error: DataError) -> ApiError {
    ApiError::new(ErrorCode::Storage, "no se pudo guardar el evento de acceso")
        .with_details(error.to_string())
}

#[tauri::command]
fn simulate_access(
    request: AccessRequest,
    state: State<'_, AppState>,
) -> Result<AccessDecision, ApiError> {
    let started = Instant::now();
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let organization_id = session.organization_id;

    let credential_id = request.credential_id.trim();
    let device_id = request.device_id.trim();
    let idempotency_key = request.idempotency_key.trim();
    let local = match db
        .access_rule_for_credential(&organization_id, credential_id)
        .map_err(access_api_error)?
    {
        Some(rule) => {
            let now = Utc::now();
            let today = now.format("%Y-%m-%d").to_string();
            let minute = i64::from(now.hour()) * 60 + i64::from(now.minute());
            decide_member_access(&MemberAccessContext {
                credential_status: &rule.credential_status,
                member_status: &rule.member_status,
                birth_date: rule.birth_date.as_deref(),
                membership_status: rule.membership_status.as_deref(),
                start_date: rule.start_date.as_deref(),
                end_date: rule.end_date.as_deref(),
                access_days: rule.access_days.as_deref(),
                access_start_minute: rule.access_start_minute,
                access_end_minute: rule.access_end_minute,
                min_age: rule.min_age,
                max_age: rule.max_age,
                today: &today,
                minute,
            })
        }
        None => decide_simulated_access(credential_id, device_id, idempotency_key)
            .map_err(|error| ApiError::invalid(error.to_string()))?,
    };

    let event_type = if local.allowed {
        "access.allowed"
    } else {
        "access.denied"
    };
    let payload_hash = sha256_hex(format!("{credential_id}\n{device_id}"));
    let recorded = db
        .record_module_event(
            &organization_id,
            "outbound",
            event_type,
            ACCESS_AGGREGATE,
            idempotency_key,
            &payload_hash,
        )
        .map_err(storage_error)?;

    if recorded.event.payload_hash != payload_hash {
        return Err(ApiError::invalid(
            "la clave de idempotencia ya se usó para otra solicitud",
        ));
    }

    let allowed = recorded.event.event_type == "access.allowed";
    if recorded.inserted {
        let _ = db.record_audit_event(
            &organization_id,
            Some(&session.user_id),
            "access.simulated",
            ACCESS_AGGREGATE,
            if allowed { "allowed" } else { "denied" },
            "tauri",
            Some(&recorded.event.id),
        );
    }
    Ok(AccessDecision {
        event_id: recorded.event.id,
        allowed,
        reason: if recorded.inserted {
            local.reason.to_string()
        } else {
            "evento ya registrado; se devuelve el resultado original".to_string()
        },
        duplicate: !recorded.inserted,
        idempotency_key: recorded.event.idempotency_key,
        latency_ms: started.elapsed().as_millis() as u64,
        decided_at: recorded.event.created_at,
    })
}

#[tauri::command]
fn list_access_events(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<AccessEventSummary>, ApiError> {
    state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let events = db
        .list_module_events(ACCESS_AGGREGATE, limit.unwrap_or(20) as usize)
        .map_err(storage_error)?;
    Ok(events
        .into_iter()
        .map(|event| AccessEventSummary {
            event_id: event.id,
            allowed: event.event_type == "access.allowed",
            idempotency_key: event.idempotency_key,
            occurred_at: event.created_at,
        })
        .collect())
}

fn access_api_error(error: DataError) -> ApiError {
    match error {
        DataError::MemberNotFound => ApiError::new(ErrorCode::NotFound, "el socio no existe"),
        DataError::MemberCredentialNotFound => {
            ApiError::new(ErrorCode::NotFound, "la credencial no existe")
        }
        DataError::AccessIdempotencyConflict => {
            ApiError::invalid("la clave de idempotencia ya se usó con otro contenido")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo procesar el acceso")
            .with_details(other.to_string()),
    }
}

fn reports_api_error(error: DataError) -> ApiError {
    match error {
        DataError::ReportScheduleNotFound => {
            ApiError::new(ErrorCode::NotFound, "la programación de reportes no existe")
        }
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo procesar el reporte")
            .with_details(other.to_string()),
    }
}

fn member_credential_contract(record: MemberCredentialRecord) -> MemberCredential {
    MemberCredential {
        id: record.id,
        member_id: record.member_id,
        member_number: record.member_number,
        member_name: record.member_name,
        code: record.code,
        status: record.status,
        revision: record.revision,
        issued_at: record.issued_at,
        revoked_at: record.revoked_at,
    }
}

fn access_snapshot_contract(record: AccessSnapshotRecord) -> AccessSnapshot {
    AccessSnapshot {
        revision: record.revision,
        generated_at: record.generated_at,
        member_count: record.member_count,
        credential_count: record.credential_count,
        checksum: record.checksum,
        signature: record.signature,
        payload_json: record.payload_json,
    }
}

fn access_event_contract(record: AccessEventRecord) -> AccessEvent {
    AccessEvent {
        id: record.id,
        credential_code: record.credential_code.unwrap_or_default(),
        member_id: record.member_id,
        member_number: record.member_number,
        member_name: record.member_name,
        device_id: record.device_id,
        zone: record.zone,
        allowed: record.allowed,
        reason: record.reason,
        snapshot_revision: record.snapshot_revision,
        idempotency_key: record.idempotency_key,
        payload_hash: record.payload_hash,
        reconciliation_status: record.reconciliation_status,
        occurred_at: record.occurred_at,
    }
}

fn attendance_hour_contract(record: gym_data::AttendanceHourRecord) -> AttendanceHour {
    AttendanceHour {
        hour: record.hour,
        visits: record.visits,
        unique_members: record.unique_members,
    }
}

fn attendance_report_contract(record: AttendanceReportRecord) -> AttendanceReport {
    AttendanceReport {
        from: record.from,
        to: record.to,
        generated_at: Utc::now().to_rfc3339(),
        total_visits: record.total_visits,
        unique_members: record.unique_members,
        peak_hour: record.peak_hour,
        low_demand_hours: record
            .low_demand_hours
            .into_iter()
            .map(attendance_hour_contract)
            .collect(),
        hours: record
            .hours
            .into_iter()
            .map(attendance_hour_contract)
            .collect(),
        days: record
            .days
            .into_iter()
            .map(|day| AttendanceDay {
                date: day.date,
                visits: day.visits,
                unique_members: day.unique_members,
            })
            .collect(),
        plans: record
            .plans
            .into_iter()
            .map(|plan| AttendancePlan {
                plan_id: plan.plan_id,
                plan_name: Some(plan.plan_name),
                visits: plan.visits,
                unique_members: plan.unique_members,
            })
            .collect(),
        zones: record
            .zones
            .into_iter()
            .map(|zone| AttendanceZone {
                zone: Some(zone.zone),
                visits: zone.visits,
                unique_members: zone.unique_members,
            })
            .collect(),
        members: record
            .members
            .into_iter()
            .map(|member| AttendanceMember {
                member_id: member.member_id,
                member_number: member.member_number,
                member_name: member.member_name,
                visits: member.visits,
                last_visit_at: member.last_visit_at,
            })
            .collect(),
    }
}

fn report_schedule_contract(record: ReportScheduleRecord) -> ReportSchedule {
    ReportSchedule {
        id: record.id,
        report_type: record.report_type,
        cadence: record.cadence,
        lookback_days: record.lookback_days,
        enabled: record.enabled,
        next_run_at: record.next_run_at,
        last_run_at: record.last_run_at,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn report_run_contract(record: ReportRunRecord) -> ReportRun {
    ReportRun {
        id: record.id,
        schedule_id: record.schedule_id,
        report_type: record.report_type,
        period_from: record.period_from,
        period_to: record.period_to,
        generated_at: record.generated_at,
        file_name: record.file_name,
        checksum: record.checksum,
        status: record.status,
        error: record.error,
    }
}

fn new_access_event(input: AccessEventInput) -> gym_data::NewAccessEvent {
    gym_data::NewAccessEvent {
        credential_code: input.credential_code,
        device_id: input.device_id,
        zone: input.zone,
        allowed: input.allowed,
        reason: input.reason,
        snapshot_revision: input.snapshot_revision,
        occurred_at: input.occurred_at,
        idempotency_key: input.idempotency_key,
    }
}

#[tauri::command]
fn publish_access_snapshot(state: State<'_, AppState>) -> Result<AccessSnapshot, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.publish_access_snapshot(&session.organization_id, state.key.as_bytes())
        .map(access_snapshot_contract)
        .map_err(access_api_error)
}

#[tauri::command]
fn latest_access_snapshot(state: State<'_, AppState>) -> Result<Option<AccessSnapshot>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.latest_access_snapshot(&session.organization_id)
        .map(|snapshot| snapshot.map(access_snapshot_contract))
        .map_err(access_api_error)
}

#[tauri::command]
fn list_member_credentials(
    member_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<MemberCredential>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.list_member_credentials(&session.organization_id, &member_id)
        .map(|records| {
            records
                .into_iter()
                .map(member_credential_contract)
                .collect()
        })
        .map_err(access_api_error)
}

#[tauri::command]
fn issue_member_credential(
    member_id: String,
    state: State<'_, AppState>,
) -> Result<MemberCredential, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.issue_member_credential(&session.organization_id, &member_id)
        .map(member_credential_contract)
        .map_err(access_api_error)
}

#[tauri::command]
fn revoke_member_credential(
    credential_id: String,
    state: State<'_, AppState>,
) -> Result<MemberCredential, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.revoke_member_credential(&session.organization_id, &credential_id)
        .map(member_credential_contract)
        .map_err(access_api_error)
}

#[tauri::command]
fn record_access_event(
    input: AccessEventInput,
    state: State<'_, AppState>,
) -> Result<AccessEvent, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    match db
        .record_access_event(&session.organization_id, new_access_event(input))
        .map_err(access_api_error)?
    {
        AccessEventWriteOutcome::Inserted(event) | AccessEventWriteOutcome::Duplicate(event) => {
            Ok(access_event_contract(event))
        }
        AccessEventWriteOutcome::Conflict => Err(ApiError::invalid(
            "la clave de idempotencia ya se usó con otro contenido",
        )),
    }
}

#[tauri::command]
fn list_access_event_log(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<AccessEvent>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.list_access_events(&session.organization_id, limit.unwrap_or(50) as usize)
        .map(|records| records.into_iter().map(access_event_contract).collect())
        .map_err(access_api_error)
}

#[tauri::command]
fn reconcile_access_events(
    input: AccessReconciliationInput,
    state: State<'_, AppState>,
) -> Result<AccessReconciliationSummary, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let events = input.events.into_iter().map(new_access_event).collect();
    let record = db
        .reconcile_access_events(&session.organization_id, input.snapshot_revision, events)
        .map_err(access_api_error)?;
    Ok(AccessReconciliationSummary {
        snapshot_revision: record.snapshot_revision,
        inserted: record.applied,
        duplicates: record.duplicates,
        stale: record.stale,
        conflicts: record
            .conflicts
            .into_iter()
            .map(|conflict| AccessConflict {
                idempotency_key: conflict.idempotency_key,
                reason: conflict.reason,
            })
            .collect(),
    })
}

#[tauri::command]
fn attendance_report(
    input: AttendanceInput,
    state: State<'_, AppState>,
) -> Result<AttendanceReport, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.attendance_report(&session.organization_id, &input.from, &input.to)
        .map(attendance_report_contract)
        .map_err(access_api_error)
}

#[tauri::command]
fn create_report_schedule(
    input: NewReportScheduleInput,
    state: State<'_, AppState>,
) -> Result<ReportSchedule, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.create_report_schedule(
        &session.organization_id,
        NewReportSchedule {
            cadence: input.cadence,
            lookback_days: input.lookback_days,
        },
    )
    .map(report_schedule_contract)
    .map_err(reports_api_error)
}

#[tauri::command]
fn list_report_schedules(state: State<'_, AppState>) -> Result<Vec<ReportSchedule>, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.list_report_schedules(&session.organization_id)
        .map(|records| records.into_iter().map(report_schedule_contract).collect())
        .map_err(reports_api_error)
}

#[tauri::command]
fn set_report_schedule_enabled(
    input: SetReportScheduleEnabledInput,
    state: State<'_, AppState>,
) -> Result<ReportSchedule, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.set_report_schedule_enabled(&session.organization_id, &input.schedule_id, input.enabled)
        .map(report_schedule_contract)
        .map_err(reports_api_error)
}

#[tauri::command]
fn list_report_runs(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<ReportRun>, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.list_report_runs(&session.organization_id, limit.unwrap_or(50) as usize)
        .map(|records| records.into_iter().map(report_run_contract).collect())
        .map_err(reports_api_error)
}

fn run_report_schedule(
    state: &AppState,
    organization_id: &str,
    user_id: &str,
    schedule: ReportScheduleRecord,
) -> Result<ReportRun, ApiError> {
    let db = state.db()?;
    let generated_at = Utc::now();
    let from = (generated_at - chrono::TimeDelta::days(schedule.lookback_days))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let to = generated_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let report = match db
        .financial_report(organization_id, &from, &to)
        .map(financial_report_contract)
    {
        Ok(report) => report,
        Err(error) => {
            let run = db
                .record_report_run(
                    organization_id,
                    Some(&schedule.id),
                    &schedule.report_type,
                    &from,
                    &to,
                    &to,
                    None,
                    "",
                    "failed",
                    Some(&error.to_string()),
                )
                .map_err(reports_api_error)?;
            return Ok(report_run_contract(run));
        }
    };
    let csv = financial_report_csv(&report);
    let checksum = sha256_hex(csv.as_bytes());
    let stamp = generated_at.format("%Y%m%dT%H%M%S%3fZ").to_string();
    let file_name = format!("reporte-programado-{}-{stamp}.csv", schedule.cadence);
    fs::create_dir_all(&state.paths.exports_dir).map_err(|error| {
        ApiError::new(
            ErrorCode::Storage,
            "no se pudo preparar la carpeta de exportaciones",
        )
        .with_details(error.to_string())
    })?;
    let path = state.paths.exports_dir.join(&file_name);
    fs::write(&path, csv.as_bytes()).map_err(|error| {
        ApiError::new(ErrorCode::Storage, "no se pudo escribir la exportación")
            .with_details(error.to_string())
    })?;
    let run = db
        .record_report_run(
            organization_id,
            Some(&schedule.id),
            &schedule.report_type,
            &from,
            &to,
            &to,
            Some(&file_name),
            &checksum,
            "completed",
            None,
        )
        .map_err(reports_api_error)?;
    let _ = db.record_audit_event(
        organization_id,
        Some(user_id),
        "report.scheduled",
        "financial_report",
        "allowed",
        "tauri",
        Some(&file_name),
    );
    Ok(report_run_contract(run))
}

#[tauri::command]
fn run_due_report_schedules(state: State<'_, AppState>) -> Result<Vec<ReportRun>, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    let now = Utc::now().to_rfc3339();
    let schedules = db
        .due_report_schedules(&session.organization_id, &now)
        .map_err(reports_api_error)?;
    let mut runs = Vec::with_capacity(schedules.len());
    for schedule in schedules {
        runs.push(run_report_schedule(
            &state,
            &session.organization_id,
            &session.user_id,
            schedule,
        )?);
    }
    Ok(runs)
}

#[tauri::command]
fn list_staff(state: State<'_, AppState>) -> Result<Vec<StaffSummary>, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    let db = state.db()?;
    let mut staff = db
        .list_staff(&session.organization_id)
        .map_err(staff_api_error)?;
    if session.role == Role::Manager {
        staff.retain(|member| matches!(member.role.as_str(), "cashier" | "accounting"));
    }
    Ok(staff.into_iter().map(staff_summary).collect())
}

#[tauri::command]
fn create_staff(
    name: String,
    role: StaffRole,
    password: String,
    state: State<'_, AppState>,
) -> Result<StaffSummary, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    let role = role_from_contract(role);
    ensure_staff_role_allowed(&session, None, role)?;

    let name = normalize_person_name(&name)
        .map_err(|error| ApiError::invalid(format!("nombre inválido: {error}")))?;
    let password_hash = hash_password(&password, STAFF_MIN_PASSWORD_CHARS)
        .map_err(|error| ApiError::invalid(error.to_string()))?;

    let db = state.db()?;
    let staff = db
        .create_staff(
            &session.organization_id,
            &name,
            &password_hash,
            role.as_str(),
        )
        .map_err(staff_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "staff.created",
        "staff_membership",
        "allowed",
        "tauri",
        None,
    );
    Ok(staff_summary(staff))
}

#[tauri::command]
fn update_staff(
    user_id: String,
    role: StaffRole,
    status: StaffStatus,
    state: State<'_, AppState>,
) -> Result<StaffSummary, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    if user_id == session.user_id {
        return Err(ApiError::invalid("no puedes modificar tu propia cuenta"));
    }

    let role = role_from_contract(role);
    let db = state.db()?;
    let current = db
        .list_staff(&session.organization_id)
        .map_err(staff_api_error)?
        .into_iter()
        .find(|member| member.user_id == user_id)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "el empleado no existe"))?;
    ensure_staff_role_allowed(&session, Some(&current.role), role)?;

    let staff = db
        .update_staff(
            &session.organization_id,
            &user_id,
            role.as_str(),
            staff_status_str(status),
        )
        .map_err(staff_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "staff.updated",
        "staff_membership",
        "allowed",
        "tauri",
        None,
    );
    Ok(staff_summary(staff))
}

#[tauri::command]
fn list_membership_plans(state: State<'_, AppState>) -> Result<Vec<MembershipPlan>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.list_membership_plans(&session.organization_id)
        .map_err(members_api_error)
        .map(|plans| plans.into_iter().map(membership_plan_summary).collect())
}

#[tauri::command]
fn create_membership_plan(
    input: CreateMembershipPlanInput,
    state: State<'_, AppState>,
) -> Result<MembershipPlan, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let plan = db
        .create_membership_plan(
            &session.organization_id,
            NewMembershipPlan {
                name: input.name,
                description: input.description,
                duration_days: input.duration_days,
                price_minor: input.price_minor,
                grace_days: input.grace_days,
                access_days: input.access_days,
                access_start_minute: input.access_start_minute,
                access_end_minute: input.access_end_minute,
                min_age: input.min_age,
                max_age: input.max_age,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership_plan.created",
        "membership_plan",
        "allowed",
        "tauri",
        Some(&plan.id),
    );
    Ok(membership_plan_summary(plan))
}

#[tauri::command]
fn update_membership_plan(
    input: UpdateMembershipPlanInput,
    state: State<'_, AppState>,
) -> Result<MembershipPlan, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let plan = db
        .update_membership_plan(
            &session.organization_id,
            &input.plan_id,
            UpdateMembershipPlan {
                name: input.name,
                description: input.description,
                duration_days: input.duration_days,
                price_minor: input.price_minor,
                grace_days: input.grace_days,
                access_days: input.access_days,
                access_start_minute: input.access_start_minute,
                access_end_minute: input.access_end_minute,
                min_age: input.min_age,
                max_age: input.max_age,
                active: input.active,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership_plan.updated",
        "membership_plan",
        "allowed",
        "tauri",
        Some(&plan.id),
    );
    Ok(membership_plan_summary(plan))
}

#[tauri::command]
fn list_members(
    query: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Member>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.list_members(&session.organization_id, query.as_deref())
        .map_err(members_api_error)
        .map(|members| members.into_iter().map(member_summary).collect())
}

#[tauri::command]
fn get_member(member_id: String, state: State<'_, AppState>) -> Result<Member, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.member(&session.organization_id, &member_id)
        .map(member_summary)
        .map_err(members_api_error)
}

#[tauri::command]
fn create_member(input: CreateMemberInput, state: State<'_, AppState>) -> Result<Member, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let member = db
        .create_member(
            &session.organization_id,
            NewMember {
                first_name: input.first_name,
                last_name: input.last_name,
                phone: input.phone,
                email: input.email,
                birth_date: input.birth_date,
                notes: input.notes,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "member.created",
        "member",
        "allowed",
        "tauri",
        Some(&member.id),
    );
    Ok(member_summary(member))
}

#[tauri::command]
fn update_member(input: UpdateMemberInput, state: State<'_, AppState>) -> Result<Member, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let member = db
        .update_member(
            &session.organization_id,
            &input.member_id,
            UpdateMember {
                first_name: input.first_name,
                last_name: input.last_name,
                phone: input.phone,
                email: input.email,
                birth_date: input.birth_date,
                status: member_status_str(input.status).to_string(),
                notes: input.notes,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "member.updated",
        "member",
        "allowed",
        "tauri",
        Some(&member.id),
    );
    Ok(member_summary(member))
}

#[tauri::command]
fn list_prospects(
    query: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Prospect>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.list_prospects(&session.organization_id, query.as_deref())
        .map_err(members_api_error)
        .map(|prospects| prospects.into_iter().map(prospect_summary).collect())
}

#[tauri::command]
fn create_prospect(
    input: CreateProspectInput,
    state: State<'_, AppState>,
) -> Result<Prospect, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let prospect = db
        .create_prospect(
            &session.organization_id,
            NewProspect {
                name: input.name,
                phone: input.phone,
                email: input.email,
                source: input.source,
                notes: input.notes,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "prospect.created",
        "prospect",
        "allowed",
        "tauri",
        Some(&prospect.id),
    );
    Ok(prospect_summary(prospect))
}

#[tauri::command]
fn update_prospect(
    input: UpdateProspectInput,
    state: State<'_, AppState>,
) -> Result<Prospect, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let prospect = db
        .update_prospect(
            &session.organization_id,
            &input.prospect_id,
            UpdateProspect {
                name: input.name,
                phone: input.phone,
                email: input.email,
                source: input.source,
                status: prospect_status_str(input.status).to_string(),
                notes: input.notes,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "prospect.updated",
        "prospect",
        "allowed",
        "tauri",
        Some(&prospect.id),
    );
    Ok(prospect_summary(prospect))
}

#[tauri::command]
fn convert_prospect(prospect_id: String, state: State<'_, AppState>) -> Result<Member, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let member = db
        .convert_prospect(&session.organization_id, &prospect_id)
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "prospect.converted",
        "member",
        "allowed",
        "tauri",
        Some(&member.id),
    );
    Ok(member_summary(member))
}

#[tauri::command]
fn list_memberships(
    member_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Membership>, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.list_memberships(&session.organization_id, &member_id)
        .map_err(members_api_error)
        .map(|memberships| memberships.into_iter().map(membership_summary).collect())
}

#[tauri::command]
fn create_membership(
    input: CreateMembershipInput,
    state: State<'_, AppState>,
) -> Result<Membership, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let membership = db
        .create_membership(
            &session.organization_id,
            &session.user_id,
            &input.member_id,
            &input.plan_id,
            input.start_date.as_deref(),
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership.created",
        "membership",
        "allowed",
        "tauri",
        Some(&membership.id),
    );
    Ok(membership_summary(membership))
}

#[tauri::command]
fn renew_membership(
    input: RenewMembershipInput,
    state: State<'_, AppState>,
) -> Result<Membership, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let membership = db
        .renew_membership(
            &session.organization_id,
            &session.user_id,
            &input.membership_id,
            input.plan_id.as_deref(),
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership.renewed",
        "membership",
        "allowed",
        "tauri",
        Some(&membership.id),
    );
    Ok(membership_summary(membership))
}

#[tauri::command]
fn pause_membership(
    input: PauseMembershipInput,
    state: State<'_, AppState>,
) -> Result<Membership, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let membership = db
        .pause_membership(
            &session.organization_id,
            &session.user_id,
            &input.membership_id,
            &input.until_date,
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership.paused",
        "membership",
        "allowed",
        "tauri",
        Some(&membership.id),
    );
    Ok(membership_summary(membership))
}

#[tauri::command]
fn resume_membership(
    input: ResumeMembershipInput,
    state: State<'_, AppState>,
) -> Result<Membership, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let membership = db
        .resume_membership(
            &session.organization_id,
            &session.user_id,
            &input.membership_id,
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership.resumed",
        "membership",
        "allowed",
        "tauri",
        Some(&membership.id),
    );
    Ok(membership_summary(membership))
}

#[tauri::command]
fn cancel_membership(
    input: CancelMembershipInput,
    state: State<'_, AppState>,
) -> Result<Membership, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let membership = db
        .cancel_membership(
            &session.organization_id,
            &session.user_id,
            &input.membership_id,
            &input.reason,
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership.cancelled",
        "membership",
        "allowed",
        "tauri",
        Some(&membership.id),
    );
    Ok(membership_summary(membership))
}

#[tauri::command]
fn member_account(
    member_id: String,
    state: State<'_, AppState>,
) -> Result<MemberAccount, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.member_account(&session.organization_id, &member_id)
        .map(member_account_summary)
        .map_err(members_api_error)
}

#[tauri::command]
fn record_membership_payment(
    input: RecordMembershipPaymentInput,
    state: State<'_, AppState>,
) -> Result<MembershipPayment, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    let payment = db
        .record_membership_payment(
            &session.organization_id,
            &session.user_id,
            NewMembershipPayment {
                charge_id: input.charge_id,
                amount_minor: input.amount_minor,
                payment_method: payment_method_str(input.payment_method).to_string(),
                reference: input.reference,
                idempotency_key: input.idempotency_key,
            },
        )
        .map_err(members_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "membership.payment_recorded",
        "membership_payment",
        "allowed",
        "tauri",
        Some(&payment.id),
    );
    Ok(membership_payment_summary(payment))
}

#[tauri::command]
fn membership_payment_receipt(
    payment_id: String,
    state: State<'_, AppState>,
) -> Result<MembershipPaymentReceipt, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.membership_payment_receipt(&session.organization_id, &payment_id)
        .map(membership_payment_receipt_summary)
        .map_err(members_api_error)
}

#[tauri::command]
fn membership_dashboard(state: State<'_, AppState>) -> Result<MembershipDashboard, ApiError> {
    let session = state.authorize(Permission::ManageMembers)?;
    let db = state.db()?;
    db.membership_dashboard(&session.organization_id)
        .map(membership_dashboard_summary)
        .map_err(members_api_error)
}

#[tauri::command]
fn list_catalog(state: State<'_, AppState>) -> Result<Vec<CatalogProduct>, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    db.list_catalog(&session.organization_id)
        .map_err(catalog_api_error)
        .map(|products| products.into_iter().map(catalog_product).collect())
}

#[tauri::command]
fn create_catalog_item(
    input: CreateCatalogItemInput,
    state: State<'_, AppState>,
) -> Result<CatalogProduct, ApiError> {
    let session = state.authorize(Permission::ManageCatalog)?;
    let db = state.db()?;
    let item = NewCatalogItem {
        name: input.name,
        category: input.category,
        variant_name: input.variant.name,
        sku: input.variant.sku,
        barcode: input.variant.barcode,
        unit: catalog_unit_str(input.variant.unit).to_string(),
        cost_minor: input.variant.cost_minor,
        price: new_catalog_price(input.variant.price),
    };
    let product = db
        .create_catalog_item(&session.organization_id, &session.user_id, item)
        .map_err(catalog_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "catalog.product_created",
        "catalog_product",
        "allowed",
        "tauri",
        Some(&product.id),
    );
    Ok(catalog_product(product))
}

#[tauri::command]
fn add_catalog_variant(
    input: AddCatalogVariantInput,
    state: State<'_, AppState>,
) -> Result<CatalogProduct, ApiError> {
    let session = state.authorize(Permission::ManageCatalog)?;
    let db = state.db()?;
    let sku = input.variant.sku.clone();
    let variant = NewCatalogVariant {
        name: input.variant.name,
        sku: input.variant.sku,
        barcode: input.variant.barcode,
        unit: catalog_unit_str(input.variant.unit).to_string(),
        cost_minor: input.variant.cost_minor,
        price: new_catalog_price(input.variant.price),
    };
    let product = db
        .add_catalog_variant(
            &session.organization_id,
            &session.user_id,
            &input.product_id,
            variant,
        )
        .map_err(catalog_api_error)?;
    let variant_id = product
        .variants
        .iter()
        .find(|variant| variant.sku == sku)
        .map(|variant| variant.id.clone());
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "catalog.variant_created",
        "catalog_variant",
        "allowed",
        "tauri",
        variant_id.as_deref(),
    );
    Ok(catalog_product(product))
}

#[tauri::command]
fn update_catalog_product(
    input: UpdateCatalogProductInput,
    state: State<'_, AppState>,
) -> Result<CatalogProduct, ApiError> {
    let session = state.authorize(Permission::ManageCatalog)?;
    let db = state.db()?;
    let update = NewCatalogProductUpdate {
        name: input.name,
        category: input.category,
        status: catalog_status_str(input.status).to_string(),
    };
    let product = db
        .update_catalog_product(&session.organization_id, &input.product_id, update)
        .map_err(catalog_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "catalog.product_updated",
        "catalog_product",
        "allowed",
        "tauri",
        Some(&product.id),
    );
    Ok(catalog_product(product))
}

#[tauri::command]
fn update_catalog_variant(
    input: UpdateCatalogVariantInput,
    state: State<'_, AppState>,
) -> Result<CatalogProduct, ApiError> {
    let session = state.authorize(Permission::ManageCatalog)?;
    let db = state.db()?;
    let update = NewCatalogVariantUpdate {
        name: input.name,
        sku: input.sku,
        barcode: input.barcode,
        unit: catalog_unit_str(input.unit).to_string(),
        cost_minor: input.cost_minor,
        status: catalog_status_str(input.status).to_string(),
    };
    let product = db
        .update_catalog_variant(&session.organization_id, &input.variant_id, update)
        .map_err(catalog_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "catalog.variant_updated",
        "catalog_variant",
        "allowed",
        "tauri",
        Some(&input.variant_id),
    );
    Ok(catalog_product(product))
}

#[tauri::command]
fn add_catalog_price(
    input: AddCatalogPriceInput,
    state: State<'_, AppState>,
) -> Result<CatalogProduct, ApiError> {
    let session = state.authorize(Permission::ManageCatalog)?;
    let db = state.db()?;
    let product = db
        .add_catalog_price(
            &session.organization_id,
            &session.user_id,
            &input.variant_id,
            new_catalog_price(input.price),
        )
        .map_err(catalog_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "catalog.price_added",
        "catalog_variant",
        "allowed",
        "tauri",
        Some(&input.variant_id),
    );
    Ok(catalog_product(product))
}

#[tauri::command]
fn list_sales(state: State<'_, AppState>) -> Result<Vec<Sale>, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    db.list_sales(&session.organization_id, 50)
        .map_err(catalog_api_error)
        .map(|sales| sales.into_iter().map(sale).collect())
}

#[tauri::command]
fn create_sale(input: CreateSaleInput, state: State<'_, AppState>) -> Result<Sale, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let new_sale = NewSale {
        items: input
            .items
            .into_iter()
            .map(|item| NewSaleItem {
                variant_id: item.variant_id,
                quantity: item.quantity,
            })
            .collect(),
        payment_method: payment_method_str(input.payment_method).to_string(),
        cash_received_minor: input.cash_received_minor,
        payment_reference: input.payment_reference,
        idempotency_key: input.idempotency_key,
    };
    let record = db
        .create_sale(&session.organization_id, &session.user_id, new_sale)
        .map_err(sale_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "sale.completed",
        "sale",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(sale(record))
}

#[tauri::command]
fn list_sale_returns(
    sale_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<SaleReturn>, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    db.list_sale_returns(&session.organization_id, sale_id.as_deref(), 50)
        .map_err(sale_api_error)
        .map(|returns| returns.into_iter().map(sale_return).collect())
}

#[tauri::command]
fn create_sale_return(
    input: CreateSaleReturnInput,
    state: State<'_, AppState>,
) -> Result<SaleReturn, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let record = db
        .create_sale_return(
            &session.organization_id,
            &session.user_id,
            NewSaleReturn {
                sale_id: input.sale_id,
                reason: input.reason,
                items: input
                    .items
                    .into_iter()
                    .map(|item| NewSaleReturnItem {
                        sale_item_id: item.sale_item_id,
                        quantity: item.quantity,
                    })
                    .collect(),
                payment_reference: input.payment_reference,
                idempotency_key: input.idempotency_key,
            },
        )
        .map_err(sale_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "sale.returned",
        "sale_return",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(sale_return(record))
}

#[tauri::command]
fn list_payment_charges(
    input: ListPaymentChargesInput,
    state: State<'_, AppState>,
) -> Result<Vec<PaymentCharge>, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.list_payment_charges(
        &session.organization_id,
        input.sale_id.as_deref(),
        input.external_reference.as_deref(),
        input.limit.unwrap_or(50),
    )
    .map_err(sale_api_error)
    .map(|charges| charges.into_iter().map(payment_charge).collect())
}

#[tauri::command]
fn list_inventory(state: State<'_, AppState>) -> Result<Vec<InventoryItem>, ApiError> {
    let session = state.authorize(Permission::ManageInventory)?;
    let db = state.db()?;
    db.list_inventory(&session.organization_id)
        .map_err(inventory_api_error)
        .map(|items| items.into_iter().map(inventory_item).collect())
}

#[tauri::command]
fn list_inventory_movements(
    variant_id: Option<String>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<InventoryMovement>, ApiError> {
    let session = state.authorize(Permission::ManageInventory)?;
    let db = state.db()?;
    db.list_inventory_movements(
        &session.organization_id,
        variant_id.as_deref(),
        limit.unwrap_or(50),
    )
    .map_err(inventory_api_error)
    .map(|movements| movements.into_iter().map(inventory_movement).collect())
}

#[tauri::command]
fn record_inventory_entry(
    input: RecordInventoryEntryInput,
    state: State<'_, AppState>,
) -> Result<InventoryMovement, ApiError> {
    let session = state.authorize(Permission::ManageInventory)?;
    let db = state.db()?;
    let record = db
        .record_inventory_entry(
            &session.organization_id,
            &session.user_id,
            NewInventoryEntry {
                variant_id: input.variant_id,
                quantity: input.quantity,
                unit_cost_minor: input.unit_cost_minor,
                reference: input.reference,
                note: input.note,
            },
        )
        .map_err(inventory_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "inventory.entry",
        "inventory_movement",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(inventory_movement(record))
}

#[tauri::command]
fn adjust_inventory(
    input: AdjustInventoryInput,
    state: State<'_, AppState>,
) -> Result<InventoryMovement, ApiError> {
    let session = state.authorize(Permission::ManageInventory)?;
    let db = state.db()?;
    let record = db
        .adjust_inventory(
            &session.organization_id,
            &session.user_id,
            NewInventoryAdjustment {
                variant_id: input.variant_id,
                quantity_delta: input.quantity_delta,
                unit_cost_minor: input.unit_cost_minor,
                reason: input.reason,
            },
        )
        .map_err(inventory_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "inventory.adjusted",
        "inventory_movement",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(inventory_movement(record))
}

#[tauri::command]
fn update_inventory_threshold(
    input: UpdateInventoryThresholdInput,
    state: State<'_, AppState>,
) -> Result<InventoryItem, ApiError> {
    let session = state.authorize(Permission::ManageInventory)?;
    let db = state.db()?;
    let record = db
        .update_inventory_threshold(
            &session.organization_id,
            &session.user_id,
            &input.variant_id,
            input.min_stock,
        )
        .map_err(inventory_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "inventory.threshold.updated",
        "inventory_policy",
        "allowed",
        "tauri",
        Some(&record.variant_id),
    );
    Ok(inventory_item(record))
}

#[tauri::command]
fn current_cash_session(state: State<'_, AppState>) -> Result<Option<CashSession>, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    db.current_cash_session(&session.organization_id)
        .map_err(cash_api_error)
        .map(|record| record.map(cash_session))
}

#[tauri::command]
fn list_cash_sessions(state: State<'_, AppState>) -> Result<Vec<CashSession>, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.list_cash_sessions(&session.organization_id, 50)
        .map_err(cash_api_error)
        .map(|sessions| sessions.into_iter().map(cash_session).collect())
}

#[tauri::command]
fn open_cash_session(
    input: OpenCashSessionInput,
    state: State<'_, AppState>,
) -> Result<CashSession, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let record = db
        .open_cash_session(
            &session.organization_id,
            &session.user_id,
            input.opening_amount_minor,
        )
        .map_err(cash_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "cash.opened",
        "cash_session",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(cash_session(record))
}

#[tauri::command]
fn close_cash_session(
    input: CloseCashSessionInput,
    state: State<'_, AppState>,
) -> Result<CashSession, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let record = db
        .close_cash_session(
            &session.organization_id,
            &input.session_id,
            &session.user_id,
            input.counted_amount_minor,
            input.difference_reason.as_deref(),
        )
        .map_err(cash_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "cash.closed",
        "cash_session",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(cash_session(record))
}

#[tauri::command]
fn list_cash_movements(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CashMovement>, ApiError> {
    let session = state.authorize_any(&[Permission::OperatePos, Permission::ViewFinance])?;
    let db = state.db()?;
    db.list_cash_movements(&session.organization_id, &session_id, 100)
        .map_err(cash_api_error)
        .map(|movements| movements.into_iter().map(cash_movement).collect())
}

#[tauri::command]
fn record_cash_movement(
    input: RecordCashMovementInput,
    state: State<'_, AppState>,
) -> Result<CashMovement, ApiError> {
    let session = state.authorize(Permission::OperatePos)?;
    let db = state.db()?;
    let record = db
        .record_cash_movement(
            &session.organization_id,
            &session.user_id,
            NewCashMovement {
                cash_session_id: input.session_id,
                movement_type: cash_movement_type_str(input.movement_type).into(),
                category: input.category,
                amount_minor: input.amount_minor,
                payment_method: payment_method_str(input.payment_method).into(),
                evidence: input.evidence,
            },
        )
        .map_err(cash_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "cash.movement_recorded",
        "cash_movement",
        "allowed",
        "tauri",
        Some(&record.id),
    );
    Ok(cash_movement(record))
}

#[tauri::command]
fn create_backup(
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<BackupSummary, ApiError> {
    let session = state.authorize(Permission::ManageBackups)?;
    let db = state.db()?;
    let backup =
        write_backup(db, &state.paths.backups_dir, &passphrase).map_err(backup_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "backup.created",
        "backup",
        "allowed",
        "tauri",
        Some(&backup.id),
    );
    Ok(backup)
}

#[tauri::command]
fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupSummary>, ApiError> {
    state.authorize(Permission::ManageBackups)?;
    read_backups(&state.paths.backups_dir).map_err(backup_api_error)
}

#[tauri::command]
fn restore_backup(
    file_name: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<RestoreOutcome, ApiError> {
    state.authorize(Permission::ManageBackups)?;
    let candidate = Path::new(&file_name);
    if candidate.file_name().and_then(|value| value.to_str()) != Some(file_name.as_str()) {
        return Err(ApiError::invalid("nombre de backup inválido"));
    }

    let backup_path = state.paths.backups_dir.join(&file_name);
    if !backup_path.exists() {
        return Err(ApiError::new(
            ErrorCode::NotFound,
            format!("no existe el backup {file_name}"),
        ));
    }

    let db = state.db()?;
    db.close().map_err(|error| {
        ApiError::new(
            ErrorCode::Storage,
            "no se pudo cerrar la base para restaurar",
        )
        .with_details(error.to_string())
    })?;

    let outcome = match apply_restore(&backup_path, &state.paths.db_path, &passphrase, &state.key) {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = db.reopen(&state.key);
            return Err(backup_api_error(error));
        }
    };
    db.reopen(&state.key).map_err(|error| {
        ApiError::new(ErrorCode::Storage, "falló la reapertura tras restaurar")
            .with_details(error.to_string())
    })?;
    // The restored database may contain a different owner or revoked staff.
    state.clear_session();

    Ok(outcome)
}

fn csv_text(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    let needs_guard = value.starts_with(['=', '+', '-', '@', '\t', '\r']);
    let needs_quotes = escaped
        .chars()
        .any(|character| matches!(character, ',' | '"' | '\n' | '\r'));
    if needs_guard {
        format!("\"'{escaped}\"")
    } else if needs_quotes {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

fn financial_report_csv(report: &FinancialReport) -> String {
    let mut rows = vec![
        "Reporte financiero".to_string(),
        format!("Desde,{}", csv_text(&report.from)),
        format!("Hasta,{}", csv_text(&report.to)),
        format!("Generado,{}", csv_text(&report.generated_at)),
        format!("Moneda,{}", csv_text(&report.currency)),
        String::new(),
        "Resumen".to_string(),
        "Métrica,Valor".to_string(),
        format!("Ventas,{}", report.sales_count),
        format!("Venta bruta (centavos),{}", report.gross_sales_minor),
        format!("Impuestos (centavos),{}", report.tax_minor),
        format!("Devoluciones,{}", report.refunds_count),
        format!("Devoluciones (centavos),{}", report.refunded_minor),
        format!("Venta neta (centavos),{}", report.net_sales_minor),
        format!("Ticket promedio (centavos),{}", report.average_sale_minor),
        format!("Attach rate (bps),{}", report.attach_rate_bps),
        format!(
            "Artículos por venta (milésimas),{}",
            report.items_per_sale_milli
        ),
        format!("Costo de ventas (centavos),{}", report.cost_of_goods_minor),
        format!("Margen (centavos),{}", report.margin_minor),
        format!("Margen (bps),{}", report.margin_bps),
        format!(
            "Inventario inicial (centavos),{}",
            report.inventory_value_start_minor
        ),
        format!(
            "Inventario final (centavos),{}",
            report.inventory_value_end_minor
        ),
        format!("Rotación (bps),{}", report.inventory_turnover_bps),
        format!("Ajustes (unidades),{}", report.adjustments_units),
        format!("Ajustes (centavos),{}", report.adjustments_cost_minor),
        format!("Turnos de caja,{}", report.cash_sessions_count),
        format!("Turnos abiertos,{}", report.open_cash_sessions),
        format!("Turnos cerrados,{}", report.closed_cash_sessions),
        format!(
            "Diferencia de caja (centavos),{}",
            report.cash_difference_minor
        ),
        format!(
            "Diferencia absoluta de caja (centavos),{}",
            report.cash_difference_abs_minor
        ),
        String::new(),
        "Métodos de pago".to_string(),
        "Método,Ventas,Cantidad,Devoluciones,Cantidad,Neto".to_string(),
    ];
    for method in &report.payment_methods {
        rows.push(format!(
            "{},{},{},{},{},{}",
            csv_text(payment_method_str(method.payment_method)),
            method.gross_sales_minor,
            method.sales_count,
            method.refunded_minor,
            method.refunds_count,
            method.net_sales_minor
        ));
    }
    rows.push(String::new());
    rows.push("Productos (centavos)".to_string());
    rows.push(
        "SKU,Producto,Variante,Unidad,Vendidas,Devueltas,Netas,Venta bruta,Devoluciones,Venta neta,Costo,Margen"
            .to_string(),
    );
    for product in &report.product_sales {
        rows.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_text(&product.sku),
            csv_text(&product.product_name),
            csv_text(&product.variant_name),
            csv_text(&product.unit),
            product.sold_units,
            product.refunded_units,
            product.net_units,
            product.gross_sales_minor,
            product.refunded_minor,
            product.net_sales_minor,
            product.cost_minor,
            product.margin_minor
        ));
    }
    rows.push(String::new());
    rows.push("Ajustes de inventario (centavos)".to_string());
    rows.push("SKU,Producto,Variante,Unidades,Costo".to_string());
    for adjustment in &report.inventory_adjustments {
        rows.push(format!(
            "{},{},{},{},{}",
            csv_text(&adjustment.sku),
            csv_text(&adjustment.product_name),
            csv_text(&adjustment.variant_name),
            adjustment.units_delta,
            adjustment.cost_minor
        ));
    }
    rows.join("\r\n") + "\r\n"
}

fn report_api_error(error: DataError) -> ApiError {
    match error {
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo generar el reporte")
            .with_details(other.to_string()),
    }
}

#[tauri::command]
fn financial_report(
    input: FinancialReportInput,
    state: State<'_, AppState>,
) -> Result<FinancialReport, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    db.financial_report(&session.organization_id, &input.from, &input.to)
        .map_err(report_api_error)
        .map(financial_report_contract)
}

#[tauri::command]
fn export_financial_report_csv(
    input: FinancialReportInput,
    state: State<'_, AppState>,
) -> Result<FinancialReportExport, ApiError> {
    let session = state.authorize(Permission::ViewFinance)?;
    let db = state.db()?;
    let report = db
        .financial_report(&session.organization_id, &input.from, &input.to)
        .map_err(report_api_error)?;
    let report = financial_report_contract(report);
    let csv = financial_report_csv(&report);
    let generated_at = Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string();
    let file_name = format!(
        "reporte-{}-{}-{generated_at}.csv",
        report.from.get(..10).unwrap_or("inicio"),
        report.to.get(..10).unwrap_or("fin")
    );
    fs::create_dir_all(&state.paths.exports_dir).map_err(|error| {
        ApiError::new(
            ErrorCode::Storage,
            "no se pudo preparar la carpeta de exportaciones",
        )
        .with_details(error.to_string())
    })?;
    let path = state.paths.exports_dir.join(&file_name);
    fs::write(&path, csv.as_bytes()).map_err(|error| {
        ApiError::new(ErrorCode::Storage, "no se pudo escribir la exportación")
            .with_details(error.to_string())
    })?;
    if let Err(error) = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "report.exported",
        "financial_report",
        "allowed",
        "tauri",
        Some(&file_name),
    ) {
        let _ = fs::remove_file(&path);
        return Err(
            ApiError::new(ErrorCode::Storage, "no se pudo auditar la exportación")
                .with_details(error.to_string()),
        );
    }
    Ok(FinancialReportExport {
        file_name,
        path: path.to_string_lossy().into_owned(),
        bytes: csv.len() as u64,
    })
}

#[tauri::command]
fn list_audit_events(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<AuditEventSummary>, ApiError> {
    let session = state.authorize(Permission::ViewAudit)?;
    let db = state.db()?;
    let limit = i64::from(limit.unwrap_or(100).clamp(1, 200));
    db.list_audit_events(&session.organization_id, limit)
        .map_err(|error| {
            ApiError::new(ErrorCode::Storage, "no se pudo leer la auditoría")
                .with_details(error.to_string())
        })
        .map(|events| events.into_iter().map(audit_summary).collect())
}

#[tauri::command]
fn verify_audit_chain(state: State<'_, AppState>) -> Result<AuditIntegrity, ApiError> {
    let session = state.authorize(Permission::ViewAudit)?;
    let db = state.db()?;
    db.verify_audit_chain(&session.organization_id)
        .map(|integrity| AuditIntegrity {
            valid: integrity.valid,
            checked_events: integrity.checked_events,
            first_broken_sequence: integrity.first_broken_sequence,
        })
        .map_err(|error| {
            ApiError::new(ErrorCode::Storage, "no se pudo verificar la auditoría")
                .with_details(error.to_string())
        })
}

fn backup_api_error(error: gym_backup::BackupError) -> ApiError {
    let code = match error {
        gym_backup::BackupError::WeakPassphrase => ErrorCode::InvalidRequest,
        gym_backup::BackupError::InvalidPassphrase => ErrorCode::Unauthorized,
        gym_backup::BackupError::InvalidFormat(_) | gym_backup::BackupError::ChecksumMismatch => {
            ErrorCode::InvalidRequest
        }
        _ => ErrorCode::Backup,
    };
    ApiError::new(code, "operación de backup fallida").with_details(error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,gym_desktop_lib=debug")),
        )
        .try_init()
        .ok();

    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;

            let key = load_or_migrate_key(&data_dir).map_err(Box::<dyn std::error::Error>::from)?;

            let db_path = data_dir.join("fitness-gym.db");
            let db = Db::open(&db_path, &key)
                .map(Arc::new)
                .map_err(|error| error.to_string());

            let paths = AppPaths {
                backups_dir: data_dir.join("backups"),
                exports_dir: data_dir.join("exports"),
                db_path,
            };

            app.manage(AppState {
                db,
                key,
                paths,
                auth: Mutex::new(AuthState::default()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            get_setup_state,
            bootstrap_owner,
            generate_owner_recovery,
            recover_owner,
            login,
            logout,
            current_session,
            simulate_access,
            list_access_events,
            publish_access_snapshot,
            latest_access_snapshot,
            list_member_credentials,
            issue_member_credential,
            revoke_member_credential,
            record_access_event,
            list_access_event_log,
            reconcile_access_events,
            attendance_report,
            create_report_schedule,
            list_report_schedules,
            set_report_schedule_enabled,
            list_report_runs,
            run_due_report_schedules,
            create_backup,
            list_backups,
            restore_backup,
            list_staff,
            create_staff,
            update_staff,
            list_membership_plans,
            create_membership_plan,
            update_membership_plan,
            list_members,
            get_member,
            create_member,
            update_member,
            list_prospects,
            create_prospect,
            update_prospect,
            convert_prospect,
            list_memberships,
            create_membership,
            renew_membership,
            pause_membership,
            resume_membership,
            cancel_membership,
            member_account,
            record_membership_payment,
            membership_payment_receipt,
            membership_dashboard,
            list_catalog,
            create_catalog_item,
            add_catalog_variant,
            update_catalog_product,
            update_catalog_variant,
            add_catalog_price,
            list_sales,
            create_sale,
            list_sale_returns,
            create_sale_return,
            list_payment_charges,
            financial_report,
            export_financial_report_csv,
            list_inventory,
            list_inventory_movements,
            record_inventory_entry,
            adjust_inventory,
            update_inventory_threshold,
            current_cash_session,
            list_cash_sessions,
            open_cash_session,
            close_cash_session,
            list_cash_movements,
            record_cash_movement,
            list_audit_events,
            verify_audit_chain
        ])
        .run(tauri::generate_context!())
        .expect("error while running Fitness Gym");
}

#[cfg(test)]
mod security_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("reads source directory") {
            let path = entry.expect("reads directory entry").path();
            if path.is_dir() {
                collect_files(&path, files);
            } else if path.extension().and_then(|value| value.to_str()) == Some("tsx")
                || path.extension().and_then(|value| value.to_str()) == Some("ts")
            {
                files.push(path);
            }
        }
    }

    #[test]
    fn capability_surface_stays_allowlisted() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let capabilities =
            fs::read_to_string(root.join("capabilities/default.json")).expect("reads capabilities");
        let parsed: serde_json::Value =
            serde_json::from_str(&capabilities).expect("valid capabilities JSON");
        assert_eq!(parsed["permissions"], serde_json::json!(["core:default"]));
        assert!(!capabilities.contains("shell:"));
        assert!(!capabilities.contains("fs:"));

        let config = fs::read_to_string(root.join("tauri.conf.json")).expect("reads tauri config");
        let parsed: serde_json::Value = serde_json::from_str(&config).expect("valid tauri JSON");
        let csp = parsed["app"]["security"]["csp"]
            .as_str()
            .expect("production CSP exists");
        assert!(csp.starts_with("default-src 'self'"));
        assert!(!csp.contains("unsafe-eval"));
    }

    #[test]
    fn frontend_does_not_use_web_storage() {
        let frontend = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
        let mut files = Vec::new();
        collect_files(&frontend, &mut files);
        assert!(!files.is_empty());

        for file in files {
            let source = fs::read_to_string(&file).expect("reads frontend source");
            assert!(
                !source.contains("localStorage") && !source.contains("sessionStorage"),
                "{} uses web storage",
                file.display()
            );
        }
    }
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    fn active_session(role: Role) -> ActiveSession {
        ActiveSession {
            session: gym_core::auth::Session {
                user_id: "user-1".into(),
                organization_id: "org-1".into(),
                name: "Ana".into(),
                role,
            },
            last_seen: Instant::now(),
        }
    }

    #[test]
    fn five_failed_attempts_lock_the_name_for_five_minutes() {
        let mut auth = AuthState::default();
        let now = Instant::now();
        for _ in 0..MAX_LOGIN_ATTEMPTS - 1 {
            assert!(!auth.record_failure("ana", now));
        }
        assert!(auth.record_failure("ana", now));
        assert!(auth.is_blocked("ana", now));
        assert!(!auth.is_blocked("ana", now + Duration::from_secs(301)));
    }

    #[test]
    fn expired_session_is_rejected_and_cleared() {
        let mut auth = AuthState::default();
        let mut active = active_session(Role::Owner);
        active.last_seen = Instant::now() - Duration::from_secs(16 * 60);
        auth.session = Some(active);

        let error = auth
            .authorize(Permission::ManageSettings)
            .expect_err("expired session must fail");
        assert_eq!(error.code, ErrorCode::Unauthorized);
        assert!(auth.session.is_none());
    }

    #[test]
    fn cashier_session_cannot_manage_backups() {
        let mut auth = AuthState {
            session: Some(active_session(Role::Cashier)),
            ..AuthState::default()
        };

        assert!(auth.authorize(Permission::OperatePos).is_ok());
        assert!(auth.authorize(Permission::ManageBackups).is_err());
    }

    #[test]
    fn manager_creates_cashiers_and_accounting_only() {
        let manager = active_session(Role::Manager).session;

        ensure_staff_role_allowed(&manager, None, Role::Cashier)
            .expect("manager can create cashiers");
        ensure_staff_role_allowed(&manager, None, Role::Accounting)
            .expect("manager can create accounting");

        let error = ensure_staff_role_allowed(&manager, None, Role::Manager)
            .expect_err("manager cannot create another manager");
        assert_eq!(error.code, ErrorCode::Unauthorized);
    }

    #[test]
    fn manager_cannot_manage_managers_or_the_owner() {
        let manager = active_session(Role::Manager).session;

        let error = ensure_staff_role_allowed(&manager, Some("manager"), Role::Cashier)
            .expect_err("manager cannot edit another manager");
        assert_eq!(error.code, ErrorCode::Unauthorized);

        let error = ensure_staff_role_allowed(&manager, Some("owner"), Role::Cashier)
            .expect_err("manager cannot edit the owner");
        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }

    #[test]
    fn owner_creates_managers_but_never_another_owner() {
        let owner = active_session(Role::Owner).session;
        ensure_staff_role_allowed(&owner, None, Role::Manager).expect("owner can create managers");

        let error = ensure_staff_role_allowed(&owner, None, Role::Owner)
            .expect_err("owner cannot create another owner");
        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }
}
