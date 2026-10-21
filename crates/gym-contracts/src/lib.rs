//! Versioned contracts shared by the Rust core and the Tauri frontend.
//!
//! Every payload is serialized in `camelCase` so the TypeScript client in
//! `packages/contracts` stays aligned without an adapter layer.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Version of the local command/event contract. Bump it on breaking changes.
pub const CONTRACT_VERSION: &str = "1.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ModuleHealth {
    pub fn ok(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Ok,
            detail: None,
        }
    }

    pub fn degraded(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub path: String,
    pub encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher_version: Option<String>,
    pub schema_version: i64,
    pub integrity_ok: bool,
    pub organizations: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub service: String,
    pub version: String,
    pub contract_version: String,
    pub status: HealthStatus,
    pub modules: Vec<ModuleHealth>,
    pub database: Option<DatabaseStatus>,
    pub checked_at: String,
}

/// Request sent by the simulated access module.
///
/// `credential_id` is a public badge identifier, not a secret. The real module
/// will replace it with an authenticated, signed envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessRequest {
    pub credential_id: String,
    pub device_id: String,
    pub idempotency_key: String,
}

/// Local decision returned to the access module and rendered by the desktop UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessDecision {
    pub event_id: String,
    pub allowed: bool,
    pub reason: String,
    pub duplicate: bool,
    pub idempotency_key: String,
    pub latency_ms: u64,
    pub decided_at: String,
}

/// Durable event summary used to prove that a decision is visible exactly once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessEventSummary {
    pub event_id: String,
    pub allowed: bool,
    pub idempotency_key: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub id: String,
    pub file_name: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub schema_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOutcome {
    pub backup_id: String,
    pub restored_at: String,
    pub schema_version: i64,
    pub integrity_ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    InvalidRequest,
    NotFound,
    Storage,
    Backup,
    Unauthorized,
    Internal,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[error("{code:?}: {message}")]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRequest, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_report_serializes_in_camel_case() {
        let report = HealthReport {
            service: "fitness-gym".into(),
            version: "0.1.0".into(),
            contract_version: CONTRACT_VERSION.into(),
            status: HealthStatus::Ok,
            modules: vec![ModuleHealth::ok("identity")],
            database: Some(DatabaseStatus {
                path: "/tmp/gym.db".into(),
                encrypted: true,
                cipher_version: Some("4.6.1".into()),
                schema_version: 1,
                integrity_ok: true,
                organizations: 1,
            }),
            checked_at: "2026-10-20T00:00:00Z".into(),
        };

        let json = serde_json::to_value(&report).expect("serializes");
        assert_eq!(json["contractVersion"], CONTRACT_VERSION);
        assert_eq!(json["database"]["cipherVersion"], "4.6.1");
        assert!(json.get("checked_at").is_none());
    }

    #[test]
    fn access_contract_serializes_in_camel_case() {
        let request = AccessRequest {
            credential_id: "CRED-ALLOW".into(),
            device_id: "recepcion-1".into(),
            idempotency_key: "req-1".into(),
        };
        let json = serde_json::to_value(request).expect("serializes");
        assert_eq!(json["credentialId"], "CRED-ALLOW");
        assert_eq!(json["deviceId"], "recepcion-1");
        assert_eq!(json["idempotencyKey"], "req-1");
    }

    #[test]
    fn api_error_omits_empty_details() {
        let error = ApiError::invalid("bad input");
        let json = serde_json::to_value(&error).expect("serializes");
        assert_eq!(json["code"], "invalidRequest");
        assert!(json.get("details").is_none());
    }
}
