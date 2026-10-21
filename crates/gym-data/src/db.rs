use gym_contracts::DatabaseStatus;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::DataError;
use crate::key::DbKey;
use crate::migrations::MIGRATIONS;

pub type LocalDatabaseStatus = DatabaseStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationRecord {
    pub id: String,
    pub name: String,
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

            conn.execute(
                "INSERT INTO organizations (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, chrono::Utc::now().to_rfc3339()],
            )
            .map_err(DataError::from)
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
        assert_eq!(second_status.schema_version, 1);
        assert_eq!(second_status.organizations, 0);
        assert!(second_status.encrypted);
        assert!(second_status.integrity_ok);
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
    fn wrong_key_is_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("gym.db");
        let key = DbKey::generate();
        drop(Db::open(&path, &key).expect("opens"));

        let error = Db::open(&path, &DbKey::generate()).expect_err("must fail");
        assert!(matches!(error, DataError::InvalidKey(_)));
    }
}
