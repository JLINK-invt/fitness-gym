//! Local password authentication and in-memory session rules.
//!
//! Passwords never leave this module in plaintext: only Argon2id PHC strings
//! are persisted. Sessions live in the Tauri process and are never written to
//! the webview, localStorage or the database.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{DateTime, Duration, Utc};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rbac::Role;

pub const OWNER_MIN_PASSWORD_CHARS: usize = 12;
pub const STAFF_MIN_PASSWORD_CHARS: usize = 8;
pub const MAX_PASSWORD_CHARS: usize = 128;
pub const MAX_LOGIN_ATTEMPTS: u8 = 5;
pub const LOGIN_LOCK_MINUTES: i64 = 5;
pub const IDLE_TIMEOUT_MINUTES: i64 = 15;
pub const MAX_NAME_CHARS: usize = 80;
pub const RECOVERY_CODE_GROUPS: usize = 4;
pub const RECOVERY_CODE_GROUP_CHARS: usize = 5;
pub const RECOVERY_CODE_CHARS: usize = RECOVERY_CODE_GROUPS * RECOVERY_CODE_GROUP_CHARS;

const RECOVERY_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("the password must have at least {minimum} characters")]
    WeakPassword { minimum: usize },

    #[error("the password cannot exceed {MAX_PASSWORD_CHARS} characters")]
    PasswordTooLong,

    #[error("the name must have between 2 and {MAX_NAME_CHARS} characters")]
    InvalidName,

    #[error("the recovery code is invalid")]
    InvalidRecoveryCode,

    #[error("the password could not be hashed: {0}")]
    Hash(String),
}

/// A person authenticated in this process. Never accepted from the webview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub user_id: String,
    pub organization_id: String,
    pub name: String,
    pub role: Role,
}

/// Trims, rejects control characters and enforces the visible-name bounds.
pub fn normalize_person_name(name: &str) -> Result<String, AuthError> {
    let name = name.trim();
    let length = name.chars().count();
    if !(2..=MAX_NAME_CHARS).contains(&length) || name.chars().any(char::is_control) {
        return Err(AuthError::InvalidName);
    }
    Ok(name.to_string())
}

/// Validates and hashes a password with Argon2id and a 128-bit random salt.
/// The caller chooses the minimum based on the role being created.
pub fn hash_password(password: &str, minimum_chars: usize) -> Result<String, AuthError> {
    if password.chars().count() < minimum_chars {
        return Err(AuthError::WeakPassword {
            minimum: minimum_chars,
        });
    }
    if password.chars().count() > MAX_PASSWORD_CHARS {
        return Err(AuthError::PasswordTooLong);
    }

    let mut salt_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut salt_bytes);
    let salt =
        SaltString::encode_b64(&salt_bytes).map_err(|error| AuthError::Hash(error.to_string()))?;

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AuthError::Hash(error.to_string()))
}

/// Verifies a password against a stored PHC string. Malformed hashes fail
/// closed and are indistinguishable from a wrong password to the caller.
pub fn verify_password(stored_hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// Creates a human-transferable recovery code such as `7K4QM-9XPTA-...`.
/// The alphabet omits visually ambiguous characters, so every byte maps
/// uniformly onto the 32 available symbols.
pub fn generate_recovery_code() -> String {
    let mut bytes = [0u8; RECOVERY_CODE_CHARS];
    OsRng.fill_bytes(&mut bytes);
    let mut code = String::with_capacity(RECOVERY_CODE_CHARS + RECOVERY_CODE_GROUPS - 1);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && index % RECOVERY_CODE_GROUP_CHARS == 0 {
            code.push('-');
        }
        code.push(RECOVERY_ALPHABET[*byte as usize % RECOVERY_ALPHABET.len()] as char);
    }
    code
}

/// Accepts lowercase input, spaces and dashes, then enforces the code
/// alphabet and exact length. Returns `None` for malformed codes.
pub fn normalize_recovery_code(value: &str) -> Option<String> {
    let normalized: String = value
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .map(|character| character.to_ascii_uppercase())
        .collect();
    if normalized.chars().count() != RECOVERY_CODE_CHARS
        || !normalized
            .bytes()
            .all(|byte| RECOVERY_ALPHABET.contains(&byte))
    {
        return None;
    }
    Some(normalized)
}

