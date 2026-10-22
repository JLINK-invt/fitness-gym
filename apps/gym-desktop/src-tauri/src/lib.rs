use chrono::Utc;
use gym_backup::{
    create_backup as write_backup, list_backups as read_backups, restore_backup as apply_restore,
};
use gym_contracts::{
    AccessDecision, AccessEventSummary, AccessRequest, ApiError, BackupSummary, ErrorCode,
    HealthReport, HealthStatus, ModuleHealth, RestoreOutcome, SessionSummary, SetupState,
    SiteStatus, SiteSummary, StaffRole, StaffStatus, StaffSummary, CONTRACT_VERSION,
};
use gym_core::{
    access::decide_simulated_access,
    auth::{
        hash_password, normalize_person_name, verify_password, IDLE_TIMEOUT_MINUTES,
        LOGIN_LOCK_MINUTES, MAX_LOGIN_ATTEMPTS, OWNER_MIN_PASSWORD_CHARS, STAFF_MIN_PASSWORD_CHARS,
    },
    rbac::{self, Permission, Role},
    CORE_VERSION, SERVICE_NAME,
};
use gym_data::{
    sha256_hex, DataError, Db, DbKey, EnvKeyProvider, FileKeyProvider, KeyProvider,
    KeyringKeyProvider,
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
        site_id: session.site_id.clone(),
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
        site_id: credentials.site_id,
        name: credentials.name,
        role,
    })
}

fn site_status(value: &str) -> SiteStatus {
    match value {
        "closed" => SiteStatus::Closed,
        _ => SiteStatus::Active,
    }
}

fn site_status_str(value: SiteStatus) -> &'static str {
    match value {
        SiteStatus::Active => "active",
        SiteStatus::Closed => "closed",
    }
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

fn site_summary(record: gym_data::SiteRecord) -> SiteSummary {
    SiteSummary {
        id: record.id,
        name: record.name,
        status: site_status(&record.status),
        created_at: record.created_at,
    }
}

fn staff_summary(record: gym_data::StaffRecord) -> StaffSummary {
    StaffSummary {
        user_id: record.user_id,
        name: record.name,
        role: role_from_str(&record.role)
            .map(staff_role)
            .unwrap_or(StaffRole::Cashier),
        site_id: record.site_id,
        site_name: record.site_name,
        status: staff_status(&record.status),
        last_login_at: record.last_login_at,
        created_at: record.created_at,
    }
}

fn site_api_error(error: DataError) -> ApiError {
    match error {
        DataError::SiteAlreadyExists => ApiError::invalid("ya existe una sede con ese nombre"),
        DataError::SiteNotFound => ApiError::new(ErrorCode::NotFound, "la sede no existe"),
        DataError::SiteHasActiveStaff => ApiError::invalid(
            "no puedes cerrar una sede con personal activo; reasigna o desactiva primero",
        ),
        DataError::InvalidValue(message) => ApiError::invalid(message),
        other => ApiError::new(ErrorCode::Storage, "no se pudo guardar la sede")
            .with_details(other.to_string()),
    }
}

fn staff_api_error(error: DataError) -> ApiError {
    match error {
        DataError::UserAlreadyExists => ApiError::invalid("ya existe una persona con ese nombre"),
        DataError::SiteNotFound => ApiError::invalid("selecciona una sede válida"),
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
) -> Result<SessionSummary, ApiError> {
    let db = state.db()?;
    let name = normalize_person_name(&name)
        .map_err(|error| ApiError::invalid(format!("nombre del dueño inválido: {error}")))?;
    let password_hash = hash_password(&password, OWNER_MIN_PASSWORD_CHARS)
        .map_err(|error| ApiError::invalid(error.to_string()))?;

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
        .create_owner(&organization.id, &name, &password_hash)
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
        None,
    );

    let session = gym_core::auth::Session {
        user_id: owner.id,
        organization_id: owner.organization_id,
        site_id: owner.site_id,
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
    Ok(summary)
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
        None,
    );

    Ok(session_summary(&ActiveSession {
        session,
        last_seen: Instant::now(),
    }))
}

