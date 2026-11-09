use super::Db;
use crate::error::DataError;
use rusqlite::{params, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportScheduleRecord {
    pub id: String,
    pub report_type: String,
    pub cadence: String,
    pub lookback_days: i64,
    pub enabled: bool,
    pub next_run_at: String,
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewReportSchedule {
    pub cadence: String,
    pub lookback_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRunRecord {
    pub id: String,
    pub schedule_id: Option<String>,
    pub report_type: String,
    pub period_from: String,
    pub period_to: String,
    pub generated_at: String,
    pub file_name: Option<String>,
    pub checksum: String,
    pub status: String,
    pub error: Option<String>,
}

impl Db {
    pub fn create_report_schedule(
        &self,
        organization_id: &str,
        schedule: NewReportSchedule,
    ) -> Result<ReportScheduleRecord, DataError> {
        let cadence = validate_cadence(&schedule.cadence)?;
        if !(1..=366).contains(&schedule.lookback_days) {
            return Err(DataError::InvalidValue(
                "report lookback must be between 1 and 366 days".into(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        self.with_connection(|conn| {
            conn.execute(
                "INSERT INTO report_schedules
                     (id, organization_id, report_type, cadence, lookback_days,
                      enabled, next_run_at, last_run_at, created_at, updated_at)
                 VALUES (?1, ?2, 'financial', ?3, ?4, 1, ?5, NULL, ?5, ?5)",
                params![id, organization_id, cadence, schedule.lookback_days, now],
            )?;
            load_report_schedule(conn, organization_id, &id)
        })
    }

    pub fn list_report_schedules(
        &self,
        organization_id: &str,
    ) -> Result<Vec<ReportScheduleRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, report_type, cadence, lookback_days, enabled,
                        next_run_at, last_run_at, created_at, updated_at
                 FROM report_schedules
                 WHERE organization_id = ?1
                 ORDER BY enabled DESC, created_at DESC",
            )?;
            let rows = statement
                .query_map(params![organization_id], map_report_schedule)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn due_report_schedules(
        &self,
        organization_id: &str,
        due_at: &str,
    ) -> Result<Vec<ReportScheduleRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, report_type, cadence, lookback_days, enabled,
                        next_run_at, last_run_at, created_at, updated_at
                 FROM report_schedules
                 WHERE organization_id = ?1 AND enabled = 1
                   AND julianday(next_run_at) <= julianday(?2)
                 ORDER BY next_run_at, created_at",
            )?;
            let rows = statement
                .query_map(params![organization_id, due_at], map_report_schedule)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn set_report_schedule_enabled(
        &self,
        organization_id: &str,
        schedule_id: &str,
        enabled: bool,
    ) -> Result<ReportScheduleRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.with_connection(|conn| {
            let changed = conn.execute(
                "UPDATE report_schedules
                 SET enabled = ?1, updated_at = ?2
                 WHERE organization_id = ?3 AND id = ?4",
                params![i64::from(enabled), now, organization_id, schedule_id],
            )?;
            if changed == 0 {
                return Err(DataError::ReportScheduleNotFound);
            }
            load_report_schedule(conn, organization_id, schedule_id)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_report_run(
        &self,
        organization_id: &str,
        schedule_id: Option<&str>,
        report_type: &str,
        period_from: &str,
        period_to: &str,
        generated_at: &str,
        file_name: Option<&str>,
        checksum: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<ReportRunRecord, DataError> {
        if status != "completed" && status != "failed" {
            return Err(DataError::InvalidValue(
                "report run status must be completed or failed".into(),
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.with_connection(|conn| {
            conn.execute(
                "INSERT INTO report_runs
                     (id, organization_id, schedule_id, report_type, period_from,
                      period_to, generated_at, file_name, checksum, status, error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id,
                    organization_id,
                    schedule_id,
                    report_type,
                    period_from,
                    period_to,
                    generated_at,
                    file_name,
                    checksum,
                    status,
                    error
                ],
            )?;
            if status == "completed" {
                if let Some(schedule_id) = schedule_id {
                    if let Some(schedule) =
                        load_report_schedule_optional(conn, organization_id, schedule_id)?
                    {
                        let next_run_at = next_run_at(&schedule.cadence, generated_at)?;
                        conn.execute(
                            "UPDATE report_schedules
                             SET last_run_at = ?1, next_run_at = ?2, updated_at = ?1
                             WHERE organization_id = ?3 AND id = ?4",
                            params![generated_at, next_run_at, organization_id, schedule_id],
                        )?;
                    }
                }
            }
            conn.query_row(
                "SELECT id, schedule_id, report_type, period_from, period_to,
                        generated_at, file_name, checksum, status, error
                 FROM report_runs WHERE id = ?1",
                params![id],
                map_report_run,
            )
            .map_err(DataError::from)
        })
    }

    pub fn list_report_runs(
        &self,
        organization_id: &str,
        limit: usize,
    ) -> Result<Vec<ReportRunRecord>, DataError> {
        let limit = limit.clamp(1, 100) as i64;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, schedule_id, report_type, period_from, period_to,
                        generated_at, file_name, checksum, status, error
                 FROM report_runs
                 WHERE organization_id = ?1
                 ORDER BY generated_at DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = statement
                .query_map(params![organization_id, limit], map_report_run)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}

fn validate_cadence(value: &str) -> Result<String, DataError> {
    match value.trim() {
        "daily" | "weekly" | "monthly" => Ok(value.trim().to_string()),
        _ => Err(DataError::InvalidValue(
            "report cadence must be daily, weekly or monthly".into(),
        )),
    }
}

fn next_run_at(cadence: &str, from: &str) -> Result<String, DataError> {
    let from = chrono::DateTime::parse_from_rfc3339(from)
        .map(|value| value.with_timezone(&chrono::Utc))
        .map_err(|_| DataError::InvalidValue("report timestamp must be RFC 3339".into()))?;
    let days = match cadence {
        "daily" => 1,
        "weekly" => 7,
        "monthly" => 30,
        _ => {
            return Err(DataError::InvalidValue(
                "report cadence must be daily, weekly or monthly".into(),
            ))
        }
    };
    Ok((from + chrono::TimeDelta::days(days)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

fn load_report_schedule(
    conn: &rusqlite::Connection,
    organization_id: &str,
    schedule_id: &str,
) -> Result<ReportScheduleRecord, DataError> {
    load_report_schedule_optional(conn, organization_id, schedule_id)?
        .ok_or(DataError::ReportScheduleNotFound)
}

fn load_report_schedule_optional(
    conn: &rusqlite::Connection,
    organization_id: &str,
    schedule_id: &str,
) -> Result<Option<ReportScheduleRecord>, DataError> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT id, report_type, cadence, lookback_days, enabled,
                next_run_at, last_run_at, created_at, updated_at
         FROM report_schedules
         WHERE organization_id = ?1 AND id = ?2",
        params![organization_id, schedule_id],
        map_report_schedule,
    )
    .optional()
    .map_err(DataError::from)
}

fn map_report_schedule(row: &Row<'_>) -> rusqlite::Result<ReportScheduleRecord> {
    Ok(ReportScheduleRecord {
        id: row.get(0)?,
        report_type: row.get(1)?,
        cadence: row.get(2)?,
        lookback_days: row.get(3)?,
        enabled: row.get::<_, i64>(4)? != 0,
        next_run_at: row.get(5)?,
        last_run_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn map_report_run(row: &Row<'_>) -> rusqlite::Result<ReportRunRecord> {
    Ok(ReportRunRecord {
        id: row.get(0)?,
        schedule_id: row.get(1)?,
        report_type: row.get(2)?,
        period_from: row.get(3)?,
        period_to: row.get(4)?,
        generated_at: row.get(5)?,
        file_name: row.get(6)?,
        checksum: row.get(7)?,
        status: row.get(8)?,
        error: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbKey, MIGRATIONS};
    use std::path::PathBuf;

    fn test_db(name: &str) -> (Db, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "fitness-gym-reports-{name}-{}.db",
            uuid::Uuid::new_v4()
        ));
        let key = DbKey::from_bytes([11u8; 32]);
        let db = Db::open(&path, &key).expect("opens test database");
        (db, path)
    }

    fn setup(db: &Db) -> String {
        let organization_id = db.create_organization("Test Gym").expect("organization");
        db.create_owner(&organization_id, "Owner", "not-used-in-this-test", None)
            .expect("owner");
        organization_id
    }

    #[test]
    fn schedule_runs_advance_and_are_idempotent_per_period() {
        let (db, path) = test_db("schedule");
        let organization_id = setup(&db);
        let schedule = db
            .create_report_schedule(
                &organization_id,
                NewReportSchedule {
                    cadence: "weekly".into(),
                    lookback_days: 7,
                },
            )
            .expect("schedule");
        assert!(schedule.enabled);
        assert_eq!(schedule.report_type, "financial");
        assert_eq!(schedule.lookback_days, 7);

        let due = db
            .due_report_schedules(
                &organization_id,
                &(chrono::Utc::now() + chrono::TimeDelta::days(1)).to_rfc3339(),
            )
            .expect("due");
        assert_eq!(due.len(), 1);

        let generated_at = "2026-10-26T12:00:00.000Z";
        let run = db
            .record_report_run(
                &organization_id,
                Some(&schedule.id),
                "financial",
                "2026-10-19T12:00:00.000Z",
                generated_at,
                generated_at,
                Some("reporte.csv"),
                "abc123",
                "completed",
                None,
            )
            .expect("run");
        assert_eq!(run.status, "completed");
        assert_eq!(run.checksum, "abc123");

        let schedules = db
            .list_report_schedules(&organization_id)
            .expect("schedules");
        assert_eq!(schedules[0].last_run_at.as_deref(), Some(generated_at));
        assert_eq!(
            schedules[0].next_run_at,
            "2026-11-02T12:00:00.000Z".to_string()
        );

        let future = db
            .due_report_schedules(&organization_id, "2026-10-27T00:00:00Z")
            .expect("future");
        assert!(future.is_empty());

        let runs = db.list_report_runs(&organization_id, 10).expect("runs");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].schedule_id.as_deref(), Some(schedule.id.as_str()));

        db.set_report_schedule_enabled(&organization_id, &schedule.id, false)
            .expect("disable");
        let disabled = db
            .list_report_schedules(&organization_id)
            .expect("schedules");
        assert!(!disabled[0].enabled);

        let status = db.status().expect("status");
        assert_eq!(status.schema_version, MIGRATIONS.last().unwrap().version);
        assert_eq!(status.schema_version, 22);
        drop(db);
        let _ = std::fs::remove_file(path);
    }
}
