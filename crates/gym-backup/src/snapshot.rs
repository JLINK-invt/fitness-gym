use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chrono::Utc;
use gym_contracts::{BackupSummary, RestoreOutcome};
use gym_data::{verify_encrypted_database, Db, DbKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::error::BackupError;

pub const BACKUP_EXTENSION: &str = "gymbak";
pub const BACKUP_FORMAT_VERSION: u32 = 1;

const MAGIC: &[u8; 8] = b"FGYMBAK1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const HEADER_LEN: usize = MAGIC.len() + SALT_LEN + NONCE_LEN;
const MIN_PASSPHRASE_LEN: usize = 8;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupMetadata {
    format_version: u32,
    id: String,
    file_name: String,
    created_at: String,
    size_bytes: u64,
    checksum: String,
    schema_version: i64,
    kdf: String,
    cipher: String,
}

pub fn create_backup(
    db: &Db,
    destination_dir: impl AsRef<Path>,
    passphrase: &str,
) -> Result<BackupSummary, BackupError> {
    validate_passphrase(passphrase)?;

    let destination_dir = destination_dir.as_ref();
    fs::create_dir_all(destination_dir)?;

    let status = db.status()?;
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now();
    let file_name = format!(
        "fitness-gym-{}-{}.{}",
        created_at.format("%Y%m%dT%H%M%SZ"),
        &id[..8],
        BACKUP_EXTENSION
    );

    let staging = tempfile::tempdir_in(destination_dir)?;
    let snapshot_path = staging.path().join("snapshot.db");
    db.vacuum_into(&snapshot_path)?;

    let plaintext = Zeroizing::new(fs::read(&snapshot_path)?);
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key(passphrase, &salt)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_slice()));
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), plaintext.as_slice())
        .map_err(|error| BackupError::Crypto(error.to_string()))?;

    let mut envelope = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    envelope.extend_from_slice(MAGIC);
    envelope.extend_from_slice(&salt);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&ciphertext);

    let final_path = destination_dir.join(&file_name);
    write_atomically(&final_path, &envelope)?;

    let checksum = sha256_hex(&envelope);
    let metadata = BackupMetadata {
        format_version: BACKUP_FORMAT_VERSION,
        id: id.clone(),
        file_name: file_name.clone(),
        created_at: created_at.to_rfc3339(),
        size_bytes: envelope.len() as u64,
        checksum: checksum.clone(),
        schema_version: status.schema_version,
        kdf: "argon2id".into(),
        cipher: "xchacha20poly1305".into(),
    };
    write_metadata(&final_path, &metadata)?;

    Ok(BackupSummary {
        id,
        file_name,
        created_at: created_at.to_rfc3339(),
        size_bytes: envelope.len() as u64,
        checksum,
        schema_version: status.schema_version,
    })
}

pub fn list_backups(destination_dir: impl AsRef<Path>) -> Result<Vec<BackupSummary>, BackupError> {
    let destination_dir = destination_dir.as_ref();
    if !destination_dir.exists() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    for entry in fs::read_dir(destination_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some(BACKUP_EXTENSION) {
            continue;
        }
        if let Ok(metadata) = read_metadata(&path) {
            summaries.push(BackupSummary {
                id: metadata.id,
                file_name: metadata.file_name,
                created_at: metadata.created_at,
                size_bytes: metadata.size_bytes,
                checksum: metadata.checksum,
                schema_version: metadata.schema_version,
            });
        }
    }

    summaries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(summaries)
}

/// Restores a snapshot over `target_db`. The caller must close the live
/// [`Db`] first so no file handle or WAL remains open during the swap.
pub fn restore_backup(
    backup_path: &Path,
    target_db: &Path,
    passphrase: &str,
    db_key: &DbKey,
) -> Result<RestoreOutcome, BackupError> {
    validate_passphrase(passphrase)?;

    let metadata = read_metadata(backup_path)?;
    if metadata.format_version != BACKUP_FORMAT_VERSION {
        return Err(BackupError::InvalidFormat(format!(
            "unsupported format version {}",
            metadata.format_version
        )));
    }

    let envelope = fs::read(backup_path)?;
    if sha256_hex(&envelope) != metadata.checksum {
        return Err(BackupError::ChecksumMismatch);
    }
    if envelope.len() <= HEADER_LEN || &envelope[..MAGIC.len()] != MAGIC {
        return Err(BackupError::InvalidFormat("bad snapshot header".into()));
    }

    let salt: [u8; SALT_LEN] = envelope[MAGIC.len()..MAGIC.len() + SALT_LEN]
        .try_into()
        .map_err(|_| BackupError::InvalidFormat("bad salt".into()))?;
    let nonce_start = MAGIC.len() + SALT_LEN;
    let nonce: [u8; NONCE_LEN] = envelope[nonce_start..nonce_start + NONCE_LEN]
        .try_into()
        .map_err(|_| BackupError::InvalidFormat("bad nonce".into()))?;
    let ciphertext = &envelope[HEADER_LEN..];

    let key = derive_key(passphrase, &salt)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_slice()));
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(XNonce::from_slice(&nonce), ciphertext)
            .map_err(|_| BackupError::InvalidPassphrase)?,
    );

    let parent = target_db.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = tempfile::tempdir_in(parent)?;
    let candidate = staging.path().join("restored.db");
    fs::write(&candidate, plaintext.as_slice())?;
    verify_encrypted_database(&candidate, db_key)?;

    swap_database(&candidate, target_db)?;

    Ok(RestoreOutcome {
        backup_id: metadata.id,
        restored_at: Utc::now().to_rfc3339(),
        schema_version: metadata.schema_version,
        integrity_ok: true,
    })
}

