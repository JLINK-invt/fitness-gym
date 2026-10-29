use rusqlite::{params, OptionalExtension, Row};

use super::{enqueue_sync_event, normalize_optional_text, required_text, Db};
use crate::error::DataError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GymSettingsRecord {
    pub id: String,
    pub business_name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub locale: String,
    pub opening_time: String,
    pub closing_time: String,
    pub slot_minutes: i64,
    pub grace_minutes: i64,
    pub inactivity_days: i64,
    pub frozen_days: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateGymSettings {
    pub business_name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub locale: String,
    pub opening_time: String,
    pub closing_time: String,
    pub slot_minutes: i64,
    pub grace_minutes: i64,
    pub inactivity_days: i64,
    pub frozen_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationLinkRecord {
    pub id: String,
    pub module_id: String,
    pub display_name: String,
    pub contract_version: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub last_seen_at: Option<String>,
    pub last_event_at: Option<String>,
    pub last_snapshot_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewIntegrationLink {
    pub module_id: String,
    pub display_name: String,
    pub contract_version: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationJournalSummary {
    pub pending: i64,
    pub sent: i64,
    pub dead_letter: i64,
    pub inbox_events: i64,
    pub last_server_sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRecord {
    pub id: String,
    pub name: String,
    pub coach_user_id: Option<String>,
    pub coach_name: Option<String>,
    pub weekday: i64,
    pub start_time: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub room: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub booked: i64,
    pub attended: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewClass {
    pub name: String,
    pub coach_user_id: Option<String>,
    pub weekday: i64,
    pub start_time: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub room: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateClass {
    pub name: String,
    pub coach_user_id: Option<String>,
    pub weekday: i64,
    pub start_time: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub room: Option<String>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassBookingRecord {
    pub id: String,
    pub class_id: String,
    pub class_name: String,
    pub member_id: String,
    pub member_name: String,
    pub status: String,
    pub note: Option<String>,
    pub booked_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignRecord {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub audience: String,
    pub status: String,
    pub starts_on: Option<String>,
    pub ends_on: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCampaign {
    pub name: String,
    pub channel: String,
    pub audience: String,
    pub status: String,
    pub starts_on: Option<String>,
    pub ends_on: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunnelStageRecord {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketingFunnelRecord {
    pub prospects_total: i64,
    pub prospects_converted: i64,
    pub conversion_bps: i64,
    pub stages: Vec<FunnelStageRecord>,
    pub active_campaigns: i64,
    pub expiring_memberships: i64,
    pub inactive_members: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentRecord {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub area: Option<String>,
    pub status: String,
    pub acquired_on: Option<String>,
    pub notes: Option<String>,
    pub open_orders: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEquipment {
    pub name: String,
    pub code: Option<String>,
    pub area: Option<String>,
    pub acquired_on: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEquipment {
    pub name: String,
    pub code: Option<String>,
    pub area: Option<String>,
    pub status: String,
    pub acquired_on: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceOrderRecord {
    pub id: String,
    pub equipment_id: Option<String>,
    pub equipment_name: Option<String>,
    pub kind: String,
    pub status: String,
    pub priority: String,
    pub summary: String,
    pub detail: Option<String>,
    pub due_on: Option<String>,
    pub reported_by: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMaintenanceOrder {
    pub equipment_id: Option<String>,
    pub kind: String,
    pub priority: String,
    pub summary: String,
    pub detail: Option<String>,
    pub due_on: Option<String>,
    pub reported_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceOrderUpdate {
    pub status: String,
    pub priority: String,
    pub summary: String,
    pub detail: Option<String>,
    pub due_on: Option<String>,
}

impl Db {
    pub fn gym_settings(&self, organization_id: &str) -> Result<GymSettingsRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let exists: Option<String> = conn
                .query_row(
                    "SELECT id FROM gym_settings WHERE organization_id = ?1",
                    params![organization_id],
                    |row| row.get(0),
                )
                .optional()?;
            if exists.is_none() {
                let (name, currency): (String, String) = conn
                    .query_row(
                        "SELECT name, currency FROM organizations WHERE id = ?1",
                        params![organization_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?
                    .ok_or_else(|| DataError::InvalidValue("the gym does not exist".into()))?;
                conn.execute(
                    "INSERT INTO gym_settings
                         (id, organization_id, business_name, currency, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        organization_id,
                        name,
                        currency,
                        now
                    ],
                )?;
            }
        }
        self.with_connection(|conn| load_gym_settings(conn, organization_id))
    }

    pub fn update_gym_settings(
        &self,
        organization_id: &str,
        update: UpdateGymSettings,
    ) -> Result<GymSettingsRecord, DataError> {
        self.gym_settings(organization_id)?;
        let business_name = required_text(&update.business_name, "business name", 1, 120)?;
        let legal_name = normalize_optional_text(update.legal_name, 160, "legal name")?;
        let tax_id = normalize_optional_text(update.tax_id, 40, "tax id")?;
        let currency = validate_currency(&update.currency)?;
        let timezone = required_text(&update.timezone, "timezone", 1, 60)?;
        let locale = required_text(&update.locale, "locale", 2, 20)?;
        let opening_time = validate_time(&update.opening_time, "opening time")?;
        let closing_time = validate_time(&update.closing_time, "closing time")?;
        let slot_minutes = validate_range(update.slot_minutes, 15, 240, "slot minutes")?;
        let grace_minutes = validate_range(update.grace_minutes, 0, 120, "grace minutes")?;
        let inactivity_days = validate_range(update.inactivity_days, 1, 365, "inactivity days")?;
        let frozen_days = validate_range(update.frozen_days, 0, 365, "frozen days")?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            transaction.execute(
                "UPDATE gym_settings
                 SET business_name = ?1, legal_name = ?2, tax_id = ?3, currency = ?4,
                     timezone = ?5, locale = ?6, opening_time = ?7, closing_time = ?8,
                     slot_minutes = ?9, grace_minutes = ?10, inactivity_days = ?11,
                     frozen_days = ?12, updated_at = ?13
                 WHERE organization_id = ?14",
                params![
                    business_name,
                    legal_name,
                    tax_id,
                    currency,
                    timezone,
                    locale,
                    opening_time,
                    closing_time,
                    slot_minutes,
                    grace_minutes,
                    inactivity_days,
                    frozen_days,
                    now,
                    organization_id
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "settings.updated",
                "gym_settings",
                organization_id,
                &format!("settings:{organization_id}:{now}"),
                &serde_json::json!({
                    "businessName": business_name,
                    "currency": currency,
                    "timezone": timezone,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_gym_settings(conn, organization_id))
    }

    pub fn list_integration_links(
        &self,
        organization_id: &str,
    ) -> Result<Vec<IntegrationLinkRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, module_id, display_name, contract_version, status, capabilities,
                        last_seen_at, last_event_at, last_snapshot_at, notes, created_at,
                        updated_at
                 FROM integration_links
                 WHERE organization_id = ?1
                 ORDER BY display_name COLLATE NOCASE",
            )?;
            let links = statement
                .query_map(params![organization_id], map_integration_link)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(links)
        })
    }

    pub fn upsert_integration_link(
        &self,
        organization_id: &str,
        link: NewIntegrationLink,
    ) -> Result<IntegrationLinkRecord, DataError> {
        let module_id = required_text(&link.module_id, "module id", 2, 60)?;
        let display_name = required_text(&link.display_name, "display name", 1, 80)?;
        let contract_version = required_text(&link.contract_version, "contract version", 1, 20)?;
        let status = validate_choice(
            &link.status,
            &["connected", "degraded", "disconnected"],
            "integration status",
        )?;
        let capabilities = link.capabilities;
        if capabilities.len() > 12 {
            return Err(DataError::InvalidValue(
                "the module cannot declare more than 12 capabilities".into(),
            ));
        }
        let capabilities: Vec<String> = capabilities
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty() && value.chars().count() <= 40)
            .collect();
        let notes = normalize_optional_text(link.notes, 300, "notes")?;
        let capabilities_json = serde_json::to_string(&capabilities).map_err(DataError::Json)?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            transaction.execute(
                "INSERT INTO integration_links
                     (id, organization_id, module_id, display_name, contract_version, status,
                      capabilities, last_seen_at, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
                 ON CONFLICT (organization_id, module_id) DO UPDATE SET
                     display_name = excluded.display_name,
                     contract_version = excluded.contract_version,
                     status = excluded.status,
                     capabilities = excluded.capabilities,
                     last_seen_at = excluded.last_seen_at,
                     notes = excluded.notes,
                     updated_at = excluded.updated_at",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    organization_id,
                    module_id,
                    display_name,
                    contract_version,
                    status,
                    capabilities_json,
                    now,
                    notes,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "integration.linked",
                "integration_link",
                &module_id,
                &format!("integration:{module_id}:{now}"),
                &serde_json::json!({
                    "moduleId": module_id,
                    "status": status,
                    "contractVersion": contract_version,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT id, module_id, display_name, contract_version, status, capabilities,
                        last_seen_at, last_event_at, last_snapshot_at, notes, created_at,
                        updated_at
                 FROM integration_links
                 WHERE organization_id = ?1 AND module_id = ?2",
                params![organization_id, module_id],
                map_integration_link,
            )
            .optional()?
            .ok_or_else(|| DataError::InvalidValue("the integration link was not saved".into()))
        })
    }

    pub fn integration_journal_summary(
        &self,
        organization_id: &str,
    ) -> Result<IntegrationJournalSummary, DataError> {
        self.with_connection(|conn| {
            let counts = |state: &str| -> Result<i64, DataError> {
                Ok(conn.query_row(
                    "SELECT count(*) FROM sync_outbox WHERE organization_id = ?1 AND state = ?2",
                    params![organization_id, state],
                    |row| row.get(0),
                )?)
            };
            let inbox_events = conn.query_row(
                "SELECT count(*) FROM sync_inbox WHERE organization_id = ?1",
                params![organization_id],
                |row| row.get(0),
            )?;
            let last_server_sequence = conn
                .query_row(
                    "SELECT last_server_sequence FROM sync_state WHERE organization_id = ?1",
                    params![organization_id],
                    |row| row.get(0),
                )
                .optional()?
                .unwrap_or(0);
            Ok(IntegrationJournalSummary {
                pending: counts("pending")?,
                sent: counts("sent")?,
                dead_letter: counts("dead_letter")?,
                inbox_events,
                last_server_sequence,
            })
        })
    }

    pub fn list_classes(&self, organization_id: &str) -> Result<Vec<ClassRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT c.id, c.name, c.coach_user_id, u.name, c.weekday, c.start_time,
                        c.duration_minutes, c.capacity, c.room, c.status, c.notes,
                        (SELECT count(*) FROM class_bookings b
                          WHERE b.class_id = c.id AND b.status IN ('booked', 'attended')),
                        (SELECT count(*) FROM class_bookings b
                          WHERE b.class_id = c.id AND b.status = 'attended'),
                        c.created_at, c.updated_at
                 FROM classes c
                 LEFT JOIN users u ON u.id = c.coach_user_id
                 WHERE c.organization_id = ?1
                 ORDER BY c.weekday, c.start_time, c.name COLLATE NOCASE",
            )?;
            let classes = statement
                .query_map(params![organization_id], map_class)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(classes)
        })
    }

    pub fn create_class(
        &self,
        organization_id: &str,
        class: NewClass,
    ) -> Result<ClassRecord, DataError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let validated = validate_class(&transaction, organization_id, NewClass { ..class })?;
            transaction.execute(
                "INSERT INTO classes
                     (id, organization_id, name, coach_user_id, weekday, start_time,
                      duration_minutes, capacity, room, status, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'active', ?10, ?11, ?11)",
                params![
                    id,
                    organization_id,
                    validated.name,
                    validated.coach_user_id,
                    validated.weekday,
                    validated.start_time,
                    validated.duration_minutes,
                    validated.capacity,
                    validated.room,
                    validated.notes,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "class.created",
                "class",
                &id,
                &format!("class:{id}:created"),
                &serde_json::json!({
                    "classId": id,
                    "name": validated.name,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.class(organization_id, &id)
    }

    pub fn update_class(
        &self,
        organization_id: &str,
        class_id: &str,
        update: UpdateClass,
    ) -> Result<ClassRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            transaction
                .query_row(
                    "SELECT id FROM classes WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, class_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .ok_or(DataError::ClassNotFound)?;
            let status = validate_choice(
                &update.status,
                &["active", "paused", "cancelled"],
                "class status",
            )?;
            let validated = validate_class(
                &transaction,
                organization_id,
                NewClass {
                    name: update.name,
                    coach_user_id: update.coach_user_id,
                    weekday: update.weekday,
                    start_time: update.start_time,
                    duration_minutes: update.duration_minutes,
                    capacity: update.capacity,
                    room: update.room,
                    notes: update.notes,
                },
            )?;
            let booked: i64 = transaction.query_row(
                "SELECT count(*) FROM class_bookings
                 WHERE class_id = ?1 AND status IN ('booked', 'attended')",
                params![class_id],
                |row| row.get(0),
            )?;
            if validated.capacity < booked {
                return Err(DataError::InvalidValue(
                    "the capacity cannot be lower than the current bookings".into(),
                ));
            }
            transaction.execute(
                "UPDATE classes
                 SET name = ?1, coach_user_id = ?2, weekday = ?3, start_time = ?4,
                     duration_minutes = ?5, capacity = ?6, room = ?7, status = ?8,
                     notes = ?9, updated_at = ?10
                 WHERE organization_id = ?11 AND id = ?12",
                params![
                    validated.name,
                    validated.coach_user_id,
                    validated.weekday,
                    validated.start_time,
                    validated.duration_minutes,
                    validated.capacity,
                    validated.room,
                    status,
                    validated.notes,
                    now,
                    organization_id,
                    class_id
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "class.updated",
                "class",
                class_id,
                &format!("class:{class_id}:updated:{now}"),
                &serde_json::json!({
                    "classId": class_id,
                    "status": status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.class(organization_id, class_id)
    }

    pub fn class(&self, organization_id: &str, class_id: &str) -> Result<ClassRecord, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT c.id, c.name, c.coach_user_id, u.name, c.weekday, c.start_time,
                        c.duration_minutes, c.capacity, c.room, c.status, c.notes,
                        (SELECT count(*) FROM class_bookings b
                          WHERE b.class_id = c.id AND b.status IN ('booked', 'attended')),
                        (SELECT count(*) FROM class_bookings b
                          WHERE b.class_id = c.id AND b.status = 'attended'),
                        c.created_at, c.updated_at
                 FROM classes c
                 LEFT JOIN users u ON u.id = c.coach_user_id
                 WHERE c.organization_id = ?1 AND c.id = ?2",
                params![organization_id, class_id],
                map_class,
            )
            .optional()?
            .ok_or(DataError::ClassNotFound)
        })
    }

    pub fn list_class_bookings(
        &self,
        organization_id: &str,
        class_id: &str,
    ) -> Result<Vec<ClassBookingRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT b.id, b.class_id, c.name, b.member_id,
                        m.first_name || ' ' || m.last_name, b.status, b.note,
                        b.booked_at, b.updated_at
                 FROM class_bookings b
                 JOIN classes c ON c.id = b.class_id
                 JOIN members m ON m.id = b.member_id
                 WHERE b.organization_id = ?1 AND b.class_id = ?2
                 ORDER BY CASE b.status
                     WHEN 'booked' THEN 0
                     WHEN 'attended' THEN 1
                     WHEN 'no_show' THEN 2
                     ELSE 3
                 END, m.last_name COLLATE NOCASE, m.first_name COLLATE NOCASE",
            )?;
            let bookings = statement
                .query_map(params![organization_id, class_id], map_class_booking)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(bookings)
        })
    }

    pub fn book_class(
        &self,
        organization_id: &str,
        class_id: &str,
        member_id: &str,
        note: Option<String>,
    ) -> Result<ClassBookingRecord, DataError> {
        let note = normalize_optional_text(note, 200, "note")?;
        let now = chrono::Utc::now().to_rfc3339();
        let booking_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let (capacity, class_status): (i64, String) = transaction
                .query_row(
                    "SELECT capacity, status FROM classes
                     WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, class_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?
                .ok_or(DataError::ClassNotFound)?;
            if class_status != "active" {
                return Err(DataError::InvalidValue(
                    "only active classes accept new bookings".into(),
                ));
            }
            let member_exists: Option<String> = transaction
                .query_row(
                    "SELECT id FROM members WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, member_id],
                    |row| row.get(0),
                )
                .optional()?;
            if member_exists.is_none() {
                return Err(DataError::MemberNotFound);
            }
            let existing: Option<(String, String)> = transaction
                .query_row(
                    "SELECT id, status FROM class_bookings
                     WHERE class_id = ?1 AND member_id = ?2",
                    params![class_id, member_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            if let Some((id, status)) = existing {
                if status != "cancelled" {
                    return Err(DataError::InvalidValue(
                        "the member already has a booking for this class".into(),
                    ));
                }
                transaction.execute(
                    "UPDATE class_bookings
                     SET status = 'booked', note = ?1, booked_at = ?2, updated_at = ?2
                     WHERE id = ?3",
                    params![note, now, id],
                )?;
                enqueue_sync_event(
                    &transaction,
                    organization_id,
                    "class.booking.updated",
                    "class_booking",
                    &id,
                    &format!("class_booking:{id}:updated"),
                    &serde_json::json!({
                        "bookingId": id,
                        "classId": class_id,
                        "memberId": member_id,
                        "status": "booked",
                        "updatedAt": now,
                    }),
                    &now,
                )?;
                transaction.commit()?;
                return self.with_connection(|conn| load_class_booking(conn, organization_id, &id));
            }
            let booked: i64 = transaction.query_row(
                "SELECT count(*) FROM class_bookings
                 WHERE class_id = ?1 AND status IN ('booked', 'attended')",
                params![class_id],
                |row| row.get(0),
            )?;
            if booked >= capacity {
                return Err(DataError::InvalidValue(
                    "the class has no free spots left".into(),
                ));
            }
            let id = uuid::Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO class_bookings
                     (id, organization_id, class_id, member_id, status, note,
                      booked_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'booked', ?5, ?6, ?6)",
                params![id, organization_id, class_id, member_id, note, now],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "class.booking.created",
                "class_booking",
                &id,
                &format!("class_booking:{id}:created"),
                &serde_json::json!({
                    "bookingId": id,
                    "classId": class_id,
                    "memberId": member_id,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
            id
        };
        self.with_connection(|conn| load_class_booking(conn, organization_id, &booking_id))
    }

    pub fn update_class_booking(
        &self,
        organization_id: &str,
        booking_id: &str,
        status: &str,
    ) -> Result<ClassBookingRecord, DataError> {
        let status = validate_choice(
            status,
            &["booked", "attended", "cancelled", "no_show"],
            "booking status",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let changed = transaction.execute(
                "UPDATE class_bookings
                 SET status = ?1, updated_at = ?2
                 WHERE organization_id = ?3 AND id = ?4",
                params![status, now, organization_id, booking_id],
            )?;
            if changed == 0 {
                return Err(DataError::ClassBookingNotFound);
            }
            enqueue_sync_event(
                &transaction,
                organization_id,
                "class.booking.updated",
                "class_booking",
                booking_id,
                &format!("class_booking:{booking_id}:updated:{now}"),
                &serde_json::json!({
                    "bookingId": booking_id,
                    "status": status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_class_booking(conn, organization_id, booking_id))
    }

    pub fn list_campaigns(&self, organization_id: &str) -> Result<Vec<CampaignRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, name, channel, audience, status, starts_on, ends_on, notes,
                        created_at, updated_at
                 FROM campaigns
                 WHERE organization_id = ?1
                 ORDER BY CASE status
                     WHEN 'active' THEN 0
                     WHEN 'draft' THEN 1
                     WHEN 'paused' THEN 2
                     ELSE 3
                 END, created_at DESC",
            )?;
            let campaigns = statement
                .query_map(params![organization_id], map_campaign)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(campaigns)
        })
    }

    pub fn create_campaign(
        &self,
        organization_id: &str,
        campaign: NewCampaign,
    ) -> Result<CampaignRecord, DataError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let validated = validate_campaign(campaign)?;
            transaction.execute(
                "INSERT INTO campaigns
                     (id, organization_id, name, channel, audience, status, starts_on,
                      ends_on, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    id,
                    organization_id,
                    validated.name,
                    validated.channel,
                    validated.audience,
                    validated.status,
                    validated.starts_on,
                    validated.ends_on,
                    validated.notes,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "campaign.created",
                "campaign",
                &id,
                &format!("campaign:{id}:created"),
                &serde_json::json!({
                    "campaignId": id,
                    "name": validated.name,
                    "status": validated.status,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_campaign(conn, organization_id, &id))
    }

    pub fn update_campaign(
        &self,
        organization_id: &str,
        campaign_id: &str,
        update: NewCampaign,
    ) -> Result<CampaignRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            transaction
                .query_row(
                    "SELECT id FROM campaigns WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, campaign_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .ok_or(DataError::CampaignNotFound)?;
            let validated = validate_campaign(update)?;
            transaction.execute(
                "UPDATE campaigns
                 SET name = ?1, channel = ?2, audience = ?3, status = ?4,
                     starts_on = ?5, ends_on = ?6, notes = ?7, updated_at = ?8
                 WHERE organization_id = ?9 AND id = ?10",
                params![
                    validated.name,
                    validated.channel,
                    validated.audience,
                    validated.status,
                    validated.starts_on,
                    validated.ends_on,
                    validated.notes,
                    now,
                    organization_id,
                    campaign_id
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "campaign.updated",
                "campaign",
                campaign_id,
                &format!("campaign:{campaign_id}:updated:{now}"),
                &serde_json::json!({
                    "campaignId": campaign_id,
                    "status": validated.status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_campaign(conn, organization_id, campaign_id))
    }

    pub fn marketing_funnel(
        &self,
        organization_id: &str,
        inactivity_days: i64,
        expiry_days: i64,
    ) -> Result<MarketingFunnelRecord, DataError> {
        let inactivity_days = validate_range(inactivity_days, 1, 365, "inactivity days")?;
        let expiry_days = validate_range(expiry_days, 1, 365, "expiry window")?;
        let today = chrono::Utc::now().date_naive();
        let expiry_limit = (today + chrono::Duration::days(expiry_days))
            .format("%Y-%m-%d")
            .to_string();
        let inactivity_cutoff = chrono::Utc::now() - chrono::Duration::days(inactivity_days);
        let inactivity_cutoff = inactivity_cutoff.to_rfc3339();
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT status, count(*) FROM prospects
                 WHERE organization_id = ?1
                 GROUP BY status",
            )?;
            let mut stages: Vec<FunnelStageRecord> = statement
                .query_map(params![organization_id], |row| {
                    Ok(FunnelStageRecord {
                        status: row.get(0)?,
                        count: row.get(1)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            stages.sort_by_key(|stage| match stage.status.as_str() {
                "new" => 0,
                "contacted" => 1,
                "trial" => 2,
                "converted" => 3,
                _ => 4,
            });
            let total: i64 = stages.iter().map(|stage| stage.count).sum();
            let converted = stages
                .iter()
                .find(|stage| stage.status == "converted")
                .map(|stage| stage.count)
                .unwrap_or(0);
            let conversion_bps = if total > 0 {
                (converted * 10_000) / total
            } else {
                0
            };
            let active_campaigns = conn.query_row(
                "SELECT count(*) FROM campaigns WHERE organization_id = ?1 AND status = 'active'",
                params![organization_id],
                |row| row.get(0),
            )?;
            let expiring_memberships = conn.query_row(
                "SELECT count(*) FROM memberships
                 WHERE organization_id = ?1 AND status = 'active'
                   AND end_date >= ?2 AND end_date <= ?3",
                params![
                    organization_id,
                    today.format("%Y-%m-%d").to_string(),
                    expiry_limit
                ],
                |row| row.get(0),
            )?;
            let inactive_members = conn.query_row(
                "SELECT count(*) FROM members m
                 WHERE m.organization_id = ?1 AND m.status = 'active'
                   AND NOT EXISTS (
                       SELECT 1 FROM access_events e
                       WHERE e.member_id = m.id AND e.allowed = 1
                         AND e.occurred_at >= ?2
                   )",
                params![organization_id, inactivity_cutoff],
                |row| row.get(0),
            )?;
            Ok(MarketingFunnelRecord {
                prospects_total: total,
                prospects_converted: converted,
                conversion_bps,
                stages,
                active_campaigns,
                expiring_memberships,
                inactive_members,
            })
        })
    }

    pub fn list_equipment(&self, organization_id: &str) -> Result<Vec<EquipmentRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT e.id, e.name, e.code, e.area, e.status, e.acquired_on, e.notes,
                        (SELECT count(*) FROM maintenance_orders o
                          WHERE o.equipment_id = e.id
                            AND o.status IN ('open', 'in_progress')),
                        e.created_at, e.updated_at
                 FROM equipment e
                 WHERE e.organization_id = ?1
                 ORDER BY e.name COLLATE NOCASE",
            )?;
            let equipment = statement
                .query_map(params![organization_id], map_equipment)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(equipment)
        })
    }

    pub fn create_equipment(
        &self,
        organization_id: &str,
        equipment: NewEquipment,
    ) -> Result<EquipmentRecord, DataError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let validated = validate_equipment(equipment)?;
            transaction.execute(
                "INSERT INTO equipment
                     (id, organization_id, name, code, area, status, acquired_on, notes,
                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'operational', ?6, ?7, ?8, ?8)",
                params![
                    id,
                    organization_id,
                    validated.name,
                    validated.code,
                    validated.area,
                    validated.acquired_on,
                    validated.notes,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "equipment.created",
                "equipment",
                &id,
                &format!("equipment:{id}:created"),
                &serde_json::json!({
                    "equipmentId": id,
                    "name": validated.name,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_equipment(conn, organization_id, &id))
    }

    pub fn update_equipment(
        &self,
        organization_id: &str,
        equipment_id: &str,
        update: UpdateEquipment,
    ) -> Result<EquipmentRecord, DataError> {
        let status = validate_choice(
            &update.status,
            &["operational", "maintenance", "out_of_service"],
            "equipment status",
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            transaction
                .query_row(
                    "SELECT id FROM equipment WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, equipment_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .ok_or(DataError::EquipmentNotFound)?;
            let validated = validate_equipment(NewEquipment {
                name: update.name,
                code: update.code,
                area: update.area,
                acquired_on: update.acquired_on,
                notes: update.notes,
            })?;
            transaction.execute(
                "UPDATE equipment
                 SET name = ?1, code = ?2, area = ?3, status = ?4, acquired_on = ?5,
                     notes = ?6, updated_at = ?7
                 WHERE organization_id = ?8 AND id = ?9",
                params![
                    validated.name,
                    validated.code,
                    validated.area,
                    status,
                    validated.acquired_on,
                    validated.notes,
                    now,
                    organization_id,
                    equipment_id
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "equipment.updated",
                "equipment",
                equipment_id,
                &format!("equipment:{equipment_id}:updated:{now}"),
                &serde_json::json!({
                    "equipmentId": equipment_id,
                    "status": status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_equipment(conn, organization_id, equipment_id))
    }

    pub fn list_maintenance_orders(
        &self,
        organization_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<MaintenanceOrderRecord>, DataError> {
        let status = status.map(str::trim).filter(|value| !value.is_empty());
        if let Some(value) = status {
            validate_choice(
                value,
                &["open", "in_progress", "completed", "cancelled"],
                "order status",
            )?;
        }
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT o.id, o.equipment_id, e.name, o.kind, o.status, o.priority,
                        o.summary, o.detail, o.due_on, o.reported_by, o.completed_at,
                        o.created_at, o.updated_at
                 FROM maintenance_orders o
                 LEFT JOIN equipment e ON e.id = o.equipment_id
                 WHERE o.organization_id = ?1 AND (?2 IS NULL OR o.status = ?2)
                 ORDER BY CASE o.status
                     WHEN 'open' THEN 0
                     WHEN 'in_progress' THEN 1
                     WHEN 'completed' THEN 2
                     ELSE 3
                 END,
                 CASE o.priority WHEN 'high' THEN 0 WHEN 'normal' THEN 1 ELSE 2 END,
                 o.due_on IS NULL, o.due_on, o.created_at DESC",
            )?;
            let orders = statement
                .query_map(params![organization_id, status], map_maintenance_order)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(orders)
        })
    }

    pub fn create_maintenance_order(
        &self,
        organization_id: &str,
        order: NewMaintenanceOrder,
    ) -> Result<MaintenanceOrderRecord, DataError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let kind = validate_choice(&order.kind, &["preventive", "corrective"], "order kind")?;
            let priority = validate_choice(
                &order.priority,
                &["low", "normal", "high"],
                "order priority",
            )?;
            let summary = required_text(&order.summary, "summary", 1, 120)?;
            let detail = normalize_optional_text(order.detail, 500, "detail")?;
            let due_on = validate_date(order.due_on, "due date")?;
            if let Some(equipment_id) = &order.equipment_id {
                transaction
                    .query_row(
                        "SELECT id FROM equipment WHERE organization_id = ?1 AND id = ?2",
                        params![organization_id, equipment_id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?
                    .ok_or(DataError::EquipmentNotFound)?;
            }
            transaction.execute(
                "INSERT INTO maintenance_orders
                     (id, organization_id, equipment_id, kind, status, priority, summary,
                      detail, due_on, reported_by, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'open', ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    id,
                    organization_id,
                    order.equipment_id,
                    kind,
                    priority,
                    summary,
                    detail,
                    due_on,
                    order.reported_by,
                    now
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "maintenance.order.created",
                "maintenance_order",
                &id,
                &format!("maintenance_order:{id}:created"),
                &serde_json::json!({
                    "orderId": id,
                    "kind": kind,
                    "priority": priority,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_maintenance_order(conn, organization_id, &id))
    }

    pub fn update_maintenance_order(
        &self,
        organization_id: &str,
        order_id: &str,
        update: MaintenanceOrderUpdate,
    ) -> Result<MaintenanceOrderRecord, DataError> {
        let status = validate_choice(
            &update.status,
            &["open", "in_progress", "completed", "cancelled"],
            "order status",
        )?;
        let priority = validate_choice(
            &update.priority,
            &["low", "normal", "high"],
            "order priority",
        )?;
        let summary = required_text(&update.summary, "summary", 1, 120)?;
        let detail = normalize_optional_text(update.detail, 500, "detail")?;
        let due_on = validate_date(update.due_on, "due date")?;
        let now = chrono::Utc::now().to_rfc3339();
        let completed_at = if status == "completed" {
            Some(now.clone())
        } else {
            None
        };
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let changed = transaction.execute(
                "UPDATE maintenance_orders
                 SET status = ?1, priority = ?2, summary = ?3, detail = ?4, due_on = ?5,
                     completed_at = ?6, updated_at = ?7
                 WHERE organization_id = ?8 AND id = ?9",
                params![
                    status,
                    priority,
                    summary,
                    detail,
                    due_on,
                    completed_at,
                    now,
                    organization_id,
                    order_id
                ],
            )?;
            if changed == 0 {
                return Err(DataError::MaintenanceOrderNotFound);
            }
            enqueue_sync_event(
                &transaction,
                organization_id,
                "maintenance.order.updated",
                "maintenance_order",
                order_id,
                &format!("maintenance_order:{order_id}:updated:{now}"),
                &serde_json::json!({
                    "orderId": order_id,
                    "status": status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_maintenance_order(conn, organization_id, order_id))
    }
}

struct ValidatedClass {
    name: String,
    coach_user_id: Option<String>,
    weekday: i64,
    start_time: String,
    duration_minutes: i64,
    capacity: i64,
    room: Option<String>,
    notes: Option<String>,
}

fn validate_class(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    class: NewClass,
) -> Result<ValidatedClass, DataError> {
    let name = required_text(&class.name, "class name", 1, 80)?;
    let weekday = validate_range(class.weekday, 0, 6, "weekday")?;
    let start_time = validate_time(&class.start_time, "start time")?;
    let duration_minutes = validate_range(class.duration_minutes, 15, 480, "duration minutes")?;
    let capacity = validate_range(class.capacity, 1, 500, "capacity")?;
    let room = normalize_optional_text(class.room, 60, "room")?;
    let notes = normalize_optional_text(class.notes, 300, "notes")?;
    let coach_user_id = match class.coach_user_id {
        Some(value) if !value.trim().is_empty() => {
            let value = value.trim().to_string();
            let exists: Option<String> = conn
                .query_row(
                    "SELECT id FROM users WHERE organization_id = ?1 AND id = ?2",
                    params![organization_id, value],
                    |row| row.get(0),
                )
                .optional()?;
            if exists.is_none() {
                return Err(DataError::StaffNotFound);
            }
            Some(value)
        }
        _ => None,
    };
    Ok(ValidatedClass {
        name,
        coach_user_id,
        weekday,
        start_time,
        duration_minutes,
        capacity,
        room,
        notes,
    })
}

struct ValidatedCampaign {
    name: String,
    channel: String,
    audience: String,
    status: String,
    starts_on: Option<String>,
    ends_on: Option<String>,
    notes: Option<String>,
}

fn validate_campaign(campaign: NewCampaign) -> Result<ValidatedCampaign, DataError> {
    let name = required_text(&campaign.name, "campaign name", 1, 80)?;
    let channel = validate_choice(
        &campaign.channel,
        &["whatsapp", "sms", "email", "in_app", "referral"],
        "campaign channel",
    )?;
    let audience = required_text(&campaign.audience, "audience", 1, 40)?;
    let status = validate_choice(
        &campaign.status,
        &["draft", "active", "paused", "completed"],
        "campaign status",
    )?;
    let starts_on = validate_date(campaign.starts_on, "start date")?;
    let ends_on = validate_date(campaign.ends_on, "end date")?;
    if let (Some(start), Some(end)) = (&starts_on, &ends_on) {
        if end < start {
            return Err(DataError::InvalidValue(
                "the campaign end date cannot be before the start date".into(),
            ));
        }
    }
    let notes = normalize_optional_text(campaign.notes, 300, "notes")?;
    Ok(ValidatedCampaign {
        name,
        channel,
        audience,
        status,
        starts_on,
        ends_on,
        notes,
    })
}

struct ValidatedEquipment {
    name: String,
    code: Option<String>,
    area: Option<String>,
    acquired_on: Option<String>,
    notes: Option<String>,
}

fn validate_equipment(equipment: NewEquipment) -> Result<ValidatedEquipment, DataError> {
    let name = required_text(&equipment.name, "equipment name", 1, 80)?;
    let code = normalize_optional_text(equipment.code, 40, "code")?;
    let area = normalize_optional_text(equipment.area, 60, "area")?;
    let acquired_on = validate_date(equipment.acquired_on, "acquired date")?;
    let notes = normalize_optional_text(equipment.notes, 300, "notes")?;
    Ok(ValidatedEquipment {
        name,
        code,
        area,
        acquired_on,
        notes,
    })
}

fn validate_choice(value: &str, allowed: &[&str], field: &str) -> Result<String, DataError> {
    let value = value.trim();
    if allowed.contains(&value) {
        Ok(value.to_string())
    } else {
        Err(DataError::InvalidValue(format!("invalid {field}")))
    }
}

fn validate_range(value: i64, min: i64, max: i64, field: &str) -> Result<i64, DataError> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(DataError::InvalidValue(format!(
            "the {field} must be between {min} and {max}"
        )))
    }
}

fn validate_currency(value: &str) -> Result<String, DataError> {
    let value = value.trim().to_uppercase();
    if value.chars().count() == 3
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        Ok(value)
    } else {
        Err(DataError::InvalidValue(
            "the currency must be a three-letter code".into(),
        ))
    }
}

fn validate_time(value: &str, field: &str) -> Result<String, DataError> {
    let value = value.trim();
    let (hours, minutes) = value
        .split_once(':')
        .ok_or_else(|| DataError::InvalidValue(format!("the {field} must use HH:MM")))?;
    let hours: i64 = hours
        .parse()
        .map_err(|_| DataError::InvalidValue(format!("the {field} must use HH:MM")))?;
    let minutes: i64 = minutes
        .parse()
        .map_err(|_| DataError::InvalidValue(format!("the {field} must use HH:MM")))?;
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return Err(DataError::InvalidValue(format!(
            "the {field} must use HH:MM"
        )));
    }
    Ok(format!("{hours:02}:{minutes:02}"))
}

fn validate_date(value: Option<String>, field: &str) -> Result<Option<String>, DataError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|date| Some(date.format("%Y-%m-%d").to_string()))
        .map_err(|_| DataError::InvalidValue(format!("the {field} must use YYYY-MM-DD")))
}

fn load_gym_settings(
    conn: &rusqlite::Connection,
    organization_id: &str,
) -> Result<GymSettingsRecord, DataError> {
    conn.query_row(
        "SELECT id, business_name, legal_name, tax_id, currency, timezone, locale,
                opening_time, closing_time, slot_minutes, grace_minutes, inactivity_days,
                frozen_days, updated_at
         FROM gym_settings
         WHERE organization_id = ?1",
        params![organization_id],
        |row| {
            Ok(GymSettingsRecord {
                id: row.get(0)?,
                business_name: row.get(1)?,
                legal_name: row.get(2)?,
                tax_id: row.get(3)?,
                currency: row.get(4)?,
                timezone: row.get(5)?,
                locale: row.get(6)?,
                opening_time: row.get(7)?,
                closing_time: row.get(8)?,
                slot_minutes: row.get(9)?,
                grace_minutes: row.get(10)?,
                inactivity_days: row.get(11)?,
                frozen_days: row.get(12)?,
                updated_at: row.get(13)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| DataError::InvalidValue("the gym settings are not available".into()))
}

fn map_integration_link(row: &Row<'_>) -> rusqlite::Result<IntegrationLinkRecord> {
    let capabilities: String = row.get(5)?;
    Ok(IntegrationLinkRecord {
        id: row.get(0)?,
        module_id: row.get(1)?,
        display_name: row.get(2)?,
        contract_version: row.get(3)?,
        status: row.get(4)?,
        capabilities: serde_json::from_str(&capabilities).unwrap_or_default(),
        last_seen_at: row.get(6)?,
        last_event_at: row.get(7)?,
        last_snapshot_at: row.get(8)?,
        notes: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn map_class(row: &Row<'_>) -> rusqlite::Result<ClassRecord> {
    Ok(ClassRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        coach_user_id: row.get(2)?,
        coach_name: row.get(3)?,
        weekday: row.get(4)?,
        start_time: row.get(5)?,
        duration_minutes: row.get(6)?,
        capacity: row.get(7)?,
        room: row.get(8)?,
        status: row.get(9)?,
        notes: row.get(10)?,
        booked: row.get(11)?,
        attended: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn load_class_booking(
    conn: &rusqlite::Connection,
    organization_id: &str,
    booking_id: &str,
) -> Result<ClassBookingRecord, DataError> {
    conn.query_row(
        "SELECT b.id, b.class_id, c.name, b.member_id,
                m.first_name || ' ' || m.last_name, b.status, b.note,
                b.booked_at, b.updated_at
         FROM class_bookings b
         JOIN classes c ON c.id = b.class_id
         JOIN members m ON m.id = b.member_id
         WHERE b.organization_id = ?1 AND b.id = ?2",
        params![organization_id, booking_id],
        map_class_booking,
    )
    .optional()?
    .ok_or(DataError::ClassBookingNotFound)
}

fn map_class_booking(row: &Row<'_>) -> rusqlite::Result<ClassBookingRecord> {
    Ok(ClassBookingRecord {
        id: row.get(0)?,
        class_id: row.get(1)?,
        class_name: row.get(2)?,
        member_id: row.get(3)?,
        member_name: row.get(4)?,
        status: row.get(5)?,
        note: row.get(6)?,
        booked_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn load_campaign(
    conn: &rusqlite::Connection,
    organization_id: &str,
    campaign_id: &str,
) -> Result<CampaignRecord, DataError> {
    conn.query_row(
        "SELECT id, name, channel, audience, status, starts_on, ends_on, notes,
                created_at, updated_at
         FROM campaigns
         WHERE organization_id = ?1 AND id = ?2",
        params![organization_id, campaign_id],
        map_campaign,
    )
    .optional()?
    .ok_or(DataError::CampaignNotFound)
}

fn map_campaign(row: &Row<'_>) -> rusqlite::Result<CampaignRecord> {
    Ok(CampaignRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        channel: row.get(2)?,
        audience: row.get(3)?,
        status: row.get(4)?,
        starts_on: row.get(5)?,
        ends_on: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn load_equipment(
    conn: &rusqlite::Connection,
    organization_id: &str,
    equipment_id: &str,
) -> Result<EquipmentRecord, DataError> {
    conn.query_row(
        "SELECT e.id, e.name, e.code, e.area, e.status, e.acquired_on, e.notes,
                (SELECT count(*) FROM maintenance_orders o
                  WHERE o.equipment_id = e.id AND o.status IN ('open', 'in_progress')),
                e.created_at, e.updated_at
         FROM equipment e
         WHERE e.organization_id = ?1 AND e.id = ?2",
        params![organization_id, equipment_id],
        map_equipment,
    )
    .optional()?
    .ok_or(DataError::EquipmentNotFound)
}

fn map_equipment(row: &Row<'_>) -> rusqlite::Result<EquipmentRecord> {
    Ok(EquipmentRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        code: row.get(2)?,
        area: row.get(3)?,
        status: row.get(4)?,
        acquired_on: row.get(5)?,
        notes: row.get(6)?,
        open_orders: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn load_maintenance_order(
    conn: &rusqlite::Connection,
    organization_id: &str,
    order_id: &str,
) -> Result<MaintenanceOrderRecord, DataError> {
    conn.query_row(
        "SELECT o.id, o.equipment_id, e.name, o.kind, o.status, o.priority, o.summary,
                o.detail, o.due_on, o.reported_by, o.completed_at, o.created_at, o.updated_at
         FROM maintenance_orders o
         LEFT JOIN equipment e ON e.id = o.equipment_id
         WHERE o.organization_id = ?1 AND o.id = ?2",
        params![organization_id, order_id],
        map_maintenance_order,
    )
    .optional()?
    .ok_or(DataError::MaintenanceOrderNotFound)
}

fn map_maintenance_order(row: &Row<'_>) -> rusqlite::Result<MaintenanceOrderRecord> {
    Ok(MaintenanceOrderRecord {
        id: row.get(0)?,
        equipment_id: row.get(1)?,
        equipment_name: row.get(2)?,
        kind: row.get(3)?,
        status: row.get(4)?,
        priority: row.get(5)?,
        summary: row.get(6)?,
        detail: row.get(7)?,
        due_on: row.get(8)?,
        reported_by: row.get(9)?,
        completed_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}
