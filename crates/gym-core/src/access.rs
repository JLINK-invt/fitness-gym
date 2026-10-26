//! Simulated access decisions for the Phase 1 vertical slice.
//!
//! This is deliberately not authentication: it accepts two known public badge
//! identifiers so the local flow can be exercised before the real module and
//! membership rules exist. Never reuse this module for production access.

use chrono::{Datelike, NaiveDate};

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

/// Inputs required to evaluate a real member credential without touching the
/// database. The caller loads the current membership snapshot and passes the
/// decision time explicitly so the result is deterministic and testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberAccessContext<'a> {
    pub credential_status: &'a str,
    pub member_status: &'a str,
    pub birth_date: Option<&'a str>,
    pub membership_status: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub access_days: Option<&'a str>,
    pub access_start_minute: Option<i64>,
    pub access_end_minute: Option<i64>,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
    pub today: &'a str,
    pub minute: i64,
}

pub fn decide_member_access(context: &MemberAccessContext<'_>) -> SimulatedAccessDecision {
    if context.credential_status != "active" {
        return denied("credencial revocada");
    }
    if context.member_status != "active" {
        return denied("socio inactivo");
    }
    let Some(status) = context.membership_status else {
        return denied("sin membresía vigente");
    };
    match status {
        "active" | "scheduled" => {}
        "paused" => return denied("membresía pausada"),
        _ => return denied("membresía vencida"),
    }
    let (Some(start_date), Some(end_date)) = (context.start_date, context.end_date) else {
        return denied("membresía sin vigencia");
    };
    if context.today < start_date || context.today > end_date {
        return denied("membresía fuera de vigencia");
    }
    let (Some(access_days), Some(start_minute), Some(end_minute)) = (
        context.access_days,
        context.access_start_minute,
        context.access_end_minute,
    ) else {
        return denied("membresía sin reglas de acceso");
    };
    if !access_day_allowed(access_days, context.today) {
        return denied("día no permitido por la membresía");
    }
    if context.minute < start_minute || context.minute >= end_minute {
        return denied("fuera del horario permitido");
    }
    if let Some(age) = member_age(context.birth_date, context.today) {
        if context.min_age.is_some_and(|min| age < min)
            || context.max_age.is_some_and(|max| age > max)
        {
            return denied("edad fuera del rango permitido");
        }
    } else if context.min_age.is_some() || context.max_age.is_some() {
        return denied("edad del socio no registrada");
    }
    SimulatedAccessDecision {
        allowed: true,
        reason: "acceso autorizado",
    }
}

fn denied(reason: &'static str) -> SimulatedAccessDecision {
    SimulatedAccessDecision {
        allowed: false,
        reason,
    }
}

fn access_day_allowed(access_days: &str, today: &str) -> bool {
    let Ok(date) = NaiveDate::parse_from_str(today, "%Y-%m-%d") else {
        return false;
    };
    let day = match date.weekday() {
        chrono::Weekday::Mon => "mon",
        chrono::Weekday::Tue => "tue",
        chrono::Weekday::Wed => "wed",
        chrono::Weekday::Thu => "thu",
        chrono::Weekday::Fri => "fri",
        chrono::Weekday::Sat => "sat",
        chrono::Weekday::Sun => "sun",
    };
    access_days
        .split(',')
        .map(str::trim)
        .any(|value| value.eq_ignore_ascii_case(day))
}

fn member_age(birth_date: Option<&str>, today: &str) -> Option<i64> {
    let birth_date = NaiveDate::parse_from_str(birth_date?, "%Y-%m-%d").ok()?;
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").ok()?;
    let mut age = today.year() - birth_date.year();
    if (today.month(), today.day()) < (birth_date.month(), birth_date.day()) {
        age -= 1;
    }
    (age >= 0).then_some(i64::from(age))
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

    fn active_context() -> MemberAccessContext<'static> {
        MemberAccessContext {
            credential_status: "active",
            member_status: "active",
            birth_date: Some("1990-01-01"),
            membership_status: Some("active"),
            start_date: Some("2026-10-01"),
            end_date: Some("2026-10-31"),
            access_days: Some("mon,tue,wed,thu,fri,sat,sun"),
            access_start_minute: Some(360),
            access_end_minute: Some(1320),
            min_age: None,
            max_age: None,
            today: "2026-10-26",
            minute: 600,
        }
    }

    #[test]
    fn active_membership_is_granted() {
        let decision = decide_member_access(&active_context());
        assert!(decision.allowed);
    }

    #[test]
    fn paused_membership_is_rejected() {
        let context = MemberAccessContext {
            membership_status: Some("paused"),
            ..active_context()
        };
        assert_eq!(decide_member_access(&context).reason, "membresía pausada");
    }

    #[test]
    fn day_and_time_windows_are_enforced() {
        let wrong_day = MemberAccessContext {
            access_days: Some("sat,sun"),
            ..active_context()
        };
        assert_eq!(
            decide_member_access(&wrong_day).reason,
            "día no permitido por la membresía"
        );

        let early = MemberAccessContext {
            minute: 300,
            ..active_context()
        };
        assert_eq!(
            decide_member_access(&early).reason,
            "fuera del horario permitido"
        );
    }

    #[test]
    fn age_limits_are_enforced() {
        let context = MemberAccessContext {
            min_age: Some(40),
            ..active_context()
        };
        assert_eq!(
            decide_member_access(&context).reason,
            "edad fuera del rango permitido"
        );

        let missing_birth_date = MemberAccessContext {
            birth_date: None,
            min_age: Some(18),
            ..active_context()
        };
        assert_eq!(
            decide_member_access(&missing_birth_date).reason,
            "edad del socio no registrada"
        );
    }
}