fn swap_database(candidate: &Path, target: &Path) -> Result<(), BackupError> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let staged = parent.join(format!(".restore-{}.db", Uuid::new_v4()));
    fs::rename(candidate, &staged)?;

    let rollback = parent.join(format!(".restore-old-{}.db", Uuid::new_v4()));
    let had_previous = target.exists();
    if had_previous {
        fs::rename(target, &rollback)?;
    }
    // The previous connection is expected to be closed; drop stale sidecars so
    // SQLite never attaches an old WAL to the restored file.
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(sidecar(target, suffix));
    }

    match fs::rename(&staged, target) {
        Ok(()) => {
            if had_previous {
                let _ = fs::remove_file(&rollback);
            }
            Ok(())
        }
        Err(error) => {
            if had_previous {
                let _ = fs::rename(&rollback, target);
            }
            let _ = fs::remove_file(&staged);
            Err(BackupError::Io(error))
        }
    }
}

fn sidecar(target: &Path, suffix: &str) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn validate_passphrase(passphrase: &str) -> Result<(), BackupError> {
    if passphrase.trim().chars().count() < MIN_PASSPHRASE_LEN {
        return Err(BackupError::WeakPassphrase);
    }
    Ok(())
}

fn derive_key(passphrase: &str, salt: &[u8; SALT_LEN]) -> Result<Zeroizing<[u8; 32]>, BackupError> {
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut_slice())
        .map_err(|error| BackupError::Crypto(error.to_string()))?;
    Ok(key)
}

fn metadata_path(backup_path: &Path) -> PathBuf {
    backup_path.with_extension(format!("{BACKUP_EXTENSION}.json"))
}

fn write_metadata(backup_path: &Path, metadata: &BackupMetadata) -> Result<(), BackupError> {
    let json = serde_json::to_vec_pretty(metadata)?;
    write_atomically(&metadata_path(backup_path), &json)
}

fn read_metadata(backup_path: &Path) -> Result<BackupMetadata, BackupError> {
    let path = metadata_path(backup_path);
    let bytes = fs::read(&path).map_err(|error| {
        BackupError::InvalidFormat(format!("missing metadata {}: {error}", path.display()))
    })?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), BackupError> {
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, DbKey, PathBuf, Db) {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db_path = dir.path().join("gym.db");
        let db = Db::open(&db_path, &key).expect("opens db");
        db.create_organization("Gimnasio Centro").expect("org");
        (dir, key, db_path, db)
    }

    #[test]
    fn backup_and_restore_round_trip() {
        let (dir, key, db_path, db) = setup();
        let backups_dir = dir.path().join("backups");

        let summary = create_backup(&db, &backups_dir, "clave-segura-123").expect("backup");
        drop(db);
        fs::remove_file(&db_path).expect("removes db");

        restore_backup(
            &backups_dir.join(&summary.file_name),
            &db_path,
            "clave-segura-123",
            &key,
        )
        .expect("restore");

        let restored = Db::open(&db_path, &key).expect("reopens db");
        let status = restored.status().expect("status");
        assert_eq!(status.organizations, 1);
        assert!(status.integrity_ok);
    }

    #[test]
    fn restore_replaces_a_closed_live_database() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db_path = dir.path().join("gym.db");
        let db = Db::open(&db_path, &key).expect("opens db");
        let backups_dir = dir.path().join("backups");
        let summary = create_backup(&db, &backups_dir, "clave-segura-123").expect("backup");

        db.create_organization("Sucursal Norte")
            .expect("org after backup");
        db.close().expect("closes db");
        restore_backup(
            &backups_dir.join(&summary.file_name),
            &db_path,
            "clave-segura-123",
            &key,
        )
        .expect("restore");
        db.reopen(&key).expect("reopens db");

        let status = db.status().expect("status");
        assert_eq!(status.organizations, 0);
        assert!(status.integrity_ok);
    }

    #[test]
    fn snapshot_does_not_leak_plaintext() {
        let (dir, _key, _db_path, db) = setup();
        let backups_dir = dir.path().join("backups");
        let summary = create_backup(&db, &backups_dir, "clave-segura-123").expect("backup");

        let bytes = fs::read(backups_dir.join(&summary.file_name)).expect("reads snapshot");
        assert!(!bytes.starts_with(b"SQLite format 3\0"));
        assert!(!bytes.windows(8).any(|window| window == b"Gimnasio"));
    }

    #[test]
    fn wrong_passphrase_cannot_restore() {
        let (dir, key, db_path, db) = setup();
        let backups_dir = dir.path().join("backups");
        let summary = create_backup(&db, &backups_dir, "clave-segura-123").expect("backup");

        let error = restore_backup(
            &backups_dir.join(&summary.file_name),
            &db_path,
            "clave-incorrecta",
            &key,
        )
        .expect_err("must fail");

        assert!(matches!(error, BackupError::InvalidPassphrase));
    }

    #[test]
    fn weak_passphrases_are_rejected() {
        let (_dir, _key, _db_path, db) = setup();
        let destination = tempfile::tempdir().expect("temp dir");
        let error = create_backup(&db, destination.path(), "corta").expect_err("must fail");
        assert!(matches!(error, BackupError::WeakPassphrase));
    }
}
