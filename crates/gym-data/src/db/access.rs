use std::collections::HashMap;

use hmac::{Hmac, Mac};
use rusqlite::{params, Connection, OptionalExtension, Row};
use sha2::Sha256;

use super::{normalized_report_range, Db};
use crate::error::DataError;
use crate::sha256_hex;

const SNAPSHOT_SIGNING_CONTEXT: &str = "fitness-gym-access-snapshot-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberCredentialRecord {
    pub id: String,
    pub member_id: String,
    pub member_number: String,
    pub member_name: String,
    pub code: String,
    pub status: String,
    pub revision: i64,
    pub issued_at: String,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialAccessRuleRecord {
    pub credential_id: String,
    pub credential_code: String,
    pub credential_status: String,
    pub credential_revision: i64,
    pub member_id: String,
    pub member_number: String,
    pub member_name: String,
    pub member_status: String,
    pub birth_date: Option<String>,
    pub membership_id: Option<String>,
    pub plan_name: Option<String>,
    pub membership_status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub access_days: Option<String>,
    pub access_start_minute: Option<i64>,
    pub access_end_minute: Option<i64>,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessSnapshotRecord {
    pub revision: i64,
    pub generated_at: String,
    pub checksum: String,
    pub signature: String,
    pub member_count: i64,
    pub credential_count: i64,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessEventRecord {
    pub id: String,
    pub credential_code: Option<String>,
    pub member_id: Option<String>,
    pub member_number: Option<String>,
    pub member_name: Option<String>,
    pub membership_id: Option<String>,
    pub plan_name: Option<String>,
    pub device_id: String,
    pub zone: Option<String>,
    pub allowed: bool,
    pub reason: String,
    pub snapshot_revision: i64,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub reconciliation_status: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAccessEvent {
    pub credential_code: String,
    pub device_id: String,
    pub zone: Option<String>,
    pub allowed: bool,
    pub reason: String,
    pub snapshot_revision: i64,
    pub occurred_at: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessEventWriteOutcome {
    Inserted(AccessEventRecord),
    Duplicate(AccessEventRecord),
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessConflictRecord {
    pub idempotency_key: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessReconciliationRecord {
    pub snapshot_revision: i64,
    pub applied: i64,
    pub duplicates: i64,
    pub stale: i64,
    pub conflicts: Vec<AccessConflictRecord>,
    pub events: Vec<AccessEventRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceHourRecord {
    pub hour: i64,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceDayRecord {
    pub date: String,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendancePlanRecord {
    pub plan_id: Option<String>,
    pub plan_name: String,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceZoneRecord {
    pub zone: String,
    pub visits: i64,
    pub unique_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceMemberRecord {
    pub member_id: String,
    pub member_number: String,
    pub member_name: String,
    pub visits: i64,
    pub last_visit_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceReportRecord {
    pub from: String,
    pub to: String,
    pub timezone: String,
    pub total_visits: i64,
    pub unique_members: i64,
    pub peak_hour: i64,
    pub low_demand_hours: Vec<AttendanceHourRecord>,
    pub hours: Vec<AttendanceHourRecord>,
    pub days: Vec<AttendanceDayRecord>,
    pub plans: Vec<AttendancePlanRecord>,
    pub zones: Vec<AttendanceZoneRecord>,
    pub members: Vec<AttendanceMemberRecord>,
}

impl Db {
    pub fn issue_member_credential(
        &self,
        organization_id: &str,
        member_id: &str,
    ) -> Result<MemberCredentialRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        let credential_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let member_number: String = transaction
                .query_row(
                    "SELECT member_number FROM members
                     WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, member_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(DataError::MemberNotFound)?;
            let code = generate_credential_code(&member_number);
            let credential_id = uuid::Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO member_credentials
                     (id, organization_id, member_id, code, status, revision,
                      issued_at, revoked_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', 1, ?5, NULL, ?5, ?5)",
                params![credential_id, organization_id, member_id, code, now],
            )?;
            transaction.commit()?;
            credential_id
        };
        self.member_credential(organization_id, &credential_id)
    }

    pub fn revoke_member_credential(
        &self,
        organization_id: &str,
        credential_id: &str,
    ) -> Result<MemberCredentialRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.with_connection(|conn| {
            let status: String = conn
                .query_row(
                    "SELECT status FROM member_credentials
                     WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, credential_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(DataError::MemberCredentialNotFound)?;
            if status == "revoked" {
                return load_member_credential(conn, organization_id, credential_id);
            }
            conn.execute(
                "UPDATE member_credentials
                 SET status = 'revoked', revision = revision + 1,
                     revoked_at = ?1, updated_at = ?1
                 WHERE organization_id = ?2 AND id = ?3",
                params![now, organization_id, credential_id],
            )?;
            load_member_credential(conn, organization_id, credential_id)
        })
    }

    pub fn member_credential(
        &self,
        organization_id: &str,
        credential_id: &str,
    ) -> Result<MemberCredentialRecord, DataError> {
        self.with_connection(|conn| load_member_credential(conn, organization_id, credential_id))
    }

    pub fn list_member_credentials(
        &self,
        organization_id: &str,
        member_id: &str,
    ) -> Result<Vec<MemberCredentialRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT c.id, c.member_id, m.member_number,
                        m.first_name || ' ' || m.last_name,
                        c.code, c.status, c.revision, c.issued_at, c.revoked_at,
                        c.created_at, c.updated_at
                 FROM member_credentials c
                 JOIN members m ON m.id = c.member_id
                 WHERE c.organization_id = ?1 AND c.member_id = ?2
                 ORDER BY c.status = 'active' DESC, c.created_at DESC",
            )?;
            let rows = statement
                .query_map(params![organization_id, member_id], map_member_credential)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn access_rule_for_credential(
        &self,
        organization_id: &str,
        credential_code: &str,
    ) -> Result<Option<CredentialAccessRuleRecord>, DataError> {
        self.with_connection(|conn| {
            load_credential_access_rule(conn, organization_id, credential_code)
        })
    }

    pub fn publish_access_snapshot(
        &self,
        organization_id: &str,
        signing_key: &[u8],
    ) -> Result<AccessSnapshotRecord, DataError> {
        let generated_at = chrono::Utc::now().to_rfc3339();
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let revision: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(revision), 0) + 1 FROM access_snapshots
             WHERE organization_id = ?1",
            params![organization_id],
            |row| row.get(0),
        )?;
        let members = snapshot_members(&transaction, organization_id)?;
        let member_count = members.len() as i64;
        let credential_count = members
            .iter()
            .map(|member| member["credentials"].as_array().map_or(0, Vec::len) as i64)
            .sum::<i64>();
        let payload = serde_json::json!({
            "revision": revision,
            "generatedAt": generated_at,
            "organizationId": organization_id,
            "members": members,
        });
        let payload_json = serde_json::to_string(&payload)?;
        let checksum = sha256_hex(payload_json.as_bytes());
        let signature = sign_snapshot(signing_key, &payload_json);
        transaction.execute(
            "INSERT INTO access_snapshots
                 (id, organization_id, revision, generated_at, checksum,
                  signature, member_count, credential_count, payload_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                uuid::Uuid::new_v4().to_string(),
                organization_id,
                revision,
                generated_at,
                checksum,
                signature,
                member_count,
                credential_count,
                payload_json
            ],
        )?;
        transaction.commit()?;
        Ok(AccessSnapshotRecord {
            revision,
            generated_at,
            checksum,
            signature,
            member_count,
            credential_count,
            payload_json,
        })
    }

    pub fn latest_access_snapshot(
        &self,
        organization_id: &str,
    ) -> Result<Option<AccessSnapshotRecord>, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT revision, generated_at, checksum, signature,
                        member_count, credential_count, payload_json
                 FROM access_snapshots
                 WHERE organization_id = ?1
                 ORDER BY revision DESC
                 LIMIT 1",
                params![organization_id],
                |row| {
                    Ok(AccessSnapshotRecord {
                        revision: row.get(0)?,
                        generated_at: row.get(1)?,
                        checksum: row.get(2)?,
                        signature: row.get(3)?,
                        member_count: row.get(4)?,
                        credential_count: row.get(5)?,
                        payload_json: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(DataError::from)
        })
    }

    pub fn record_access_event(
        &self,
        organization_id: &str,
        event: NewAccessEvent,
    ) -> Result<AccessEventWriteOutcome, DataError> {
        let credential_code = required_text(&event.credential_code, "credential code", 64)?;
        let device_id = required_text(&event.device_id, "device id", 64)?;
        let idempotency_key = required_text(&event.idempotency_key, "idempotency key", 128)?;
        let zone = event
            .zone
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| required_text(value, "zone", 64))
            .transpose()?;
        let reason = required_text(&event.reason, "reason", 160)?;
        let occurred_at = normalized_timestamp(&event.occurred_at)?;
        if event.snapshot_revision < 0 {
            return Err(DataError::InvalidValue(
                "snapshot revision cannot be negative".into(),
            ));
        }
        let payload_hash = access_event_hash(
            &credential_code,
            &device_id,
            zone.as_deref(),
            event.allowed,
            &reason,
            event.snapshot_revision,
            &occurred_at,
        );

        let event_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let existing: Option<String> = transaction
                .query_row(
                    "SELECT payload_hash FROM access_events
                     WHERE organization_id = ?1 AND idempotency_key = ?2",
                    params![organization_id, idempotency_key],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(existing_hash) = existing {
                if existing_hash != payload_hash {
                    return Ok(AccessEventWriteOutcome::Conflict);
                }
                let event =
                    load_access_event_by_key(&transaction, organization_id, &idempotency_key)?;
                return Ok(AccessEventWriteOutcome::Duplicate(event));
            }

            let rule =
                load_credential_access_rule(&transaction, organization_id, &credential_code)?;
            let latest_revision: i64 = transaction.query_row(
                "SELECT COALESCE(MAX(revision), 0) FROM access_snapshots
                 WHERE organization_id = ?1",
                params![organization_id],
                |row| row.get(0),
            )?;
            let reconciliation_status = if event.snapshot_revision < latest_revision {
                "stale"
            } else {
                "applied"
            };
            let event_id = uuid::Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO access_events
                     (id, organization_id, credential_code, member_id,
                      membership_id, device_id, zone, allowed, reason,
                      snapshot_revision, idempotency_key, payload_hash,
                      reconciliation_status, occurred_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    event_id,
                    organization_id,
                    credential_code,
                    rule.as_ref().map(|rule| rule.member_id.as_str()),
                    rule.as_ref().and_then(|rule| rule.membership_id.as_deref()),
                    device_id,
                    zone,
                    i64::from(event.allowed),
                    reason,
                    event.snapshot_revision,
                    idempotency_key,
                    payload_hash,
                    reconciliation_status,
                    occurred_at,
                    chrono::Utc::now().to_rfc3339()
                ],
            )?;
            transaction.commit()?;
            event_id
        };
        let event = self.access_event(organization_id, &event_id)?;
        Ok(AccessEventWriteOutcome::Inserted(event))
    }

    pub fn reconcile_access_events(
        &self,
        organization_id: &str,
        snapshot_revision: i64,
        events: Vec<NewAccessEvent>,
    ) -> Result<AccessReconciliationRecord, DataError> {
        let latest_revision = self
            .latest_access_snapshot(organization_id)?
            .map(|snapshot| snapshot.revision)
            .unwrap_or(0);
        if snapshot_revision < 0 {
            return Err(DataError::InvalidValue(
                "snapshot revision cannot be negative".into(),
            ));
        }
        let mut applied = 0;
        let mut duplicates = 0;
        let mut stale = 0;
        let mut conflicts = Vec::new();
        let mut recorded = Vec::new();
        for mut event in events {
            if event.snapshot_revision == 0 {
                event.snapshot_revision = snapshot_revision;
            }
            let idempotency_key = event.idempotency_key.clone();
            match self.record_access_event(organization_id, event)? {
                AccessEventWriteOutcome::Inserted(event) => {
                    if event.reconciliation_status == "stale" {
                        stale += 1;
                    } else {
                        applied += 1;
                    }
                    recorded.push(event);
                }
                AccessEventWriteOutcome::Duplicate(event) => {
                    duplicates += 1;
                    recorded.push(event);
                }
                AccessEventWriteOutcome::Conflict => {
                    conflicts.push(AccessConflictRecord {
                        idempotency_key,
                        reason: "la clave de idempotencia ya se usó con otro contenido".into(),
                    });
                }
            }
        }
        Ok(AccessReconciliationRecord {
            snapshot_revision: latest_revision,
            applied,
            duplicates,
            stale,
            conflicts,
            events: recorded,
        })
    }

    pub fn access_event(
        &self,
        organization_id: &str,
        event_id: &str,
    ) -> Result<AccessEventRecord, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                &access_event_select("WHERE ae.organization_id = ?1 AND ae.id = ?2"),
                params![organization_id, event_id],
                map_access_event,
            )
            .optional()?
            .ok_or_else(|| DataError::InvalidValue("access event not found".into()))
        })
    }

    pub fn list_access_events(
        &self,
        organization_id: &str,
        limit: usize,
    ) -> Result<Vec<AccessEventRecord>, DataError> {
        let limit = limit.clamp(1, 100) as i64;
        self.with_connection(|conn| {
            let mut events = {
                let mut statement = conn.prepare(&access_event_select(
                    "WHERE ae.organization_id = ?1
                     ORDER BY ae.occurred_at DESC, ae.id DESC
                     LIMIT ?2",
                ))?;
                let rows = statement
                    .query_map(params![organization_id, limit], map_access_event)?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            };
            let mut statement = conn.prepare(
                "SELECT id, event_type, idempotency_key, payload_hash, created_at
                 FROM module_events
                 WHERE organization_id = ?1 AND aggregate = 'access_decision'
                   AND NOT EXISTS (
                       SELECT 1 FROM access_events ae WHERE ae.id = module_events.id
                   )
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )?;
            let legacy = statement
                .query_map(params![organization_id, limit], |row| {
                    let event_type: String = row.get(1)?;
                    Ok(AccessEventRecord {
                        id: row.get(0)?,
                        credential_code: None,
                        member_id: None,
                        member_number: None,
                        member_name: None,
                        membership_id: None,
                        plan_name: None,
                        device_id: "legacy".into(),
                        zone: None,
                        allowed: event_type == "access.allowed",
                        reason: event_type,
                        snapshot_revision: 0,
                        idempotency_key: row.get(2)?,
                        payload_hash: row.get(3)?,
                        reconciliation_status: "legacy".into(),
                        occurred_at: row.get(4)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            events.extend(legacy);
            events.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
            events.truncate(limit as usize);
            Ok(events)
        })
    }

    pub fn attendance_report(
        &self,
        organization_id: &str,
        from: &str,
        to: &str,
    ) -> Result<AttendanceReportRecord, DataError> {
        let (from, to) = normalized_report_range(from, to)?;
        self.with_connection(|conn| {
            let (total_visits, unique_members) = conn.query_row(
                "SELECT COUNT(*), COUNT(DISTINCT member_id)
                 FROM access_events
                 WHERE organization_id = ?1 AND allowed = 1
                   AND reconciliation_status <> 'legacy'
                   AND julianday(occurred_at) >= julianday(?2)
                   AND julianday(occurred_at) < julianday(?3)",
                params![organization_id, from, to],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;

            let (open_start, open_end) = conn.query_row(
                "SELECT COALESCE(MIN(access_start_minute / 60), 0),
                        COALESCE(MAX((access_end_minute + 59) / 60), 24)
                 FROM memberships
                 WHERE organization_id = ?1
                   AND status IN ('scheduled', 'active', 'paused')",
                params![organization_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )?;
            let open_start = open_start.clamp(0, 23);
            let open_end = open_end.clamp(open_start + 1, 24);

            let mut by_hour: HashMap<i64, (i64, i64)> = HashMap::new();
            {
                let mut statement = conn.prepare(
                    "SELECT CAST(strftime('%H', occurred_at) AS INTEGER),
                            COUNT(*), COUNT(DISTINCT member_id)
                     FROM access_events
                     WHERE organization_id = ?1 AND allowed = 1
                       AND reconciliation_status <> 'legacy'
                       AND julianday(occurred_at) >= julianday(?2)
                       AND julianday(occurred_at) < julianday(?3)
                     GROUP BY 1",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })?;
                for row in rows {
                    let (hour, visits, members) = row?;
                    by_hour.insert(hour, (visits, members));
                }
            }
            let hours = (open_start..open_end)
                .map(|hour| {
                    let (visits, unique_members) = by_hour.get(&hour).copied().unwrap_or((0, 0));
                    AttendanceHourRecord {
                        hour,
                        visits,
                        unique_members,
                    }
                })
                .collect::<Vec<_>>();
            let peak_hour = hours
                .iter()
                .max_by_key(|hour| hour.visits)
                .map(|hour| hour.hour)
                .unwrap_or(open_start);
            let peak_visits = hours.iter().map(|hour| hour.visits).max().unwrap_or(0);
            let low_demand_hours = hours
                .iter()
                .filter(|hour| peak_visits > 0 && hour.visits as f64 <= peak_visits as f64 * 0.5)
                .cloned()
                .collect::<Vec<_>>();

            let days = {
                let mut statement = conn.prepare(
                    "SELECT substr(occurred_at, 1, 10), COUNT(*),
                            COUNT(DISTINCT member_id)
                     FROM access_events
                     WHERE organization_id = ?1 AND allowed = 1
                       AND reconciliation_status <> 'legacy'
                       AND julianday(occurred_at) >= julianday(?2)
                       AND julianday(occurred_at) < julianday(?3)
                     GROUP BY 1 ORDER BY 1",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok(AttendanceDayRecord {
                        date: row.get(0)?,
                        visits: row.get(1)?,
                        unique_members: row.get(2)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            };

            let plans = {
                let mut statement = conn.prepare(
                    "SELECT ms.plan_id, ms.plan_name, COUNT(*),
                            COUNT(DISTINCT ae.member_id)
                     FROM access_events ae
                     LEFT JOIN memberships ms ON ms.id = ae.membership_id
                     WHERE ae.organization_id = ?1 AND ae.allowed = 1
                       AND ae.reconciliation_status <> 'legacy'
                       AND julianday(ae.occurred_at) >= julianday(?2)
                       AND julianday(ae.occurred_at) < julianday(?3)
                     GROUP BY ms.plan_id, ms.plan_name
                     ORDER BY COUNT(*) DESC, ms.plan_name",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok(AttendancePlanRecord {
                        plan_id: row.get(0)?,
                        plan_name: row
                            .get::<_, Option<String>>(1)?
                            .unwrap_or_else(|| "Sin plan".into()),
                        visits: row.get(2)?,
                        unique_members: row.get(3)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            };

            let zones = {
                let mut statement = conn.prepare(
                    "SELECT COALESCE(NULLIF(trim(zone), ''), 'Sin zona'),
                            COUNT(*), COUNT(DISTINCT member_id)
                     FROM access_events
                     WHERE organization_id = ?1 AND allowed = 1
                       AND reconciliation_status <> 'legacy'
                       AND julianday(occurred_at) >= julianday(?2)
                       AND julianday(occurred_at) < julianday(?3)
                     GROUP BY 1 ORDER BY COUNT(*) DESC, 1",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok(AttendanceZoneRecord {
                        zone: row.get(0)?,
                        visits: row.get(1)?,
                        unique_members: row.get(2)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            };

            let members = {
                let mut statement = conn.prepare(
                    "SELECT m.id, m.member_number,
                            m.first_name || ' ' || m.last_name,
                            COUNT(*), MAX(ae.occurred_at)
                     FROM access_events ae
                     JOIN members m ON m.id = ae.member_id
                     WHERE ae.organization_id = ?1 AND ae.allowed = 1
                       AND ae.reconciliation_status <> 'legacy'
                       AND julianday(ae.occurred_at) >= julianday(?2)
                       AND julianday(ae.occurred_at) < julianday(?3)
                     GROUP BY m.id, m.member_number, m.first_name, m.last_name
                     ORDER BY COUNT(*) DESC, m.last_name, m.first_name
                     LIMIT 100",
                )?;
                let rows = statement.query_map(params![organization_id, from, to], |row| {
                    Ok(AttendanceMemberRecord {
                        member_id: row.get(0)?,
                        member_number: row.get(1)?,
                        member_name: row.get(2)?,
                        visits: row.get(3)?,
                        last_visit_at: row.get(4)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            };

            Ok(AttendanceReportRecord {
                from,
                to,
                timezone: "UTC".into(),
                total_visits,
                unique_members,
                peak_hour,
                low_demand_hours,
                hours,
                days,
                plans,
                zones,
                members,
            })
        })
    }
}

fn generate_credential_code(member_number: &str) -> String {
    let token = uuid::Uuid::new_v4().to_string().replace('-', "");
    format!("GYM-{member_number}-{}", token[..10].to_uppercase())
}

fn required_text(value: &str, field: &str, max: usize) -> Result<String, DataError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DataError::InvalidValue(format!("{field} is required")));
    }
    if value.len() > max {
        return Err(DataError::InvalidValue(format!(
            "{field} exceeds {max} bytes"
        )));
    }
    Ok(value.to_string())
}

fn normalized_timestamp(value: &str) -> Result<String, DataError> {
    chrono::DateTime::parse_from_rfc3339(value.trim())
        .map(|value| {
            value
                .with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        })
        .map_err(|_| DataError::InvalidValue("access event timestamp must be RFC 3339".into()))
}

fn access_event_hash(
    credential_code: &str,
    device_id: &str,
    zone: Option<&str>,
    allowed: bool,
    reason: &str,
    snapshot_revision: i64,
    occurred_at: &str,
) -> String {
    sha256_hex(format!(
        "{credential_code}\n{device_id}\n{}\n{allowed}\n{reason}\n{snapshot_revision}\n{occurred_at}",
        zone.unwrap_or("none")
    ))
}

fn sign_snapshot(signing_key: &[u8], payload: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(signing_key).expect("HMAC accepts keys of any length");
    mac.update(SNAPSHOT_SIGNING_CONTEXT.as_bytes());
    mac.update(b"\n");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn snapshot_members(
    conn: &Connection,
    organization_id: &str,
) -> Result<Vec<serde_json::Value>, DataError> {
    let mut statement = conn.prepare(
        "SELECT id, member_number, first_name, last_name, status
         FROM members
         WHERE organization_id = ?1
         ORDER BY member_number",
    )?;
    let rows = statement
        .query_map(params![organization_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut members = Vec::with_capacity(rows.len());
    for (member_id, member_number, first_name, last_name, status) in rows {
        let mut credentials_statement = conn.prepare(
            "SELECT code, status, revision, issued_at, revoked_at
             FROM member_credentials
             WHERE organization_id = ?1 AND member_id = ?2
             ORDER BY code",
        )?;
        let credentials = credentials_statement
            .query_map(params![organization_id, member_id], |row| {
                Ok(serde_json::json!({
                    "code": row.get::<_, String>(0)?,
                    "status": row.get::<_, String>(1)?,
                    "revision": row.get::<_, i64>(2)?,
                    "issuedAt": row.get::<_, String>(3)?,
                    "revokedAt": row.get::<_, Option<String>>(4)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let membership = conn
            .query_row(
                "SELECT id, plan_id, plan_name, plan_version, status,
                        start_date, end_date, access_days,
                        access_start_minute, access_end_minute, min_age, max_age
                 FROM memberships
                 WHERE organization_id = ?1 AND member_id = ?2
                   AND status IN ('scheduled', 'active', 'paused')
                 ORDER BY created_at DESC
                 LIMIT 1",
                params![organization_id, member_id],
                |row| {
                    Ok(serde_json::json!({
                        "membershipId": row.get::<_, String>(0)?,
                        "planId": row.get::<_, String>(1)?,
                        "planName": row.get::<_, String>(2)?,
                        "planVersion": row.get::<_, i64>(3)?,
                        "status": row.get::<_, String>(4)?,
                        "startDate": row.get::<_, String>(5)?,
                        "endDate": row.get::<_, String>(6)?,
                        "accessDays": row.get::<_, String>(7)?,
                        "accessStartMinute": row.get::<_, i64>(8)?,
                        "accessEndMinute": row.get::<_, i64>(9)?,
                        "minAge": row.get::<_, Option<i64>>(10)?,
                        "maxAge": row.get::<_, Option<i64>>(11)?,
                    }))
                },
            )
            .optional()?;
        members.push(serde_json::json!({
            "memberId": member_id,
            "memberNumber": member_number,
            "memberName": format!("{first_name} {last_name}"),
            "memberStatus": status,
            "membership": membership,
            "credentials": credentials,
        }));
    }
    Ok(members)
}

fn load_member_credential(
    conn: &Connection,
    organization_id: &str,
    credential_id: &str,
) -> Result<MemberCredentialRecord, DataError> {
    conn.query_row(
        "SELECT c.id, c.member_id, m.member_number,
                m.first_name || ' ' || m.last_name,
                c.code, c.status, c.revision, c.issued_at, c.revoked_at,
                c.created_at, c.updated_at
         FROM member_credentials c
         JOIN members m ON m.id = c.member_id
         WHERE c.organization_id = ?1 AND c.id = ?2",
        params![organization_id, credential_id],
        map_member_credential,
    )
    .optional()?
    .ok_or(DataError::MemberCredentialNotFound)
}

fn map_member_credential(row: &Row<'_>) -> rusqlite::Result<MemberCredentialRecord> {
    Ok(MemberCredentialRecord {
        id: row.get(0)?,
        member_id: row.get(1)?,
        member_number: row.get(2)?,
        member_name: row.get(3)?,
        code: row.get(4)?,
        status: row.get(5)?,
        revision: row.get(6)?,
        issued_at: row.get(7)?,
        revoked_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn load_credential_access_rule(
    conn: &Connection,
    organization_id: &str,
    credential_code: &str,
) -> Result<Option<CredentialAccessRuleRecord>, DataError> {
    conn.query_row(
        "SELECT c.id, c.code, c.status, c.revision,
                m.id, m.member_number,
                m.first_name || ' ' || m.last_name,
                m.status, m.birth_date,
                ms.id, ms.plan_name, ms.status, ms.start_date, ms.end_date,
                ms.access_days, ms.access_start_minute, ms.access_end_minute,
                ms.min_age, ms.max_age
         FROM member_credentials c
         JOIN members m ON m.id = c.member_id
         LEFT JOIN memberships ms ON ms.member_id = m.id
              AND ms.organization_id = m.organization_id
              AND ms.status IN ('scheduled', 'active', 'paused')
         WHERE c.organization_id = ?1 AND c.code = ?2
         ORDER BY ms.created_at DESC
         LIMIT 1",
        params![organization_id, credential_code],
        |row| {
            Ok(CredentialAccessRuleRecord {
                credential_id: row.get(0)?,
                credential_code: row.get(1)?,
                credential_status: row.get(2)?,
                credential_revision: row.get(3)?,
                member_id: row.get(4)?,
                member_number: row.get(5)?,
                member_name: row.get(6)?,
                member_status: row.get(7)?,
                birth_date: row.get(8)?,
                membership_id: row.get(9)?,
                plan_name: row.get(10)?,
                membership_status: row.get(11)?,
                start_date: row.get(12)?,
                end_date: row.get(13)?,
                access_days: row.get(14)?,
                access_start_minute: row.get(15)?,
                access_end_minute: row.get(16)?,
                min_age: row.get(17)?,
                max_age: row.get(18)?,
            })
        },
    )
    .optional()
    .map_err(DataError::from)
}

fn access_event_select(filter: &str) -> String {
    format!(
        "SELECT ae.id, ae.credential_code, ae.member_id, m.member_number,
                CASE WHEN m.id IS NULL THEN NULL
                     ELSE m.first_name || ' ' || m.last_name END,
                ae.membership_id, ms.plan_name, ae.device_id, ae.zone,
                ae.allowed, ae.reason, ae.snapshot_revision,
                ae.idempotency_key, ae.payload_hash,
                ae.reconciliation_status, ae.occurred_at
         FROM access_events ae
         LEFT JOIN members m ON m.id = ae.member_id
         LEFT JOIN memberships ms ON ms.id = ae.membership_id
         {filter}"
    )
}

fn map_access_event(row: &Row<'_>) -> rusqlite::Result<AccessEventRecord> {
    Ok(AccessEventRecord {
        id: row.get(0)?,
        credential_code: row.get(1)?,
        member_id: row.get(2)?,
        member_number: row.get(3)?,
        member_name: row.get(4)?,
        membership_id: row.get(5)?,
        plan_name: row.get(6)?,
        device_id: row.get(7)?,
        zone: row.get(8)?,
        allowed: row.get::<_, i64>(9)? != 0,
        reason: row.get(10)?,
        snapshot_revision: row.get(11)?,
        idempotency_key: row.get(12)?,
        payload_hash: row.get(13)?,
        reconciliation_status: row.get(14)?,
        occurred_at: row.get(15)?,
    })
}

fn load_access_event_by_key(
    conn: &Connection,
    organization_id: &str,
    idempotency_key: &str,
) -> Result<AccessEventRecord, DataError> {
    conn.query_row(
        &access_event_select("WHERE ae.organization_id = ?1 AND ae.idempotency_key = ?2"),
        params![organization_id, idempotency_key],
        map_access_event,
    )
    .optional()?
    .ok_or_else(|| DataError::InvalidValue("access event not found".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbKey, MIGRATIONS};
    use std::path::PathBuf;

    fn test_db(name: &str) -> (Db, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "fitness-gym-access-{name}-{}.db",
            uuid::Uuid::new_v4()
        ));
        let key = DbKey::from_bytes([9u8; 32]);
        let db = Db::open(&path, &key).expect("opens test database");
        (db, path)
    }

    fn setup(db: &Db) -> String {
        let organization_id = db.create_organization("Test Gym").expect("organization");
        db.create_owner(&organization_id, "Owner", "not-used-in-this-test", None)
            .expect("owner");
        organization_id
    }

    fn member_with_membership(db: &Db, organization_id: &str) -> (String, String) {
        let member = db
            .create_member(
                organization_id,
                crate::NewMember {
                    first_name: "Ana".into(),
                    last_name: "López".into(),
                    phone: None,
                    email: None,
                    birth_date: Some("1990-01-01".into()),
                    notes: None,
                },
            )
            .expect("member");
        let plan = db
            .create_membership_plan(
                organization_id,
                crate::NewMembershipPlan {
                    name: "Mensual".into(),
                    description: None,
                    duration_days: 30,
                    price_minor: 50_000,
                    grace_days: 0,
                    access_days: "mon,tue,wed,thu,fri,sat,sun".into(),
                    access_start_minute: 0,
                    access_end_minute: 1440,
                    min_age: None,
                    max_age: None,
                },
            )
            .expect("plan");
        let actor_id = db
            .list_staff(organization_id)
            .expect("staff")
            .into_iter()
            .find(|staff| staff.role == "owner")
            .expect("owner")
            .user_id;
        let membership = db
            .create_membership(organization_id, &actor_id, &member.id, &plan.id, None)
            .expect("membership");
        db.issue_member_credential(organization_id, &member.id)
            .expect("credential");
        (member.id, membership.id)
    }

    #[test]
    fn schema_19_is_applied() {
        let (db, path) = test_db("schema");
        let status = db.status().expect("status");
        assert_eq!(status.schema_version, MIGRATIONS.last().unwrap().version);
        assert_eq!(status.schema_version, 22);
        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_is_signed_and_revisioned() {
        let (db, path) = test_db("snapshot");
        let organization_id = setup(&db);
        let (member_id, _) = member_with_membership(&db, &organization_id);
        let credentials = db
            .list_member_credentials(&organization_id, &member_id)
            .expect("credentials");
        assert_eq!(credentials.len(), 1);
        assert!(credentials[0].code.starts_with("GYM-"));

        let snapshot = db
            .publish_access_snapshot(&organization_id, b"test-signing-key")
            .expect("snapshot");
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.member_count, 1);
        assert_eq!(snapshot.credential_count, 1);
        assert_eq!(
            snapshot.checksum,
            sha256_hex(snapshot.payload_json.as_bytes())
        );
        assert_eq!(
            snapshot.signature,
            sign_snapshot(b"test-signing-key", &snapshot.payload_json)
        );
        let payload: serde_json::Value =
            serde_json::from_str(&snapshot.payload_json).expect("payload");
        assert_eq!(
            payload["members"][0]["memberId"].as_str(),
            Some(member_id.as_str())
        );

        let second = db
            .publish_access_snapshot(&organization_id, b"test-signing-key")
            .expect("second snapshot");
        assert_eq!(second.revision, 2);
        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn access_events_are_idempotent_and_marked_stale() {
        let (db, path) = test_db("events");
        let organization_id = setup(&db);
        let (member_id, _) = member_with_membership(&db, &organization_id);
        let code = db
            .list_member_credentials(&organization_id, &member_id)
            .expect("credentials")[0]
            .code
            .clone();
        let snapshot = db
            .publish_access_snapshot(&organization_id, b"key")
            .expect("snapshot");

        let event = NewAccessEvent {
            credential_code: code.clone(),
            device_id: "reader-1".into(),
            zone: Some("Piso 1".into()),
            allowed: true,
            reason: "acceso autorizado".into(),
            snapshot_revision: snapshot.revision,
            occurred_at: "2026-10-26T10:00:00Z".into(),
            idempotency_key: "event-1".into(),
        };
        let outcome = db
            .record_access_event(&organization_id, event.clone())
            .expect("records");
        assert!(matches!(outcome, AccessEventWriteOutcome::Inserted(_)));

        let duplicate = db
            .record_access_event(&organization_id, event.clone())
            .expect("duplicate");
        assert!(matches!(duplicate, AccessEventWriteOutcome::Duplicate(_)));

        let conflict = db
            .record_access_event(
                &organization_id,
                NewAccessEvent {
                    reason: "otro motivo".into(),
                    ..event.clone()
                },
            )
            .expect("conflict");
        assert!(matches!(conflict, AccessEventWriteOutcome::Conflict));

        let stale = db
            .record_access_event(
                &organization_id,
                NewAccessEvent {
                    snapshot_revision: snapshot.revision - 1,
                    idempotency_key: "event-2".into(),
                    occurred_at: "2026-10-26T10:05:00Z".into(),
                    ..event
                },
            )
            .expect("stale");
        match stale {
            AccessEventWriteOutcome::Inserted(event) => {
                assert_eq!(event.reconciliation_status, "stale");
                assert_eq!(event.member_id.as_deref(), Some(member_id.as_str()));
            }
            _ => panic!("expected inserted stale event"),
        }

        let listed = db.list_access_events(&organization_id, 10).expect("events");
        assert_eq!(listed.len(), 2);
        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn attendance_groups_by_hour_plan_zone_and_detects_low_demand() {
        let (db, path) = test_db("attendance");
        let organization_id = setup(&db);
        let (member_id, membership_id) = member_with_membership(&db, &organization_id);
        let code = db
            .list_member_credentials(&organization_id, &member_id)
            .expect("credentials")[0]
            .code
            .clone();
        let snapshot = db
            .publish_access_snapshot(&organization_id, b"key")
            .expect("snapshot");
        for (index, hour) in [6, 6, 7, 20].iter().enumerate() {
            db.record_access_event(
                &organization_id,
                NewAccessEvent {
                    credential_code: code.clone(),
                    device_id: "reader-1".into(),
                    zone: Some(if *hour < 12 { "Matutino" } else { "Vespertino" }.into()),
                    allowed: true,
                    reason: "acceso autorizado".into(),
                    snapshot_revision: snapshot.revision,
                    occurred_at: format!("2026-10-26T{hour:02}:00:00Z"),
                    idempotency_key: format!("attendance-{index}"),
                },
            )
            .expect("event");
        }
        let report = db
            .attendance_report(
                &organization_id,
                "2026-10-26T00:00:00Z",
                "2026-10-27T00:00:00Z",
            )
            .expect("attendance");
        assert_eq!(report.total_visits, 4);
        assert_eq!(report.unique_members, 1);
        assert_eq!(report.peak_hour, 6);
        assert_eq!(report.hours.len(), 24);
        assert!(report.low_demand_hours.iter().any(|hour| hour.hour == 20));
        assert_eq!(report.plans[0].visits, 4);
        assert_eq!(report.zones.len(), 2);
        assert_eq!(report.members[0].member_id, member_id);
        assert_eq!(report.members[0].visits, 4);
        assert_eq!(report.days[0].date, "2026-10-26");
        assert_eq!(
            db.access_rule_for_credential(&organization_id, &code)
                .expect("rule")
                .expect("credential")
                .membership_id
                .as_deref(),
            Some(membership_id.as_str())
        );
        drop(db);
        let _ = std::fs::remove_file(path);
    }
}
