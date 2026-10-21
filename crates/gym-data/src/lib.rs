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
    verify_encrypted_database, Db, LocalDatabaseStatus, ModuleEventRecord, OrganizationRecord,
    RecordedModuleEvent,
};
pub use error::DataError;
pub use key::{DbKey, EnvKeyProvider, FileKeyProvider, KeyProvider, KeyringKeyProvider};
pub use migrations::{Migration, MIGRATIONS};

/// Stable hex digest for event payload de-duplication.
pub fn sha256_hex(value: impl AsRef<[u8]>) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(value.as_ref()))
}
