//! Simulated access decisions for the Phase 1 vertical slice.
//!
//! This is deliberately not authentication: it accepts two known public badge
//! identifiers so the local flow can be exercised before the real module and
//! membership rules exist. Never reuse this module for production access.

use crate::error::CoreError;

pub const SIMULATED_ALLOW_CREDENTIAL: &str = "CRED-ALLOW";
pub const SIMULATED_DENY_CREDENTIAL: &str = "CRED-DENY";
pub const MAX_CREDENTIAL_LENGTH: usize = 64;
pub const MAX_DEVICE_LENGTH: usize = 64;
pub const MAX_IDEMPOTENCY_LENGTH: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulatedAccessDecision {
    pub allowed: bool,
    pub reason: &'static str,
}

pub fn decide_simulated_access(
    credential_id: &str,
    device_id: &str,
    idempotency_key: &str,
) -> Result<SimulatedAccessDecision, CoreError> {
    let credential_id = required(credential_id, "credential id", MAX_CREDENTIAL_LENGTH)?;
    required(device_id, "device id", MAX_DEVICE_LENGTH)?;
    required(idempotency_key, "idempotency key", MAX_IDEMPOTENCY_LENGTH)?;

    Ok(match credential_id {
        SIMULATED_ALLOW_CREDENTIAL => SimulatedAccessDecision {
            allowed: true,
            reason: "credencial simulada activa",
        },
        SIMULATED_DENY_CREDENTIAL => SimulatedAccessDecision {
            allowed: false,
            reason: "credencial simulada sin acceso",
        },
        _ => SimulatedAccessDecision {
            allowed: false,
            reason: "credencial desconocida",
        },
    })
}

fn required<'a>(value: &'a str, field: &str, max: usize) -> Result<&'a str, CoreError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CoreError::InvalidValue(format!("{field} is required")));
    }
    if value.len() > max {
        return Err(CoreError::InvalidValue(format!(
            "{field} exceeds {max} bytes"
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_allow_credential_is_granted() {
        let decision =
            decide_simulated_access("CRED-ALLOW", "recepcion-1", "req-1").expect("valid request");
        assert!(decision.allowed);
    }

    #[test]
    fn known_deny_credential_is_rejected() {
        let decision =
            decide_simulated_access("CRED-DENY", "recepcion-1", "req-2").expect("valid request");
        assert!(!decision.allowed);
    }

    #[test]
    fn unknown_credential_is_rejected() {
        let decision =
            decide_simulated_access("CRED-999", "recepcion-1", "req-3").expect("valid request");
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "credencial desconocida");
    }

    #[test]
    fn missing_idempotency_key_is_invalid() {
        let error = decide_simulated_access("CRED-ALLOW", "recepcion-1", "  ")
            .expect_err("must reject empty key");
        assert!(matches!(error, CoreError::InvalidValue(_)));
    }
}
