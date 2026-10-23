use chrono::Utc;
use gym_backup::{
    create_backup as write_backup, list_backups as read_backups, restore_backup as apply_restore,
};
use gym_contracts::{
    AccessDecision, AccessEventSummary, AccessRequest, AddCatalogPriceInput,
    AddCatalogVariantInput, ApiError, AuditEventSummary, AuditIntegrity, AuditResult,
    BackupSummary, CatalogPrice, CatalogPriceInput, CatalogProduct, CatalogStatus, CatalogUnit,
    CatalogVariant, CreateCatalogItemInput, CreateSaleInput, ErrorCode, HealthReport, HealthStatus,
    ModuleHealth, PaymentMethod, RecoveryCode, RecoveryKit, RestoreOutcome, Sale, SaleItem,
    SessionSummary, SetupState, StaffRole, StaffStatus, StaffSummary, UpdateCatalogProductInput,
    UpdateCatalogVariantInput, CONTRACT_VERSION,
};
use gym_core::{
    access::decide_simulated_access,
    auth::{
        generate_recovery_code, hash_password, hash_recovery_code, normalize_person_name,
        verify_password, verify_recovery_code, IDLE_TIMEOUT_MINUTES, LOGIN_LOCK_MINUTES,
        MAX_LOGIN_ATTEMPTS, OWNER_MIN_PASSWORD_CHARS, STAFF_MIN_PASSWORD_CHARS,
    },
    rbac::{self, Permission, Role},
    CORE_VERSION, SERVICE_NAME,
};
use gym_data::{
    sha256_hex, CatalogPriceRecord, CatalogProductRecord, CatalogVariantRecord, DataError, Db,
    DbKey, EnvKeyProvider, FileKeyProvider, KeyProvider, KeyringKeyProvider, NewCatalogItem,
    NewCatalogPrice, NewCatalogProductUpdate, NewCatalogVariant, NewCatalogVariantUpdate, NewSale,
    NewSaleItem, SaleItemRecord, SaleRecord,
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
        rbac::authorize(session.role, permission).map_err(|_| {
            ApiError::new(
                ErrorCode::Unauthorized,
                "tu rol no permite realizar esta acción",
            )
        })?;
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
    }
}

fn sale(record: SaleRecord) -> Sale {
    Sale {
        id: record.id,
        currency: record.currency,
        subtotal_minor: record.subtotal_minor,
        tax_minor: record.tax_minor,
        total_minor: record.total_minor,
        payment_method: payment_method(&record.payment_method),
        cash_received_minor: record.cash_received_minor,
        change_minor: record.change_minor,
        sold_by: record.sold_by,
        sold_by_name: record.sold_by_name,
        items: record.items.into_iter().map(sale_item).collect(),
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
    let local = decide_simulated_access(credential_id, device_id, idempotency_key)
        .map_err(|error| ApiError::invalid(error.to_string()))?;

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
    };
    let record = db
        .create_sale(&session.organization_id, &session.user_id, new_sale)
        .map_err(catalog_api_error)?;
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
            create_backup,
            list_backups,
            restore_backup,
            list_staff,
            create_staff,
            update_staff,
            list_catalog,
            create_catalog_item,
            add_catalog_variant,
            update_catalog_product,
            update_catalog_variant,
            add_catalog_price,
            list_sales,
            create_sale,
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
