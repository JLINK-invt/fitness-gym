#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use keyring::Entry;
use rand::rngs::OsRng;
use rand::RngCore;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

use crate::error::DataError;

const KEY_LEN: usize = 32;

/// 256-bit SQLCipher key. Debug output is redacted and bytes are zeroized.
#[derive(Clone)]
pub struct DbKey(Zeroizing<[u8; KEY_LEN]>);

impl DbKey {
    pub fn generate() -> Self {
        let mut bytes = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut bytes);
        Self(Zeroizing::new(bytes))
    }

    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn from_hex(value: &str) -> Result<Self, DataError> {
        let normalized = value.trim();
        let decoded =
            hex::decode(normalized).map_err(|error| DataError::InvalidKey(error.to_string()))?;
        let bytes: [u8; KEY_LEN] = decoded
            .try_into()
            .map_err(|_| DataError::InvalidKey("expected 32 bytes (64 hex chars)".into()))?;
        Ok(Self::from_bytes(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    /// Hex form used by `PRAGMA key = "x'...'"`. Keep it short-lived.
    pub(crate) fn to_pragma_hex(&self) -> Zeroizing<String> {
        Zeroizing::new(hex::encode(self.0.as_slice()))
    }
}

impl fmt::Debug for DbKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DbKey([REDACTED])")
    }
}

/// Custodian of the database key. Production adapters use the OS keychain;
/// tests and the first-run bootstrap use a protected file.
pub trait KeyProvider {
    fn load_or_create(&self) -> Result<DbKey, DataError>;
}

#[derive(Debug, Clone)]
pub struct FileKeyProvider {
    path: PathBuf,
}

impl FileKeyProvider {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn restrict_permissions(path: &Path) -> Result<(), DataError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        #[cfg(not(unix))]
        {
            let _ = path;
        }
        Ok(())
    }

    /// Persists an existing key with `0600` permissions. Used only for the
    /// controlled development/CI fallback and for migrating older installs.
    pub fn store(&self, key: &DbKey) -> Result<(), DataError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, hex::encode(key.as_bytes()))?;
        Self::restrict_permissions(&self.path)
    }
}

impl KeyProvider for FileKeyProvider {
    fn load_or_create(&self) -> Result<DbKey, DataError> {
        if self.path.exists() {
            let contents = fs::read_to_string(&self.path)?;
            Self::restrict_permissions(&self.path)?;
            return DbKey::from_hex(&contents);
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let key = DbKey::generate();
        fs::write(&self.path, hex::encode(key.as_bytes()))?;
        Self::restrict_permissions(&self.path)?;
        Ok(key)
    }
}

/// OS keychain adapter (Secret Service, Keychain or Credential Manager).
/// The key is stored as hex under a service/account pair; nothing is written
/// to the application data directory on supported platforms.
#[derive(Debug, Clone)]
pub struct KeyringKeyProvider {
    service: String,
    account: String,
}

impl KeyringKeyProvider {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }

    pub fn get(&self) -> Result<Option<DbKey>, DataError> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let entry = Entry::new(&self.service, &self.account)
                .map_err(|error| DataError::Keyring(error.to_string()))?;
            match entry.get_password() {
                Ok(value) => {
                    let value = Zeroizing::new(value);
                    DbKey::from_hex(&value).map(Some)
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(error) => Err(DataError::Keyring(error.to_string())),
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(DataError::Keyring(
                "the OS keychain is not supported on this platform".into(),
            ))
        }
    }

    pub fn store(&self, key: &DbKey) -> Result<(), DataError> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let encoded = Zeroizing::new(hex::encode(key.as_bytes()));
            let entry = Entry::new(&self.service, &self.account)
                .map_err(|error| DataError::Keyring(error.to_string()))?;
            entry
                .set_password(encoded.as_str())
                .map_err(|error| DataError::Keyring(error.to_string()))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            let _ = key;
            Err(DataError::Keyring(
                "the OS keychain is not supported on this platform".into(),
            ))
        }
    }
}

impl KeyProvider for KeyringKeyProvider {
    fn load_or_create(&self) -> Result<DbKey, DataError> {
        if let Some(key) = self.get()? {
            return Ok(key);
        }
        let key = DbKey::generate();
        self.store(&key)?;
        Ok(key)
    }
}

/// Reads the key from `GYM_DB_KEY` (64 hex chars). Intended for CI and
/// unattended installs; never commit the value.
#[derive(Debug, Clone)]
pub struct EnvKeyProvider {
    variable: String,
}

impl Default for EnvKeyProvider {
    fn default() -> Self {
        Self::new("GYM_DB_KEY")
    }
}

impl EnvKeyProvider {
    pub fn new(variable: impl Into<String>) -> Self {
        Self {
            variable: variable.into(),
        }
    }
}

impl KeyProvider for EnvKeyProvider {
    fn load_or_create(&self) -> Result<DbKey, DataError> {
        let value = std::env::var(&self.variable)
            .map_err(|_| DataError::InvalidKey(format!("{} is not set", self.variable)))?;
        DbKey::from_hex(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_provider_is_stable_and_restricted() {
        let dir = tempfile::tempdir().expect("temp dir");
        let provider = FileKeyProvider::new(dir.path().join("keys/db.key"));

        let first = provider.load_or_create().expect("first key");
        let second = provider.load_or_create().expect("second key");
        assert_eq!(first.as_bytes(), second.as_bytes());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(provider.path())
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn rejects_short_hex_keys() {
        assert!(DbKey::from_hex("abcd").is_err());
    }
}