pub fn hash_recovery_code(code: &str) -> Result<String, AuthError> {
    let normalized = normalize_recovery_code(code).ok_or(AuthError::InvalidRecoveryCode)?;
    hash_password(&normalized, RECOVERY_CODE_CHARS)
}

/// Malformed stored hashes and malformed submitted codes both fail closed.
pub fn verify_recovery_code(stored_hash: &str, code: &str) -> bool {
    normalize_recovery_code(code)
        .is_some_and(|normalized| verify_password(stored_hash, &normalized))
}

/// Session inactivity is measured from the last authorized command.
pub fn session_is_idle(last_seen: DateTime<Utc>, now: DateTime<Utc>, timeout_minutes: i64) -> bool {
    now.signed_duration_since(last_seen) >= Duration::minutes(timeout_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_password_must_have_twelve_characters() {
        let error = hash_password("corta-12345", OWNER_MIN_PASSWORD_CHARS)
            .expect_err("must reject short owner password");
        assert_eq!(
            error,
            AuthError::WeakPassword {
                minimum: OWNER_MIN_PASSWORD_CHARS
            }
        );
    }

    #[test]
    fn password_hashes_are_salted_and_verifiable() {
        let first =
            hash_password("clave-segura-123", OWNER_MIN_PASSWORD_CHARS).expect("hashes password");
        let second = hash_password("clave-segura-123", OWNER_MIN_PASSWORD_CHARS)
            .expect("hashes the same password again");

        assert_ne!(first, second, "each hash needs a unique salt");
        assert!(first.starts_with("$argon2id$"));
        assert!(verify_password(&first, "clave-segura-123"));
        assert!(!verify_password(&first, "clave-incorrecta"));
        assert!(!verify_password("not-a-phc-string", "clave-segura-123"));
    }

    #[test]
    fn names_are_trimmed_and_bounded() {
        assert_eq!(
            normalize_person_name("  Ana Pérez  ").expect("valid name"),
            "Ana Pérez"
        );
        assert_eq!(normalize_person_name("A"), Err(AuthError::InvalidName));
        assert_eq!(
            normalize_person_name("Ana\nPérez"),
            Err(AuthError::InvalidName)
        );
    }

    #[test]
    fn recovery_codes_are_transferable_and_hashed() {
        let code = generate_recovery_code();
        assert_eq!(code.len(), RECOVERY_CODE_CHARS + RECOVERY_CODE_GROUPS - 1);
        assert_eq!(
            normalize_recovery_code(&code.to_lowercase().replace('-', " ")),
            normalize_recovery_code(&code)
        );

        let hash = hash_recovery_code(&code).expect("hashes recovery code");
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_recovery_code(&hash, &code.to_lowercase()));
        assert!(!verify_recovery_code(&hash, &generate_recovery_code()));
    }

    #[test]
    fn malformed_recovery_codes_are_rejected() {
        assert_eq!(normalize_recovery_code(""), None);
        assert_eq!(normalize_recovery_code("AAAAA-AAAAA-AAAAA-AAAA"), None);
        assert_eq!(normalize_recovery_code("AAAAA-AAAAA-AAAAA-AAA1A"), None);
        assert_eq!(
            hash_recovery_code("no-es-un-codigo"),
            Err(AuthError::InvalidRecoveryCode)
        );
        assert!(!verify_recovery_code("not-a-phc-string", "AAAAA"));
    }

    #[test]
    fn idle_sessions_expire_at_fifteen_minutes() {
        let now = Utc::now();
        assert!(!session_is_idle(
            now - Duration::minutes(14),
            now,
            IDLE_TIMEOUT_MINUTES
        ));
        assert!(session_is_idle(
            now - Duration::minutes(15),
            now,
            IDLE_TIMEOUT_MINUTES
        ));
    }
}