#[tauri::command]
fn logout(state: State<'_, AppState>) {
    state.clear_session();
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
fn list_sites(state: State<'_, AppState>) -> Result<Vec<SiteSummary>, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    let db = state.db()?;
    let mut sites = db
        .list_sites(&session.organization_id)
        .map_err(site_api_error)?;
    if session.role == Role::Manager {
        sites.retain(|site| Some(&site.id) == session.site_id.as_ref());
    }
    Ok(sites.into_iter().map(site_summary).collect())
}

#[tauri::command]
fn create_site(name: String, state: State<'_, AppState>) -> Result<SiteSummary, ApiError> {
    let session = state.authorize(Permission::ManageSettings)?;
    let name = normalize_person_name(&name)
        .map_err(|error| ApiError::invalid(format!("nombre de sede inválido: {error}")))?;
    let db = state.db()?;
    let site = db
        .create_site(&session.organization_id, &name)
        .map_err(site_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "site.created",
        "site",
        "allowed",
        None,
    );
    Ok(site_summary(site))
}

#[tauri::command]
fn update_site(
    site_id: String,
    name: String,
    status: SiteStatus,
    state: State<'_, AppState>,
) -> Result<SiteSummary, ApiError> {
    let session = state.authorize(Permission::ManageSettings)?;
    let name = normalize_person_name(&name)
        .map_err(|error| ApiError::invalid(format!("nombre de sede inválido: {error}")))?;
    let db = state.db()?;
    let site = db
        .update_site(
            &session.organization_id,
            &site_id,
            &name,
            site_status_str(status),
        )
        .map_err(site_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "site.updated",
        "site",
        "allowed",
        None,
    );
    Ok(site_summary(site))
}

#[tauri::command]
fn list_staff(state: State<'_, AppState>) -> Result<Vec<StaffSummary>, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    let db = state.db()?;
    let mut staff = db
        .list_staff(&session.organization_id)
        .map_err(staff_api_error)?;
    if session.role == Role::Manager {
        staff.retain(|member| member.role != "owner" && member.site_id == session.site_id);
    }
    Ok(staff.into_iter().map(staff_summary).collect())
}

#[tauri::command]
fn create_staff(
    name: String,
    role: StaffRole,
    site_id: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<StaffSummary, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    let role = role_from_contract(role);
    if role == Role::Owner {
        return Err(ApiError::invalid(
            "el dueño se crea solo durante la configuración inicial",
        ));
    }
    if session.role == Role::Manager && role == Role::Manager {
        return Err(ApiError::new(
            ErrorCode::Unauthorized,
            "un encargado no puede crear otro encargado",
        ));
    }

    let site_id = if session.role == Role::Manager {
        session
            .site_id
            .clone()
            .ok_or_else(|| ApiError::invalid("tu cuenta no tiene una sede asignada"))?
    } else {
        let value = site_id.trim();
        if value.is_empty() {
            return Err(ApiError::invalid("selecciona una sede"));
        }
        value.to_string()
    };

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
            &site_id,
        )
        .map_err(staff_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "staff.created",
        "staff_membership",
        "allowed",
        None,
    );
    Ok(staff_summary(staff))
}

#[tauri::command]
fn update_staff(
    user_id: String,
    role: StaffRole,
    site_id: String,
    status: StaffStatus,
    state: State<'_, AppState>,
) -> Result<StaffSummary, ApiError> {
    let session = state.authorize(Permission::ManageStaff)?;
    if user_id == session.user_id {
        return Err(ApiError::invalid("no puedes modificar tu propia cuenta"));
    }

    let role = role_from_contract(role);
    if role == Role::Owner {
        return Err(ApiError::invalid(
            "la cuenta del dueño no se puede modificar",
        ));
    }

    let db = state.db()?;
    let current = db
        .list_staff(&session.organization_id)
        .map_err(staff_api_error)?
        .into_iter()
        .find(|member| member.user_id == user_id)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "el empleado no existe"))?;
    if current.role == "owner" {
        return Err(ApiError::invalid(
            "la cuenta del dueño no se puede modificar",
        ));
    }

    let site_id = if session.role == Role::Manager {
        if current.site_id != session.site_id || role == Role::Manager {
            return Err(ApiError::new(
                ErrorCode::Unauthorized,
                "solo puedes administrar cajeros y contabilidad de tu sede",
            ));
        }
        let own_site = session
            .site_id
            .clone()
            .ok_or_else(|| ApiError::invalid("tu cuenta no tiene una sede asignada"))?;
        if site_id != own_site {
            return Err(ApiError::new(
                ErrorCode::Unauthorized,
                "no puedes mover personal a otra sede",
            ));
        }
        own_site
    } else {
        let value = site_id.trim();
        if value.is_empty() {
            return Err(ApiError::invalid("selecciona una sede"));
        }
        value.to_string()
    };

    let staff = db
        .update_staff(
            &session.organization_id,
            &user_id,
            role.as_str(),
            &site_id,
            staff_status_str(status),
        )
        .map_err(staff_api_error)?;
    let _ = db.record_audit_event(
        &session.organization_id,
        Some(&session.user_id),
        "staff.updated",
        "staff_membership",
        "allowed",
        None,
    );
    Ok(staff_summary(staff))
}

#[tauri::command]
fn create_backup(
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<BackupSummary, ApiError> {
    state.authorize(Permission::ManageBackups)?;
    let db = state.db()?;
    write_backup(db, &state.paths.backups_dir, &passphrase).map_err(backup_api_error)
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
            login,
            logout,
            current_session,
            simulate_access,
            list_access_events,
            create_backup,
            list_backups,
            restore_backup,
            list_sites,
            create_site,
            update_site,
            list_staff,
            create_staff,
            update_staff
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
                site_id: None,
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
}
