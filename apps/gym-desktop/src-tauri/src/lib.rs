use chrono::Utc;
use gym_backup::{
    create_backup as write_backup, list_backups as read_backups, restore_backup as apply_restore,
};
use gym_contracts::{
    AccessDecision, AccessEventSummary, AccessRequest, ApiError, BackupSummary, ErrorCode,
    HealthReport, HealthStatus, ModuleHealth, RestoreOutcome, CONTRACT_VERSION,
};
use gym_core::{access::decide_simulated_access, CORE_VERSION, SERVICE_NAME};
use gym_data::{
    sha256_hex, DataError, Db, DbKey, EnvKeyProvider, FileKeyProvider, KeyProvider,
    KeyringKeyProvider,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
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

struct AppState {
    db: Result<Arc<Db>, String>,
    key: DbKey,
    paths: AppPaths,
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

#[tauri::command]
fn create_organization(name: String, state: State<'_, AppState>) -> Result<String, ApiError> {
    let db = state.db()?;
    db.create_organization(&name).map_err(|error| match error {
        DataError::OrganizationAlreadyExists => {
            ApiError::invalid("este equipo ya está configurado para un gimnasio")
        }
        other => ApiError::new(ErrorCode::Storage, "no se pudo configurar el gimnasio")
            .with_details(other.to_string()),
    })
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
    let db = state.db()?;
    let organization = db
        .organization()
        .map_err(storage_error)?
        .ok_or_else(|| ApiError::invalid("configura el gimnasio antes de simular un acceso"))?;

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
            &organization.id,
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
fn create_backup(
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<BackupSummary, ApiError> {
    let db = state.db()?;
    write_backup(db, &state.paths.backups_dir, &passphrase).map_err(backup_api_error)
}

#[tauri::command]
fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupSummary>, ApiError> {
    read_backups(&state.paths.backups_dir).map_err(backup_api_error)
}

#[tauri::command]
fn restore_backup(
    file_name: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<RestoreOutcome, ApiError> {
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

            app.manage(AppState { db, key, paths });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            create_organization,
            simulate_access,
            list_access_events,
            create_backup,
            list_backups,
            restore_backup
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
