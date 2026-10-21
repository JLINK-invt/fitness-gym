//! Encrypted snapshots and verified local restore.
//!
//! A snapshot is the SQLCipher database copied with `VACUUM INTO` and then
//! wrapped in an XChaCha20-Poly1305 envelope whose key is derived from the
//! operator passphrase with Argon2id. The passphrase is never stored.

mod error;
mod snapshot;

pub use error::BackupError;
pub use snapshot::{
    create_backup, list_backups, restore_backup, BACKUP_EXTENSION, BACKUP_FORMAT_VERSION,
};
