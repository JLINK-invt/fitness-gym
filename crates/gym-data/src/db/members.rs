use std::collections::HashMap;

use chrono::Datelike;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};

use super::{
    enqueue_sync_event, normalize_optional_text, organization_currency, required_text, Db,
    MAX_AMOUNT_MINOR,
};
use crate::error::DataError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipPlanRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub currency: String,
    pub grace_days: i64,
    pub version: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMembershipPlan {
    pub name: String,
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub grace_days: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMembershipPlan {
    pub name: String,
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub grace_days: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRecord {
    pub id: String,
    pub member_number: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<String>,
    pub status: String,
    pub joined_at: String,
    pub notes: Option<String>,
    pub active_membership: Option<MembershipRecord>,
    pub debt_minor: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMember {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMember {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<String>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProspectRecord {
    pub id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub converted_member_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProspect {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProspect {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRecord {
    pub id: String,
    pub member_id: String,
    pub member_name: String,
    pub plan_id: String,
    pub plan_name: String,
    pub plan_version: i64,
    pub plan_version_id: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub price_minor: i64,
    pub currency: String,
    pub grace_days: i64,
    pub access_days: String,
    pub access_start_minute: i64,
    pub access_end_minute: i64,
    pub min_age: Option<i64>,
    pub max_age: Option<i64>,
    pub paused_at: Option<String>,
    pub pause_until: Option<String>,
    pub cancellation_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipChargeRecord {
    pub id: String,
    pub membership_id: String,
    pub member_id: String,
    pub description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub due_date: String,
    pub status: String,
    pub paid_minor: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipPaymentRecord {
    pub id: String,
    pub charge_id: String,
    pub membership_id: String,
    pub member_id: String,
    pub amount_minor: i64,
    pub payment_method: String,
    pub reference: Option<String>,
    pub receipt_number: String,
    pub cash_session_id: Option<String>,
    pub received_by: Option<String>,
    pub received_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipPaymentReceiptRecord {
    pub payment_id: String,
    pub receipt_number: String,
    pub gym_name: String,
    pub member_number: String,
    pub member_name: String,
    pub charge_description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub payment_method: String,
    pub reference: Option<String>,
    pub received_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipDashboardTrendRecord {
    pub month: String,
    pub new_members: i64,
    pub renewals: i64,
    pub cancellations: i64,
    pub income_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberAccountRecord {
    pub member_id: String,
    pub currency: String,
    pub charges: Vec<MembershipChargeRecord>,
    pub payments: Vec<MembershipPaymentRecord>,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipDashboardRecord {
    pub period_start: String,
    pub period_end: String,
    pub timezone: String,
    pub active_members: i64,
    pub active_memberships: i64,
    pub expiring_soon: i64,
    pub past_due: i64,
    pub debt_minor: i64,
    pub new_members_this_month: i64,
    pub income_this_month_minor: i64,
    pub renewals_this_month: i64,
    pub cancellations_this_month: i64,
    pub trend: Vec<MembershipDashboardTrendRecord>,
    pub recent_payments: Vec<MembershipPaymentReceiptRecord>,
    pub expiring_memberships: Vec<MembershipRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMembershipPayment {
    pub charge_id: String,
    pub amount_minor: i64,
    pub payment_method: String,
    pub reference: Option<String>,
    pub idempotency_key: String,
}

struct ValidatedMembershipPlan {
    name: String,
    description: Option<String>,
    duration_days: i64,
    price_minor: i64,
    grace_days: i64,
    access_days: String,
    access_start_minute: i64,
    access_end_minute: i64,
    min_age: Option<i64>,
    max_age: Option<i64>,
    active: bool,
}

struct ValidatedMember {
    first_name: String,
    last_name: String,
    phone: Option<String>,
    email: Option<String>,
    birth_date: Option<String>,
    notes: Option<String>,
}

struct MemberBase {
    id: String,
    member_number: String,
    first_name: String,
    last_name: String,
    phone: Option<String>,
    email: Option<String>,
    birth_date: Option<String>,
    status: String,
    joined_at: String,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

impl Db {
    pub fn list_membership_plans(
        &self,
        organization_id: &str,
    ) -> Result<Vec<MembershipPlanRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, name, description, duration_days, price_minor, currency,
                        grace_days, current_version, access_days, access_start_minute,
                        access_end_minute, min_age, max_age, active, created_at, updated_at
                 FROM membership_plans
                 WHERE organization_id = ?1
                 ORDER BY active DESC, price_minor, name COLLATE NOCASE",
            )?;
            let plans = statement
                .query_map(params![organization_id], map_membership_plan)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(plans)
        })
    }

    pub fn membership_plan(
        &self,
        organization_id: &str,
        plan_id: &str,
    ) -> Result<MembershipPlanRecord, DataError> {
        self.with_connection(|conn| load_membership_plan(conn, organization_id, plan_id))
    }

    pub fn create_membership_plan(
        &self,
        organization_id: &str,
        plan: NewMembershipPlan,
    ) -> Result<MembershipPlanRecord, DataError> {
        let plan = validate_membership_plan(plan)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let currency = organization_currency(&transaction, organization_id)?;
            ensure_unique_membership_plan(&transaction, organization_id, &plan.name, None)?;
            transaction.execute(
                "INSERT INTO membership_plans
                     (id, organization_id, name, description, duration_days,
                      price_minor, currency, grace_days, current_version, access_days,
                      access_start_minute, access_end_minute, min_age, max_age,
                      active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
                params![
                    id,
                    organization_id,
                    plan.name,
                    plan.description,
                    plan.duration_days,
                    plan.price_minor,
                    currency,
                    plan.grace_days,
                    plan.access_days,
                    plan.access_start_minute,
                    plan.access_end_minute,
                    plan.min_age,
                    plan.max_age,
                    i64::from(plan.active),
                    now
                ],
            )?;
            insert_membership_plan_version(
                &transaction,
                organization_id,
                &id,
                1,
                &plan,
                &currency,
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership_plan(organization_id, &id)
    }

    pub fn update_membership_plan(
        &self,
        organization_id: &str,
        plan_id: &str,
        plan: UpdateMembershipPlan,
    ) -> Result<MembershipPlanRecord, DataError> {
        let plan = validate_membership_plan_update(plan)?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            ensure_unique_membership_plan(
                &transaction,
                organization_id,
                &plan.name,
                Some(plan_id),
            )?;
            let existing = load_membership_plan(&transaction, organization_id, plan_id)?;
            let version = existing.version + 1;
            let currency = organization_currency(&transaction, organization_id)?;
            let changed = transaction.execute(
                "UPDATE membership_plans
                 SET name = ?1, description = ?2, duration_days = ?3,
                     price_minor = ?4, grace_days = ?5, current_version = ?6,
                     access_days = ?7, access_start_minute = ?8,
                     access_end_minute = ?9, min_age = ?10, max_age = ?11,
                     active = ?12, updated_at = ?13
                 WHERE organization_id = ?14 AND id = ?15",
                params![
                    plan.name,
                    plan.description,
                    plan.duration_days,
                    plan.price_minor,
                    plan.grace_days,
                    version,
                    plan.access_days,
                    plan.access_start_minute,
                    plan.access_end_minute,
                    plan.min_age,
                    plan.max_age,
                    i64::from(plan.active),
                    now,
                    organization_id,
                    plan_id
                ],
            )?;
            if changed == 0 {
                return Err(DataError::MembershipPlanNotFound);
            }
            insert_membership_plan_version(
                &transaction,
                organization_id,
                plan_id,
                version,
                &plan,
                &currency,
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership_plan(organization_id, plan_id)
    }

    pub fn list_members(
        &self,
        organization_id: &str,
        query: Option<&str>,
    ) -> Result<Vec<MemberRecord>, DataError> {
        self.refresh_memberships(organization_id)?;
        let query = query.map(str::trim).filter(|value| !value.is_empty());
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, member_number, first_name, last_name, phone, email,
                        birth_date, status, joined_at, notes, created_at, updated_at
                 FROM members
                 WHERE organization_id = ?1
                   AND (
                       ?2 IS NULL
                       OR first_name LIKE '%' || ?2 || '%' COLLATE NOCASE
                       OR last_name LIKE '%' || ?2 || '%' COLLATE NOCASE
                       OR member_number LIKE '%' || ?2 || '%' COLLATE NOCASE
                       OR phone LIKE '%' || ?2 || '%' COLLATE NOCASE
                       OR email LIKE '%' || ?2 || '%' COLLATE NOCASE
                   )
                 ORDER BY (status = 'active') DESC,
                          last_name COLLATE NOCASE, first_name COLLATE NOCASE",
            )?;
            let rows = statement
                .query_map(params![organization_id, query], map_member_base)?
                .collect::<Result<Vec<_>, _>>()?;
            rows.into_iter()
                .map(|row| hydrate_member(conn, organization_id, row))
                .collect()
        })
    }

    pub fn member(
        &self,
        organization_id: &str,
        member_id: &str,
    ) -> Result<MemberRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        self.with_connection(|conn| load_member(conn, organization_id, member_id))
    }

    pub fn create_member(
        &self,
        organization_id: &str,
        member: NewMember,
    ) -> Result<MemberRecord, DataError> {
        let member = validate_new_member(member)?;
        let now = chrono::Utc::now().to_rfc3339();
        let joined_at = date_string(today());
        let member_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let member_id =
                insert_member(&transaction, organization_id, &member, &joined_at, &now)?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "member.created",
                "member",
                &member_id,
                &format!("member:{member_id}:created"),
                &serde_json::json!({
                    "memberId": member_id,
                    "firstName": member.first_name,
                    "lastName": member.last_name,
                    "joinedAt": joined_at,
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
            member_id
        };
        self.member(organization_id, &member_id)
    }

    pub fn update_member(
        &self,
        organization_id: &str,
        member_id: &str,
        member: UpdateMember,
    ) -> Result<MemberRecord, DataError> {
        if !matches!(member.status.as_str(), "active" | "inactive" | "archived") {
            return Err(DataError::InvalidValue("invalid member status".into()));
        }
        let first_name = required_text(&member.first_name, "first name", 1, 80)?;
        let last_name = required_text_allow_empty(&member.last_name, "last name", 80)?;
        let phone = normalize_optional_text(member.phone, 40, "phone")?;
        let email = validate_email(member.email)?;
        let birth_date = validate_birth_date(member.birth_date)?;
        let notes = normalize_optional_text(member.notes, 500, "notes")?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let changed = transaction.execute(
                "UPDATE members
                 SET first_name = ?1, last_name = ?2, phone = ?3, email = ?4,
                     birth_date = ?5, status = ?6, notes = ?7, updated_at = ?8
                 WHERE organization_id = ?9 AND id = ?10",
                params![
                    first_name,
                    last_name,
                    phone,
                    email,
                    birth_date,
                    member.status,
                    notes,
                    now,
                    organization_id,
                    member_id
                ],
            )?;
            if changed == 0 {
                return Err(DataError::MemberNotFound);
            }
            enqueue_sync_event(
                &transaction,
                organization_id,
                "member.updated",
                "member",
                member_id,
                &format!("member:{member_id}:updated:{now}"),
                &serde_json::json!({
                    "memberId": member_id,
                    "status": member.status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.member(organization_id, member_id)
    }

    pub fn list_prospects(
        &self,
        organization_id: &str,
        query: Option<&str>,
    ) -> Result<Vec<ProspectRecord>, DataError> {
        let query = query.map(str::trim).filter(|value| !value.is_empty());
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, name, phone, email, source, status, notes,
                        converted_member_id, created_at, updated_at
                 FROM prospects
                 WHERE organization_id = ?1
                   AND (
                       ?2 IS NULL
                       OR name LIKE '%' || ?2 || '%' COLLATE NOCASE
                       OR phone LIKE '%' || ?2 || '%' COLLATE NOCASE
                       OR email LIKE '%' || ?2 || '%' COLLATE NOCASE
                   )
                 ORDER BY CASE status
                     WHEN 'new' THEN 0
                     WHEN 'contacted' THEN 1
                     WHEN 'trial' THEN 2
                     WHEN 'converted' THEN 3
                     ELSE 4
                 END, created_at DESC",
            )?;
            let prospects = statement
                .query_map(params![organization_id, query], map_prospect)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(prospects)
        })
    }

    pub fn create_prospect(
        &self,
        organization_id: &str,
        prospect: NewProspect,
    ) -> Result<ProspectRecord, DataError> {
        let name = required_text(&prospect.name, "prospect name", 1, 80)?;
        let phone = normalize_optional_text(prospect.phone, 40, "phone")?;
        let email = validate_email(prospect.email)?;
        let source = normalize_optional_text(prospect.source, 60, "source")?;
        let notes = normalize_optional_text(prospect.notes, 500, "notes")?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            transaction.execute(
                "INSERT INTO prospects
                     (id, organization_id, name, phone, email, source, status,
                      notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'new', ?7, ?8, ?8)",
                params![id, organization_id, name, phone, email, source, notes, now],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "prospect.created",
                "prospect",
                &id,
                &format!("prospect:{id}:created"),
                &serde_json::json!({
                    "prospectId": id,
                    "name": name,
                    "status": "new",
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_prospect(conn, organization_id, &id))
    }

    pub fn update_prospect(
        &self,
        organization_id: &str,
        prospect_id: &str,
        prospect: UpdateProspect,
    ) -> Result<ProspectRecord, DataError> {
        if !matches!(
            prospect.status.as_str(),
            "new" | "contacted" | "trial" | "converted" | "lost"
        ) {
            return Err(DataError::InvalidValue("invalid prospect status".into()));
        }
        let name = required_text(&prospect.name, "prospect name", 1, 80)?;
        let phone = normalize_optional_text(prospect.phone, 40, "phone")?;
        let email = validate_email(prospect.email)?;
        let source = normalize_optional_text(prospect.source, 60, "source")?;
        let notes = normalize_optional_text(prospect.notes, 500, "notes")?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let existing = load_prospect(&transaction, organization_id, prospect_id)?;
            if existing.status == "converted" && prospect.status != "converted" {
                return Err(DataError::InvalidValue(
                    "a converted prospect cannot be reopened".into(),
                ));
            }
            if prospect.status == "converted" && existing.converted_member_id.is_none() {
                return Err(DataError::InvalidValue(
                    "convert the prospect from the conversion action".into(),
                ));
            }
            transaction.execute(
                "UPDATE prospects
                 SET name = ?1, phone = ?2, email = ?3, source = ?4,
                     status = ?5, notes = ?6, updated_at = ?7
                 WHERE organization_id = ?8 AND id = ?9",
                params![
                    name,
                    phone,
                    email,
                    source,
                    prospect.status,
                    notes,
                    now,
                    organization_id,
                    prospect_id
                ],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "prospect.updated",
                "prospect",
                prospect_id,
                &format!("prospect:{prospect_id}:updated:{now}"),
                &serde_json::json!({
                    "prospectId": prospect_id,
                    "name": name,
                    "status": prospect.status,
                    "updatedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.with_connection(|conn| load_prospect(conn, organization_id, prospect_id))
    }

    pub fn convert_prospect(
        &self,
        organization_id: &str,
        prospect_id: &str,
    ) -> Result<MemberRecord, DataError> {
        let now = chrono::Utc::now().to_rfc3339();
        let joined_at = date_string(today());
        let member_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let prospect = load_prospect(&transaction, organization_id, prospect_id)?;
            if prospect.status == "converted" || prospect.converted_member_id.is_some() {
                return Err(DataError::ProspectAlreadyConverted);
            }
            let (first_name, last_name) = split_prospect_name(&prospect.name);
            let member = ValidatedMember {
                first_name,
                last_name,
                phone: prospect.phone,
                email: prospect.email,
                birth_date: None,
                notes: prospect.notes,
            };
            let member_id =
                insert_member(&transaction, organization_id, &member, &joined_at, &now)?;
            transaction.execute(
                "UPDATE prospects
                 SET status = 'converted', converted_member_id = ?1, updated_at = ?2
                 WHERE organization_id = ?3 AND id = ?4",
                params![member_id, now, organization_id, prospect_id],
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "member.created",
                "member",
                &member_id,
                &format!("member:{member_id}:created"),
                &serde_json::json!({
                    "memberId": member_id,
                    "firstName": member.first_name,
                    "lastName": member.last_name,
                    "joinedAt": joined_at,
                    "createdAt": now,
                }),
                &now,
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "prospect.converted",
                "member",
                &member_id,
                &format!("prospect:{prospect_id}:converted"),
                &serde_json::json!({
                    "prospectId": prospect_id,
                    "memberId": member_id,
                    "convertedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
            member_id
        };
        self.member(organization_id, &member_id)
    }

    pub fn list_memberships(
        &self,
        organization_id: &str,
        member_id: &str,
    ) -> Result<Vec<MembershipRecord>, DataError> {
        self.refresh_memberships(organization_id)?;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT m.id, m.member_id, TRIM(mem.first_name || ' ' || mem.last_name),
                        m.plan_id, m.plan_name, m.plan_version, m.plan_version_id,
                        m.status, m.start_date, m.end_date,
                        m.price_minor, m.currency, m.grace_days, m.access_days,
                        m.access_start_minute, m.access_end_minute, m.min_age, m.max_age,
                        m.paused_at, m.pause_until, m.cancellation_reason,
                        m.created_at, m.updated_at
                 FROM memberships m
                 JOIN members mem
                   ON mem.organization_id = m.organization_id AND mem.id = m.member_id
                 WHERE m.organization_id = ?1 AND m.member_id = ?2
                 ORDER BY m.created_at DESC",
            )?;
            let memberships = statement
                .query_map(params![organization_id, member_id], map_membership)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DataError::from)?;
            Ok(memberships)
        })
    }

    pub fn membership(
        &self,
        organization_id: &str,
        membership_id: &str,
    ) -> Result<MembershipRecord, DataError> {
        self.with_connection(|conn| load_membership(conn, organization_id, membership_id))
    }

    pub fn create_membership(
        &self,
        organization_id: &str,
        actor_id: &str,
        member_id: &str,
        plan_id: &str,
        start_date: Option<&str>,
    ) -> Result<MembershipRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        let start = match start_date {
            Some(value) => parse_date(value, "membership start date")?,
            None => today(),
        };
        let now = chrono::Utc::now().to_rfc3339();
        let membership_id = uuid::Uuid::new_v4().to_string();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let member = load_member_base(&transaction, organization_id, member_id)?;
            if member.status != "active" {
                return Err(DataError::InvalidValue(
                    "only active members can receive a membership".into(),
                ));
            }
            let plan = load_membership_plan(&transaction, organization_id, plan_id)?;
            if !plan.active {
                return Err(DataError::InvalidValue(
                    "the membership plan is not active".into(),
                ));
            }
            ensure_plan_eligibility(&plan, &member, start)?;
            let plan_version_id =
                load_plan_version_id(&transaction, organization_id, &plan.id, plan.version)?;
            let current: bool = transaction.query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM memberships
                     WHERE organization_id = ?1 AND member_id = ?2
                       AND status IN ('scheduled', 'active', 'paused')
                 )",
                params![organization_id, member_id],
                |row| row.get(0),
            )?;
            if current {
                return Err(DataError::MembershipAlreadyCurrent);
            }
            let end = start + chrono::TimeDelta::days(plan.duration_days - 1);
            let status = if start > today() {
                "scheduled"
            } else {
                "active"
            };
            transaction.execute(
                "INSERT INTO memberships
                     (id, organization_id, member_id, plan_id, plan_name,
                      plan_version, plan_version_id, status, start_date, end_date,
                      price_minor, currency, grace_days, access_days,
                      access_start_minute, access_end_minute, min_age, max_age,
                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         ?14, ?15, ?16, ?17, ?18, ?19, ?19)",
                params![
                    membership_id,
                    organization_id,
                    member_id,
                    plan.id,
                    plan.name,
                    plan.version,
                    plan_version_id,
                    status,
                    date_string(start),
                    date_string(end),
                    plan.price_minor,
                    plan.currency,
                    plan.grace_days,
                    plan.access_days,
                    plan.access_start_minute,
                    plan.access_end_minute,
                    plan.min_age,
                    plan.max_age,
                    now
                ],
            )?;
            insert_membership_event(
                &transaction,
                organization_id,
                &membership_id,
                "created",
                None,
                status,
                None,
                actor_id,
                &now,
            )?;
            if plan.price_minor > 0 {
                insert_membership_charge(
                    &transaction,
                    organization_id,
                    &membership_id,
                    member_id,
                    &format!("Membresía {}", plan.name),
                    plan.price_minor,
                    &plan.currency,
                    &date_string(start),
                    &now,
                )?;
            }
            enqueue_sync_event(
                &transaction,
                organization_id,
                "membership.created",
                "membership",
                &membership_id,
                &format!("membership:{membership_id}:created"),
                &serde_json::json!({
                    "membershipId": membership_id,
                    "memberId": member_id,
                    "planId": plan.id,
                    "planVersion": plan.version,
                    "planVersionId": plan_version_id,
                    "status": status,
                    "startDate": date_string(start),
                    "endDate": date_string(end),
                    "createdAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership(organization_id, &membership_id)
    }

    pub fn renew_membership(
        &self,
        organization_id: &str,
        actor_id: &str,
        membership_id: &str,
        plan_id: Option<&str>,
    ) -> Result<MembershipRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let membership = load_membership(&transaction, organization_id, membership_id)?;
            if matches!(membership.status.as_str(), "paused" | "cancelled") {
                return Err(DataError::MembershipStateConflict);
            }
            let plan = match plan_id {
                Some(plan_id) => load_membership_plan(&transaction, organization_id, plan_id)?,
                None => load_membership_plan(&transaction, organization_id, &membership.plan_id)?,
            };
            if !plan.active {
                return Err(DataError::InvalidValue(
                    "the membership plan is not active".into(),
                ));
            }
            let current_end = parse_date(&membership.end_date, "membership end date")?;
            let period_start = std::cmp::max(today(), current_end + chrono::TimeDelta::days(1));
            let member = load_member_base(&transaction, organization_id, &membership.member_id)?;
            ensure_plan_eligibility(&plan, &member, period_start)?;
            let plan_version_id =
                load_plan_version_id(&transaction, organization_id, &plan.id, plan.version)?;
            let end = period_start + chrono::TimeDelta::days(plan.duration_days - 1);
            let membership_start = parse_date(&membership.start_date, "membership start date")?;
            let status = if membership_start > today() {
                "scheduled"
            } else {
                "active"
            };
            transaction.execute(
                "UPDATE memberships
                 SET plan_id = ?1, plan_name = ?2, plan_version = ?3,
                     plan_version_id = ?4, status = ?5, end_date = ?6,
                     price_minor = ?7, currency = ?8, grace_days = ?9,
                     access_days = ?10, access_start_minute = ?11,
                     access_end_minute = ?12, min_age = ?13, max_age = ?14,
                     paused_at = NULL, pause_until = NULL,
                     cancellation_reason = NULL, updated_at = ?15
                 WHERE organization_id = ?16 AND id = ?17",
                params![
                    plan.id,
                    plan.name,
                    plan.version,
                    plan_version_id,
                    status,
                    date_string(end),
                    plan.price_minor,
                    plan.currency,
                    plan.grace_days,
                    plan.access_days,
                    plan.access_start_minute,
                    plan.access_end_minute,
                    plan.min_age,
                    plan.max_age,
                    now,
                    organization_id,
                    membership_id
                ],
            )?;
            insert_membership_event(
                &transaction,
                organization_id,
                membership_id,
                "renewed",
                Some(&membership.status),
                status,
                Some(&format!("Nuevo periodo {}", date_string(period_start))),
                actor_id,
                &now,
            )?;
            if plan.price_minor > 0 {
                insert_membership_charge(
                    &transaction,
                    organization_id,
                    membership_id,
                    &membership.member_id,
                    &format!("Renovación {}", plan.name),
                    plan.price_minor,
                    &plan.currency,
                    &date_string(period_start),
                    &now,
                )?;
            }
            enqueue_sync_event(
                &transaction,
                organization_id,
                "membership.renewed",
                "membership",
                membership_id,
                &format!(
                    "membership:{membership_id}:renewed:{}",
                    date_string(period_start)
                ),
                &serde_json::json!({
                    "membershipId": membership_id,
                    "memberId": membership.member_id,
                    "planId": plan.id,
                    "planVersion": plan.version,
                    "planVersionId": plan_version_id,
                    "status": status,
                    "startDate": date_string(period_start),
                    "endDate": date_string(end),
                    "renewedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership(organization_id, membership_id)
    }

    pub fn pause_membership(
        &self,
        organization_id: &str,
        actor_id: &str,
        membership_id: &str,
        until_date: &str,
    ) -> Result<MembershipRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        let until = parse_date(until_date, "pause end date")?;
        if until < today() {
            return Err(DataError::InvalidValue(
                "the pause end date cannot be in the past".into(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let membership = load_membership(&transaction, organization_id, membership_id)?;
            if !matches!(membership.status.as_str(), "active" | "scheduled") {
                return Err(DataError::MembershipStateConflict);
            }
            transaction.execute(
                "UPDATE memberships
                 SET status = 'paused', paused_at = ?1, pause_until = ?2, updated_at = ?3
                 WHERE organization_id = ?4 AND id = ?5",
                params![
                    date_string(today()),
                    date_string(until),
                    now,
                    organization_id,
                    membership_id
                ],
            )?;
            insert_membership_event(
                &transaction,
                organization_id,
                membership_id,
                "paused",
                Some(&membership.status),
                "paused",
                Some(&format!("Hasta {}", date_string(until))),
                actor_id,
                &now,
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "membership.paused",
                "membership",
                membership_id,
                &format!("membership:{membership_id}:paused:{}", date_string(until)),
                &serde_json::json!({
                    "membershipId": membership_id,
                    "memberId": membership.member_id,
                    "pauseUntil": date_string(until),
                    "pausedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership(organization_id, membership_id)
    }

    pub fn resume_membership(
        &self,
        organization_id: &str,
        actor_id: &str,
        membership_id: &str,
    ) -> Result<MembershipRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let membership = load_membership(&transaction, organization_id, membership_id)?;
            if membership.status != "paused" {
                return Err(DataError::MembershipStateConflict);
            }
            let paused_at = membership
                .paused_at
                .as_deref()
                .ok_or(DataError::MembershipStateConflict)?;
            let paused_at = parse_date(paused_at, "pause start date")?;
            let paused_days = (today() - paused_at).num_days().max(0);
            let end = parse_date(&membership.end_date, "membership end date")?
                + chrono::TimeDelta::days(paused_days);
            let start = parse_date(&membership.start_date, "membership start date")?;
            let status = if start > today() {
                "scheduled"
            } else {
                "active"
            };
            transaction.execute(
                "UPDATE memberships
                 SET status = ?1, end_date = ?2, paused_at = NULL,
                     pause_until = NULL, updated_at = ?3
                 WHERE organization_id = ?4 AND id = ?5",
                params![
                    status,
                    date_string(end),
                    now,
                    organization_id,
                    membership_id
                ],
            )?;
            insert_membership_event(
                &transaction,
                organization_id,
                membership_id,
                "resumed",
                Some("paused"),
                status,
                Some(&format!("Extendida {paused_days} días")),
                actor_id,
                &now,
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "membership.resumed",
                "membership",
                membership_id,
                &format!("membership:{membership_id}:resumed:{now}"),
                &serde_json::json!({
                    "membershipId": membership_id,
                    "memberId": membership.member_id,
                    "endDate": date_string(end),
                    "resumedAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership(organization_id, membership_id)
    }

    pub fn cancel_membership(
        &self,
        organization_id: &str,
        actor_id: &str,
        membership_id: &str,
        reason: &str,
    ) -> Result<MembershipRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        let reason = required_text(reason, "cancellation reason", 1, 300)?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let membership = load_membership(&transaction, organization_id, membership_id)?;
            if !matches!(
                membership.status.as_str(),
                "active" | "scheduled" | "paused"
            ) {
                return Err(DataError::MembershipStateConflict);
            }
            transaction.execute(
                "UPDATE memberships
                 SET status = 'cancelled', paused_at = NULL, pause_until = NULL,
                     cancellation_reason = ?1, updated_at = ?2
                 WHERE organization_id = ?3 AND id = ?4",
                params![reason, now, organization_id, membership_id],
            )?;
            insert_membership_event(
                &transaction,
                organization_id,
                membership_id,
                "cancelled",
                Some(&membership.status),
                "cancelled",
                Some(&reason),
                actor_id,
                &now,
            )?;
            enqueue_sync_event(
                &transaction,
                organization_id,
                "membership.cancelled",
                "membership",
                membership_id,
                &format!("membership:{membership_id}:cancelled:{now}"),
                &serde_json::json!({
                    "membershipId": membership_id,
                    "memberId": membership.member_id,
                    "reason": reason,
                    "cancelledAt": now,
                }),
                &now,
            )?;
            transaction.commit()?;
        }
        self.membership(organization_id, membership_id)
    }

    pub fn member_account(
        &self,
        organization_id: &str,
        member_id: &str,
    ) -> Result<MemberAccountRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        self.with_connection(|conn| {
            let member = load_member(conn, organization_id, member_id)?;
            let currency = organization_currency_conn(conn, organization_id)?;
            let charges = load_member_charges(conn, organization_id, member_id)?;
            let payments = load_member_payments(conn, organization_id, member_id)?;
            let balance_minor = charges
                .iter()
                .filter(|charge| charge.status != "void")
                .map(|charge| charge.amount_minor - charge.paid_minor)
                .sum();
            Ok(MemberAccountRecord {
                member_id: member.id,
                currency,
                charges,
                payments,
                balance_minor,
            })
        })
    }

    pub fn record_membership_payment(
        &self,
        organization_id: &str,
        received_by: &str,
        payment: NewMembershipPayment,
    ) -> Result<MembershipPaymentRecord, DataError> {
        if payment.amount_minor <= 0 || payment.amount_minor > MAX_AMOUNT_MINOR {
            return Err(DataError::InvalidValue(
                "payment amount must be greater than zero".into(),
            ));
        }
        if !matches!(
            payment.payment_method.as_str(),
            "cash" | "transfer" | "point"
        ) {
            return Err(DataError::InvalidValue("invalid payment method".into()));
        }
        let reference = normalize_optional_text(payment.reference, 120, "payment reference")?;
        let idempotency_key = required_text(&payment.idempotency_key, "idempotency key", 1, 120)?;
        let request_hash = membership_payment_hash(
            &payment.charge_id,
            payment.amount_minor,
            &payment.payment_method,
            reference.as_deref(),
        );
        let now = chrono::Utc::now().to_rfc3339();
        let payment_id = {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let existing: Option<(String, String)> = transaction
                .query_row(
                    "SELECT id, request_hash FROM membership_payments
                     WHERE organization_id = ?1 AND idempotency_key = ?2",
                    params![organization_id, idempotency_key],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            if let Some((existing_id, existing_hash)) = existing {
                if existing_hash != request_hash {
                    return Err(DataError::MembershipPaymentIdempotencyConflict);
                }
                transaction.commit()?;
                existing_id
            } else {
                let charge = load_charge(&transaction, organization_id, &payment.charge_id)?;
                if charge.status != "pending" {
                    return Err(DataError::MembershipChargeNotPayable);
                }
                let remaining = charge.amount_minor - charge.paid_minor;
                if payment.amount_minor > remaining {
                    return Err(DataError::MembershipPaymentExceedsBalance);
                }
                let payment_id = uuid::Uuid::new_v4().to_string();
                let receipt_number = next_membership_receipt_number(&transaction, organization_id)?;
                let cash_session_id = if payment.payment_method == "cash" {
                    Some(open_cash_session_id(&transaction, organization_id)?)
                } else {
                    None
                };
                transaction.execute(
                    "INSERT INTO membership_payments
                         (id, organization_id, charge_id, membership_id, member_id,
                          amount_minor, payment_method, reference, receipt_number,
                          cash_session_id, idempotency_key, request_hash, received_by,
                          created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    params![
                        payment_id,
                        organization_id,
                        charge.id,
                        charge.membership_id,
                        charge.member_id,
                        payment.amount_minor,
                        payment.payment_method,
                        reference,
                        receipt_number,
                        cash_session_id,
                        idempotency_key,
                        request_hash,
                        received_by,
                        now
                    ],
                )?;
                if let Some(cash_session_id) = cash_session_id.as_deref() {
                    let movement_id = uuid::Uuid::new_v4().to_string();
                    let evidence = format!("Pago membresía {receipt_number}");
                    transaction.execute(
                        "INSERT INTO cash_movements
                             (id, organization_id, cash_session_id, movement_type, category,
                              amount_minor, payment_method, evidence, authorized_by,
                              membership_payment_id, created_at)
                         VALUES (?1, ?2, ?3, 'income', 'membership_payment', ?4, 'cash',
                                 ?5, ?6, ?7, ?8)",
                        params![
                            movement_id,
                            organization_id,
                            cash_session_id,
                            payment.amount_minor,
                            evidence,
                            received_by,
                            payment_id,
                            now
                        ],
                    )?;
                    enqueue_sync_event(
                        &transaction,
                        organization_id,
                        "cash.movement_recorded",
                        "cash_movement",
                        &movement_id,
                        &format!("cash-movement:{movement_id}:created"),
                        &serde_json::json!({
                            "movementId": movement_id,
                            "cashSessionId": cash_session_id,
                            "movementType": "income",
                            "category": "membership_payment",
                            "amountMinor": payment.amount_minor,
                            "paymentMethod": "cash",
                            "membershipPaymentId": payment_id,
                            "receiptNumber": receipt_number,
                            "createdAt": now,
                        }),
                        &now,
                    )?;
                }
                let paid_minor = charge.paid_minor + payment.amount_minor;
                let status = if paid_minor == charge.amount_minor {
                    "paid"
                } else {
                    "pending"
                };
                transaction.execute(
                    "UPDATE membership_charges
                     SET paid_minor = ?1, status = ?2, updated_at = ?3
                     WHERE organization_id = ?4 AND id = ?5",
                    params![paid_minor, status, now, organization_id, charge.id],
                )?;
                enqueue_sync_event(
                    &transaction,
                    organization_id,
                    "membership.payment_recorded",
                    "membership_payment",
                    &payment_id,
                    &format!("membership-payment:{payment_id}:recorded"),
                    &serde_json::json!({
                        "paymentId": payment_id,
                        "chargeId": charge.id,
                        "membershipId": charge.membership_id,
                        "memberId": charge.member_id,
                        "amountMinor": payment.amount_minor,
                        "paymentMethod": payment.payment_method,
                        "reference": reference,
                        "receiptNumber": receipt_number,
                        "cashSessionId": cash_session_id,
                        "receivedBy": received_by,
                        "createdAt": now,
                    }),
                    &now,
                )?;
                transaction.commit()?;
                payment_id
            }
        };
        self.membership_payment(organization_id, &payment_id)
    }

    pub fn membership_payment(
        &self,
        organization_id: &str,
        payment_id: &str,
    ) -> Result<MembershipPaymentRecord, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT p.id, p.charge_id, p.membership_id, p.member_id, p.amount_minor,
                        p.payment_method, p.reference, p.receipt_number, p.cash_session_id,
                        p.received_by, u.name, p.created_at
                 FROM membership_payments p
                 LEFT JOIN users u ON u.id = p.received_by
                 WHERE p.organization_id = ?1 AND p.id = ?2",
                params![organization_id, payment_id],
                map_membership_payment,
            )
            .optional()?
            .ok_or(DataError::MembershipPaymentNotFound)
        })
    }

    pub fn membership_payment_receipt(
        &self,
        organization_id: &str,
        payment_id: &str,
    ) -> Result<MembershipPaymentReceiptRecord, DataError> {
        self.with_connection(|conn| {
            load_membership_payment_receipt(conn, organization_id, payment_id)
        })
    }

    pub fn membership_dashboard(
        &self,
        organization_id: &str,
    ) -> Result<MembershipDashboardRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        self.with_connection(|conn| {
            let today = today();
            let month_start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .ok_or_else(|| DataError::InvalidValue("invalid current date".into()))?;
            let next_month = if today.month() == 12 {
                chrono::NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
            }
            .ok_or_else(|| DataError::InvalidValue("invalid current date".into()))?;
            let period_end = next_month - chrono::TimeDelta::days(1);
            let soon = today + chrono::TimeDelta::days(7);
            let active_members: i64 = conn.query_row(
                "SELECT count(*) FROM members
                 WHERE organization_id = ?1 AND status = 'active'",
                params![organization_id],
                |row| row.get(0),
            )?;
            let active_memberships: i64 = conn.query_row(
                "SELECT count(*) FROM memberships
                 WHERE organization_id = ?1 AND status = 'active'",
                params![organization_id],
                |row| row.get(0),
            )?;
            let expiring_soon: i64 = conn.query_row(
                "SELECT count(*) FROM memberships
                 WHERE organization_id = ?1 AND status = 'active'
                   AND end_date BETWEEN ?2 AND ?3",
                params![organization_id, date_string(today), date_string(soon)],
                |row| row.get(0),
            )?;
            let past_due: i64 = conn.query_row(
                "SELECT count(DISTINCT member_id) FROM membership_charges
                 WHERE organization_id = ?1 AND status = 'pending' AND due_date < ?2",
                params![organization_id, date_string(today)],
                |row| row.get(0),
            )?;
            let debt_minor: i64 = conn.query_row(
                "SELECT COALESCE(SUM(amount_minor - paid_minor), 0)
                 FROM membership_charges
                 WHERE organization_id = ?1 AND status = 'pending'",
                params![organization_id],
                |row| row.get(0),
            )?;
            let new_members_this_month: i64 = conn.query_row(
                "SELECT count(*) FROM members
                 WHERE organization_id = ?1 AND joined_at >= ?2 AND joined_at < ?3",
                params![
                    organization_id,
                    date_string(month_start),
                    date_string(next_month)
                ],
                |row| row.get(0),
            )?;
            let income_this_month_minor: i64 = conn.query_row(
                "SELECT COALESCE(SUM(amount_minor), 0) FROM membership_payments
                 WHERE organization_id = ?1 AND created_at >= ?2 AND created_at < ?3",
                params![
                    organization_id,
                    date_string(month_start),
                    date_string(next_month)
                ],
                |row| row.get(0),
            )?;
            let renewals_this_month: i64 = conn.query_row(
                "SELECT count(*) FROM membership_events
                 WHERE organization_id = ?1 AND event_type = 'renewed'
                   AND created_at >= ?2 AND created_at < ?3",
                params![
                    organization_id,
                    date_string(month_start),
                    date_string(next_month)
                ],
                |row| row.get(0),
            )?;
            let cancellations_this_month: i64 = conn.query_row(
                "SELECT count(*) FROM membership_events
                 WHERE organization_id = ?1 AND event_type = 'cancelled'
                   AND created_at >= ?2 AND created_at < ?3",
                params![
                    organization_id,
                    date_string(month_start),
                    date_string(next_month)
                ],
                |row| row.get(0),
            )?;
            let trend = dashboard_trend(conn, organization_id, month_start)?;
            let recent_payments = load_recent_payment_receipts(conn, organization_id, 5)?;
            let expiring_memberships =
                load_expiring_memberships(conn, organization_id, today, soon, 5)?;
            Ok(MembershipDashboardRecord {
                period_start: date_string(month_start),
                period_end: date_string(period_end),
                timezone: "UTC".into(),
                active_members,
                active_memberships,
                expiring_soon,
                past_due,
                debt_minor,
                new_members_this_month,
                income_this_month_minor,
                renewals_this_month,
                cancellations_this_month,
                trend,
                recent_payments,
                expiring_memberships,
            })
        })
    }

    fn refresh_memberships(&self, organization_id: &str) -> Result<(), DataError> {
        let today = date_string(today());
        let now = chrono::Utc::now().to_rfc3339();
        self.with_connection(|conn| {
            conn.execute(
                "UPDATE memberships
                 SET status = 'active', updated_at = ?1
                 WHERE organization_id = ?2 AND status = 'scheduled'
                   AND start_date <= ?3 AND end_date >= ?3",
                params![now, organization_id, today],
            )?;
            conn.execute(
                "UPDATE memberships
                 SET status = 'expired', updated_at = ?1
                 WHERE organization_id = ?2 AND status IN ('scheduled', 'active')
                   AND end_date < ?3",
                params![now, organization_id, today],
            )?;
            Ok(())
        })
    }
}

fn dashboard_trend(
    conn: &Connection,
    organization_id: &str,
    current_month_start: chrono::NaiveDate,
) -> Result<Vec<MembershipDashboardTrendRecord>, DataError> {
    let mut months = Vec::with_capacity(6);
    let base =
        current_month_start.year() * 12 + i32::try_from(current_month_start.month0()).unwrap_or(0);
    for offset in (0..6).rev() {
        let index = base - offset;
        let year = index.div_euclid(12);
        let month = u32::try_from(index.rem_euclid(12) + 1).unwrap_or(1);
        let date = chrono::NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| DataError::InvalidValue("invalid trend month".into()))?;
        months.push(date.format("%Y-%m").to_string());
    }
    let first = months
        .first()
        .ok_or_else(|| DataError::InvalidValue("invalid trend range".into()))?;

    let mut new_members = HashMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT strftime('%Y-%m', joined_at), count(*) FROM members
             WHERE organization_id = ?1 AND joined_at >= ?2
             GROUP BY 1",
        )?;
        let rows = statement.query_map(params![organization_id, first], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (month, count) = row?;
            new_members.insert(month, count);
        }
    }
    let mut renewals = HashMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT strftime('%Y-%m', created_at), count(*) FROM membership_events
             WHERE organization_id = ?1 AND event_type = 'renewed' AND created_at >= ?2
             GROUP BY 1",
        )?;
        let rows = statement.query_map(params![organization_id, first], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (month, count) = row?;
            renewals.insert(month, count);
        }
    }
    let mut cancellations = HashMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT strftime('%Y-%m', created_at), count(*) FROM membership_events
             WHERE organization_id = ?1 AND event_type = 'cancelled' AND created_at >= ?2
             GROUP BY 1",
        )?;
        let rows = statement.query_map(params![organization_id, first], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (month, count) = row?;
            cancellations.insert(month, count);
        }
    }
    let mut income = HashMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT strftime('%Y-%m', created_at), COALESCE(SUM(amount_minor), 0)
             FROM membership_payments
             WHERE organization_id = ?1 AND created_at >= ?2
             GROUP BY 1",
        )?;
        let rows = statement.query_map(params![organization_id, first], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (month, amount) = row?;
            income.insert(month, amount);
        }
    }

    Ok(months
        .into_iter()
        .map(|month| MembershipDashboardTrendRecord {
            new_members: new_members.get(&month).copied().unwrap_or(0),
            renewals: renewals.get(&month).copied().unwrap_or(0),
            cancellations: cancellations.get(&month).copied().unwrap_or(0),
            income_minor: income.get(&month).copied().unwrap_or(0),
            month,
        })
        .collect())
}

fn load_recent_payment_receipts(
    conn: &Connection,
    organization_id: &str,
    limit: i64,
) -> Result<Vec<MembershipPaymentReceiptRecord>, DataError> {
    let mut statement = conn.prepare(
        "SELECT p.id, COALESCE(p.receipt_number, ''),
                o.name, mem.member_number,
                TRIM(mem.first_name || ' ' || mem.last_name),
                c.description, p.amount_minor, c.currency, p.payment_method,
                p.reference, u.name, p.created_at
         FROM membership_payments p
         JOIN organizations o ON o.id = p.organization_id
         JOIN members mem
           ON mem.organization_id = p.organization_id AND mem.id = p.member_id
         JOIN membership_charges c
           ON c.organization_id = p.organization_id AND c.id = p.charge_id
         LEFT JOIN users u ON u.id = p.received_by
         WHERE p.organization_id = ?1
         ORDER BY p.created_at DESC LIMIT ?2",
    )?;
    let rows = statement
        .query_map(
            params![organization_id, limit.clamp(1, 20)],
            map_membership_payment_receipt,
        )?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DataError::from)?;
    Ok(rows)
}

fn load_expiring_memberships(
    conn: &Connection,
    organization_id: &str,
    today: chrono::NaiveDate,
    soon: chrono::NaiveDate,
    limit: i64,
) -> Result<Vec<MembershipRecord>, DataError> {
    let mut statement = conn.prepare(
        "SELECT m.id, m.member_id, TRIM(mem.first_name || ' ' || mem.last_name),
                m.plan_id, m.plan_name, m.plan_version, m.plan_version_id,
                m.status, m.start_date, m.end_date,
                m.price_minor, m.currency, m.grace_days, m.access_days,
                m.access_start_minute, m.access_end_minute, m.min_age, m.max_age,
                m.paused_at, m.pause_until, m.cancellation_reason,
                m.created_at, m.updated_at
         FROM memberships m
         JOIN members mem
           ON mem.organization_id = m.organization_id AND mem.id = m.member_id
         WHERE m.organization_id = ?1 AND m.status = 'active'
           AND m.end_date BETWEEN ?2 AND ?3
         ORDER BY m.end_date, mem.last_name COLLATE NOCASE
         LIMIT ?4",
    )?;
    let rows = statement
        .query_map(
            params![
                organization_id,
                date_string(today),
                date_string(soon),
                limit.clamp(1, 20)
            ],
            map_membership,
        )?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DataError::from)?;
    Ok(rows)
}

fn validate_membership_plan(plan: NewMembershipPlan) -> Result<ValidatedMembershipPlan, DataError> {
    let min_age = validate_age(plan.min_age, "minimum age")?;
    let max_age = validate_age(plan.max_age, "maximum age")?;
    validate_age_range(min_age, max_age)?;
    let access_start_minute =
        validate_access_minute(plan.access_start_minute, "access start minute", 0, 1439)?;
    let access_end_minute =
        validate_access_minute(plan.access_end_minute, "access end minute", 1, 1440)?;
    validate_access_window(access_start_minute, access_end_minute)?;
    Ok(ValidatedMembershipPlan {
        name: required_text(&plan.name, "plan name", 2, 80)?,
        description: normalize_optional_text(plan.description, 300, "description")?,
        duration_days: validate_duration(plan.duration_days)?,
        price_minor: validate_price(plan.price_minor)?,
        grace_days: validate_grace_days(plan.grace_days)?,
        access_days: validate_access_days(&plan.access_days)?,
        access_start_minute,
        access_end_minute,
        min_age,
        max_age,
        active: true,
    })
}

fn validate_membership_plan_update(
    plan: UpdateMembershipPlan,
) -> Result<ValidatedMembershipPlan, DataError> {
    let min_age = validate_age(plan.min_age, "minimum age")?;
    let max_age = validate_age(plan.max_age, "maximum age")?;
    validate_age_range(min_age, max_age)?;
    let access_start_minute =
        validate_access_minute(plan.access_start_minute, "access start minute", 0, 1439)?;
    let access_end_minute =
        validate_access_minute(plan.access_end_minute, "access end minute", 1, 1440)?;
    validate_access_window(access_start_minute, access_end_minute)?;
    Ok(ValidatedMembershipPlan {
        name: required_text(&plan.name, "plan name", 2, 80)?,
        description: normalize_optional_text(plan.description, 300, "description")?,
        duration_days: validate_duration(plan.duration_days)?,
        price_minor: validate_price(plan.price_minor)?,
        grace_days: validate_grace_days(plan.grace_days)?,
        access_days: validate_access_days(&plan.access_days)?,
        access_start_minute,
        access_end_minute,
        min_age,
        max_age,
        active: plan.active,
    })
}

fn validate_duration(value: i64) -> Result<i64, DataError> {
    if (1..=3660).contains(&value) {
        Ok(value)
    } else {
        Err(DataError::InvalidValue(
            "plan duration must be between 1 and 3660 days".into(),
        ))
    }
}

fn validate_price(value: i64) -> Result<i64, DataError> {
    if (0..=MAX_AMOUNT_MINOR).contains(&value) {
        Ok(value)
    } else {
        Err(DataError::InvalidValue(
            "plan price must be between zero and the maximum amount".into(),
        ))
    }
}

fn validate_grace_days(value: i64) -> Result<i64, DataError> {
    if (0..=90).contains(&value) {
        Ok(value)
    } else {
        Err(DataError::InvalidValue(
            "grace days must be between 0 and 90".into(),
        ))
    }
}

fn validate_access_days(value: &str) -> Result<String, DataError> {
    let mut days = Vec::new();
    for day in value.split(',') {
        let day = day.trim().to_ascii_lowercase();
        if day.is_empty() {
            continue;
        }
        if !matches!(
            day.as_str(),
            "mon" | "tue" | "wed" | "thu" | "fri" | "sat" | "sun"
        ) {
            return Err(DataError::InvalidValue(
                "access days must use mon, tue, wed, thu, fri, sat or sun".into(),
            ));
        }
        if !days.contains(&day) {
            days.push(day);
        }
    }
    if days.is_empty() {
        return Err(DataError::InvalidValue(
            "a plan must allow at least one access day".into(),
        ));
    }
    Ok(days.join(","))
}

fn validate_access_minute(
    value: i64,
    field: &str,
    minimum: i64,
    maximum: i64,
) -> Result<i64, DataError> {
    if (minimum..=maximum).contains(&value) {
        Ok(value)
    } else {
        Err(DataError::InvalidValue(format!(
            "{field} must be between {minimum} and {maximum}"
        )))
    }
}

fn validate_age(value: Option<i64>, field: &str) -> Result<Option<i64>, DataError> {
    match value {
        Some(age) if !(0..=120).contains(&age) => Err(DataError::InvalidValue(format!(
            "{field} must be between 0 and 120"
        ))),
        Some(age) => Ok(Some(age)),
        None => Ok(None),
    }
}

fn validate_access_window(start: i64, end: i64) -> Result<(), DataError> {
    if start >= end {
        return Err(DataError::InvalidValue(
            "access start time must be before the end time".into(),
        ));
    }
    Ok(())
}

fn validate_age_range(min_age: Option<i64>, max_age: Option<i64>) -> Result<(), DataError> {
    if let (Some(min), Some(max)) = (min_age, max_age) {
        if max < min {
            return Err(DataError::InvalidValue(
                "maximum age cannot be lower than minimum age".into(),
            ));
        }
    }
    Ok(())
}

fn member_age_on(birth_date: &str, at: chrono::NaiveDate) -> Result<i64, DataError> {
    let birth = parse_date(birth_date, "birth date")?;
    if birth > at {
        return Err(DataError::InvalidValue(
            "birth date cannot be after the membership start date".into(),
        ));
    }
    let mut age = at.year() - birth.year();
    if (at.month(), at.day()) < (birth.month(), birth.day()) {
        age -= 1;
    }
    Ok(i64::from(age))
}

fn ensure_plan_eligibility(
    plan: &MembershipPlanRecord,
    member: &MemberBase,
    start: chrono::NaiveDate,
) -> Result<(), DataError> {
    if plan.min_age.is_none() && plan.max_age.is_none() {
        return Ok(());
    }
    let birth_date = member.birth_date.as_deref().ok_or_else(|| {
        DataError::InvalidValue("the member needs a birth date for this plan's age rules".into())
    })?;
    let age = member_age_on(birth_date, start)?;
    if plan.min_age.is_some_and(|minimum| age < minimum) {
        return Err(DataError::InvalidValue(
            "the member does not meet the plan's minimum age".into(),
        ));
    }
    if plan.max_age.is_some_and(|maximum| age > maximum) {
        return Err(DataError::InvalidValue(
            "the member exceeds the plan's maximum age".into(),
        ));
    }
    Ok(())
}

fn validate_new_member(member: NewMember) -> Result<ValidatedMember, DataError> {
    Ok(ValidatedMember {
        first_name: required_text(&member.first_name, "first name", 1, 80)?,
        last_name: required_text_allow_empty(&member.last_name, "last name", 80)?,
        phone: normalize_optional_text(member.phone, 40, "phone")?,
        email: validate_email(member.email)?,
        birth_date: validate_birth_date(member.birth_date)?,
        notes: normalize_optional_text(member.notes, 500, "notes")?,
    })
}

fn required_text_allow_empty(
    value: &str,
    field: &str,
    max_chars: usize,
) -> Result<String, DataError> {
    let value = value.trim();
    if value.chars().count() > max_chars {
        return Err(DataError::InvalidValue(format!(
            "{field} must have at most {max_chars} characters"
        )));
    }
    Ok(value.to_string())
}

fn validate_email(value: Option<String>) -> Result<Option<String>, DataError> {
    let Some(value) = normalize_optional_text(value, 120, "email")? else {
        return Ok(None);
    };
    if !value.contains('@') {
        return Err(DataError::InvalidValue("email is not valid".into()));
    }
    Ok(Some(value.to_lowercase()))
}

fn validate_birth_date(value: Option<String>) -> Result<Option<String>, DataError> {
    let Some(value) = normalize_optional_text(value, 10, "birth date")? else {
        return Ok(None);
    };
    let date = parse_date(&value, "birth date")?;
    if date > today() {
        return Err(DataError::InvalidValue(
            "birth date cannot be in the future".into(),
        ));
    }
    Ok(Some(date_string(date)))
}

fn parse_date(value: &str, field: &str) -> Result<chrono::NaiveDate, DataError> {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| DataError::InvalidValue(format!("{field} must use YYYY-MM-DD")))
}

fn date_string(value: chrono::NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn today() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

fn split_prospect_name(name: &str) -> (String, String) {
    let mut parts = name.splitn(2, char::is_whitespace);
    let first = parts.next().unwrap_or(name).trim().to_string();
    let last = parts.next().unwrap_or("").trim().to_string();
    (first, last)
}

fn insert_member(
    transaction: &Transaction<'_>,
    organization_id: &str,
    member: &ValidatedMember,
    joined_at: &str,
    now: &str,
) -> Result<String, DataError> {
    let next: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(CAST(SUBSTR(member_number, 5) AS INTEGER)), 0) + 1
         FROM members WHERE organization_id = ?1",
        params![organization_id],
        |row| row.get(0),
    )?;
    let member_number = format!("SOC-{next:05}");
    let id = uuid::Uuid::new_v4().to_string();
    transaction.execute(
        "INSERT INTO members
             (id, organization_id, member_number, first_name, last_name, phone, email,
              birth_date, status, joined_at, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?10, ?11, ?11)",
        params![
            id,
            organization_id,
            member_number,
            member.first_name,
            member.last_name,
            member.phone,
            member.email,
            member.birth_date,
            joined_at,
            member.notes,
            now
        ],
    )?;
    Ok(id)
}

fn ensure_unique_membership_plan(
    transaction: &Transaction<'_>,
    organization_id: &str,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let duplicate: bool = transaction.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM membership_plans
             WHERE organization_id = ?1
               AND name = ?2 COLLATE NOCASE
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, name, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        Err(DataError::MembershipPlanAlreadyExists)
    } else {
        Ok(())
    }
}

fn insert_membership_plan_version(
    transaction: &Transaction<'_>,
    organization_id: &str,
    plan_id: &str,
    version: i64,
    plan: &ValidatedMembershipPlan,
    currency: &str,
    now: &str,
) -> Result<(), DataError> {
    transaction.execute(
        "INSERT INTO membership_plan_versions
             (id, organization_id, plan_id, version, name, description, duration_days,
              price_minor, currency, grace_days, access_days, access_start_minute,
              access_end_minute, min_age, max_age, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            uuid::Uuid::new_v4().to_string(),
            organization_id,
            plan_id,
            version,
            plan.name,
            plan.description,
            plan.duration_days,
            plan.price_minor,
            currency,
            plan.grace_days,
            plan.access_days,
            plan.access_start_minute,
            plan.access_end_minute,
            plan.min_age,
            plan.max_age,
            now
        ],
    )?;
    Ok(())
}

fn load_plan_version_id(
    conn: &Connection,
    organization_id: &str,
    plan_id: &str,
    version: i64,
) -> Result<String, DataError> {
    conn.query_row(
        "SELECT id FROM membership_plan_versions
         WHERE organization_id = ?1 AND plan_id = ?2 AND version = ?3",
        params![organization_id, plan_id, version],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| DataError::InvalidValue("membership plan version not found".into()))
}

fn load_membership_plan(
    conn: &Connection,
    organization_id: &str,
    plan_id: &str,
) -> Result<MembershipPlanRecord, DataError> {
    conn.query_row(
        "SELECT id, name, description, duration_days, price_minor, currency,
                grace_days, current_version, access_days, access_start_minute,
                access_end_minute, min_age, max_age, active, created_at, updated_at
         FROM membership_plans
         WHERE organization_id = ?1 AND id = ?2",
        params![organization_id, plan_id],
        map_membership_plan,
    )
    .optional()?
    .ok_or(DataError::MembershipPlanNotFound)
}

fn map_membership_plan(row: &Row<'_>) -> rusqlite::Result<MembershipPlanRecord> {
    Ok(MembershipPlanRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        duration_days: row.get(3)?,
        price_minor: row.get(4)?,
        currency: row.get(5)?,
        grace_days: row.get(6)?,
        version: row.get(7)?,
        access_days: row.get(8)?,
        access_start_minute: row.get(9)?,
        access_end_minute: row.get(10)?,
        min_age: row.get(11)?,
        max_age: row.get(12)?,
        active: row.get::<_, i64>(13)? != 0,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

fn map_member_base(row: &Row<'_>) -> rusqlite::Result<MemberBase> {
    Ok(MemberBase {
        id: row.get(0)?,
        member_number: row.get(1)?,
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        phone: row.get(4)?,
        email: row.get(5)?,
        birth_date: row.get(6)?,
        status: row.get(7)?,
        joined_at: row.get(8)?,
        notes: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn load_member_base(
    conn: &Connection,
    organization_id: &str,
    member_id: &str,
) -> Result<MemberBase, DataError> {
    conn.query_row(
        "SELECT id, member_number, first_name, last_name, phone, email,
                birth_date, status, joined_at, notes, created_at, updated_at
         FROM members
         WHERE organization_id = ?1 AND id = ?2",
        params![organization_id, member_id],
        map_member_base,
    )
    .optional()?
    .ok_or(DataError::MemberNotFound)
}

fn load_member(
    conn: &Connection,
    organization_id: &str,
    member_id: &str,
) -> Result<MemberRecord, DataError> {
    let base = load_member_base(conn, organization_id, member_id)?;
    hydrate_member(conn, organization_id, base)
}

fn hydrate_member(
    conn: &Connection,
    organization_id: &str,
    base: MemberBase,
) -> Result<MemberRecord, DataError> {
    let active_membership = conn
        .query_row(
            "SELECT m.id, m.member_id, TRIM(mem.first_name || ' ' || mem.last_name),
                    m.plan_id, m.plan_name, m.plan_version, m.plan_version_id,
                    m.status, m.start_date, m.end_date,
                    m.price_minor, m.currency, m.grace_days, m.access_days,
                    m.access_start_minute, m.access_end_minute, m.min_age, m.max_age,
                    m.paused_at, m.pause_until, m.cancellation_reason,
                    m.created_at, m.updated_at
             FROM memberships m
             JOIN members mem
               ON mem.organization_id = m.organization_id AND mem.id = m.member_id
             WHERE m.organization_id = ?1 AND m.member_id = ?2
               AND m.status IN ('scheduled', 'active', 'paused')
             ORDER BY m.created_at DESC
             LIMIT 1",
            params![organization_id, base.id],
            map_membership,
        )
        .optional()?;
    let debt_minor: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_minor - paid_minor), 0)
         FROM membership_charges
         WHERE organization_id = ?1 AND member_id = ?2 AND status = 'pending'",
        params![organization_id, base.id],
        |row| row.get(0),
    )?;
    Ok(MemberRecord {
        id: base.id,
        member_number: base.member_number,
        first_name: base.first_name,
        last_name: base.last_name,
        phone: base.phone,
        email: base.email,
        birth_date: base.birth_date,
        status: base.status,
        joined_at: base.joined_at,
        notes: base.notes,
        active_membership,
        debt_minor,
        created_at: base.created_at,
        updated_at: base.updated_at,
    })
}

fn map_prospect(row: &Row<'_>) -> rusqlite::Result<ProspectRecord> {
    Ok(ProspectRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        phone: row.get(2)?,
        email: row.get(3)?,
        source: row.get(4)?,
        status: row.get(5)?,
        notes: row.get(6)?,
        converted_member_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn load_prospect(
    conn: &Connection,
    organization_id: &str,
    prospect_id: &str,
) -> Result<ProspectRecord, DataError> {
    conn.query_row(
        "SELECT id, name, phone, email, source, status, notes,
                converted_member_id, created_at, updated_at
         FROM prospects
         WHERE organization_id = ?1 AND id = ?2",
        params![organization_id, prospect_id],
        map_prospect,
    )
    .optional()?
    .ok_or(DataError::ProspectNotFound)
}

fn map_membership(row: &Row<'_>) -> rusqlite::Result<MembershipRecord> {
    Ok(MembershipRecord {
        id: row.get(0)?,
        member_id: row.get(1)?,
        member_name: row.get(2)?,
        plan_id: row.get(3)?,
        plan_name: row.get(4)?,
        plan_version: row.get(5)?,
        plan_version_id: row.get(6)?,
        status: row.get(7)?,
        start_date: row.get(8)?,
        end_date: row.get(9)?,
        price_minor: row.get(10)?,
        currency: row.get(11)?,
        grace_days: row.get(12)?,
        access_days: row.get(13)?,
        access_start_minute: row.get(14)?,
        access_end_minute: row.get(15)?,
        min_age: row.get(16)?,
        max_age: row.get(17)?,
        paused_at: row.get(18)?,
        pause_until: row.get(19)?,
        cancellation_reason: row.get(20)?,
        created_at: row.get(21)?,
        updated_at: row.get(22)?,
    })
}

fn load_membership(
    conn: &Connection,
    organization_id: &str,
    membership_id: &str,
) -> Result<MembershipRecord, DataError> {
    conn.query_row(
        "SELECT m.id, m.member_id, TRIM(mem.first_name || ' ' || mem.last_name),
                m.plan_id, m.plan_name, m.plan_version, m.plan_version_id,
                m.status, m.start_date, m.end_date,
                m.price_minor, m.currency, m.grace_days, m.access_days,
                m.access_start_minute, m.access_end_minute, m.min_age, m.max_age,
                m.paused_at, m.pause_until, m.cancellation_reason,
                m.created_at, m.updated_at
         FROM memberships m
         JOIN members mem
           ON mem.organization_id = m.organization_id AND mem.id = m.member_id
         WHERE m.organization_id = ?1 AND m.id = ?2",
        params![organization_id, membership_id],
        map_membership,
    )
    .optional()?
    .ok_or(DataError::MembershipNotFound)
}

fn map_charge(row: &Row<'_>) -> rusqlite::Result<MembershipChargeRecord> {
    Ok(MembershipChargeRecord {
        id: row.get(0)?,
        membership_id: row.get(1)?,
        member_id: row.get(2)?,
        description: row.get(3)?,
        amount_minor: row.get(4)?,
        currency: row.get(5)?,
        due_date: row.get(6)?,
        status: row.get(7)?,
        paid_minor: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn load_charge(
    conn: &Connection,
    organization_id: &str,
    charge_id: &str,
) -> Result<MembershipChargeRecord, DataError> {
    conn.query_row(
        "SELECT id, membership_id, member_id, description, amount_minor,
                currency, due_date, status, paid_minor, created_at, updated_at
         FROM membership_charges
         WHERE organization_id = ?1 AND id = ?2",
        params![organization_id, charge_id],
        map_charge,
    )
    .optional()?
    .ok_or(DataError::MembershipChargeNotFound)
}

fn load_member_charges(
    conn: &Connection,
    organization_id: &str,
    member_id: &str,
) -> Result<Vec<MembershipChargeRecord>, DataError> {
    let mut statement = conn.prepare(
        "SELECT id, membership_id, member_id, description, amount_minor,
                currency, due_date, status, paid_minor, created_at, updated_at
         FROM membership_charges
         WHERE organization_id = ?1 AND member_id = ?2
         ORDER BY due_date DESC, created_at DESC",
    )?;
    let charges = statement
        .query_map(params![organization_id, member_id], map_charge)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DataError::from)?;
    Ok(charges)
}

fn map_membership_payment(row: &Row<'_>) -> rusqlite::Result<MembershipPaymentRecord> {
    Ok(MembershipPaymentRecord {
        id: row.get(0)?,
        charge_id: row.get(1)?,
        membership_id: row.get(2)?,
        member_id: row.get(3)?,
        amount_minor: row.get(4)?,
        payment_method: row.get(5)?,
        reference: row.get(6)?,
        receipt_number: row.get(7)?,
        cash_session_id: row.get(8)?,
        received_by: row.get(9)?,
        received_by_name: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn map_membership_payment_receipt(
    row: &Row<'_>,
) -> rusqlite::Result<MembershipPaymentReceiptRecord> {
    Ok(MembershipPaymentReceiptRecord {
        payment_id: row.get(0)?,
        receipt_number: row.get(1)?,
        gym_name: row.get(2)?,
        member_number: row.get(3)?,
        member_name: row.get(4)?,
        charge_description: row.get(5)?,
        amount_minor: row.get(6)?,
        currency: row.get(7)?,
        payment_method: row.get(8)?,
        reference: row.get(9)?,
        received_by_name: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn load_membership_payment_receipt(
    conn: &Connection,
    organization_id: &str,
    payment_id: &str,
) -> Result<MembershipPaymentReceiptRecord, DataError> {
    conn.query_row(
        "SELECT p.id, COALESCE(p.receipt_number, ''),
                o.name, mem.member_number,
                TRIM(mem.first_name || ' ' || mem.last_name),
                c.description, p.amount_minor, c.currency, p.payment_method,
                p.reference, u.name, p.created_at
         FROM membership_payments p
         JOIN organizations o ON o.id = p.organization_id
         JOIN members mem
           ON mem.organization_id = p.organization_id AND mem.id = p.member_id
         JOIN membership_charges c
           ON c.organization_id = p.organization_id AND c.id = p.charge_id
         LEFT JOIN users u ON u.id = p.received_by
         WHERE p.organization_id = ?1 AND p.id = ?2",
        params![organization_id, payment_id],
        map_membership_payment_receipt,
    )
    .optional()?
    .ok_or(DataError::MembershipPaymentNotFound)
}

fn load_member_payments(
    conn: &Connection,
    organization_id: &str,
    member_id: &str,
) -> Result<Vec<MembershipPaymentRecord>, DataError> {
    let mut statement = conn.prepare(
        "SELECT p.id, p.charge_id, p.membership_id, p.member_id, p.amount_minor,
                p.payment_method, p.reference, p.receipt_number, p.cash_session_id,
                p.received_by, u.name, p.created_at
         FROM membership_payments p
         LEFT JOIN users u ON u.id = p.received_by
         WHERE p.organization_id = ?1 AND p.member_id = ?2
         ORDER BY p.created_at DESC",
    )?;
    let payments = statement
        .query_map(params![organization_id, member_id], map_membership_payment)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DataError::from)?;
    Ok(payments)
}

#[allow(clippy::too_many_arguments)]
fn insert_membership_event(
    transaction: &Transaction<'_>,
    organization_id: &str,
    membership_id: &str,
    event_type: &str,
    from_status: Option<&str>,
    to_status: &str,
    note: Option<&str>,
    actor_id: &str,
    created_at: &str,
) -> Result<(), DataError> {
    transaction.execute(
        "INSERT INTO membership_events
             (id, organization_id, membership_id, event_type, from_status,
              to_status, note, actor_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            uuid::Uuid::new_v4().to_string(),
            organization_id,
            membership_id,
            event_type,
            from_status,
            to_status,
            note,
            actor_id,
            created_at
        ],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_membership_charge(
    transaction: &Transaction<'_>,
    organization_id: &str,
    membership_id: &str,
    member_id: &str,
    description: &str,
    amount_minor: i64,
    currency: &str,
    due_date: &str,
    created_at: &str,
) -> Result<String, DataError> {
    let id = uuid::Uuid::new_v4().to_string();
    transaction.execute(
        "INSERT INTO membership_charges
             (id, organization_id, membership_id, member_id, description,
              amount_minor, currency, due_date, status, paid_minor,
              created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', 0, ?9, ?9)",
        params![
            id,
            organization_id,
            membership_id,
            member_id,
            description,
            amount_minor,
            currency,
            due_date,
            created_at
        ],
    )?;
    Ok(id)
}

fn next_membership_receipt_number(
    conn: &Connection,
    organization_id: &str,
) -> Result<String, DataError> {
    let next: i64 = conn.query_row(
        "SELECT COALESCE(MAX(CAST(SUBSTR(receipt_number, 5) AS INTEGER)), 0) + 1
         FROM membership_payments
         WHERE organization_id = ?1 AND receipt_number IS NOT NULL",
        params![organization_id],
        |row| row.get(0),
    )?;
    Ok(format!("REC-{next:05}"))
}

fn open_cash_session_id(conn: &Connection, organization_id: &str) -> Result<String, DataError> {
    conn.query_row(
        "SELECT id FROM cash_sessions
         WHERE organization_id = ?1 AND status = 'open'
         ORDER BY opened_at DESC LIMIT 1",
        params![organization_id],
        |row| row.get(0),
    )
    .optional()?
    .ok_or(DataError::CashSessionNotOpen)
}

fn membership_payment_hash(
    charge_id: &str,
    amount_minor: i64,
    payment_method: &str,
    reference: Option<&str>,
) -> String {
    crate::sha256_hex(format!(
        "{}|{}|{}|{}",
        charge_id,
        amount_minor,
        payment_method,
        reference.unwrap_or("none")
    ))
}

fn organization_currency_conn(
    conn: &Connection,
    organization_id: &str,
) -> Result<String, DataError> {
    conn.query_row(
        "SELECT currency FROM organizations WHERE id = ?1",
        params![organization_id],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| DataError::InvalidValue("the gym does not exist".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbKey, MIGRATIONS};
    use std::path::PathBuf;

    fn test_db(name: &str) -> (Db, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "fitness-gym-members-{name}-{}.db",
            uuid::Uuid::new_v4()
        ));
        let key = DbKey::from_bytes([7u8; 32]);
        let db = Db::open(&path, &key).expect("opens test database");
        (db, path)
    }

    fn setup(db: &Db) -> String {
        let organization_id = db.create_organization("Test Gym").expect("organization");
        db.create_owner(&organization_id, "Owner", "not-used-in-this-test", None)
            .expect("owner");
        organization_id
    }

    fn owner_id(db: &Db, organization_id: &str) -> String {
        db.list_staff(organization_id)
            .expect("staff")
            .into_iter()
            .find(|staff| staff.role == "owner")
            .expect("owner row")
            .user_id
    }

    #[test]
    fn schema_19_is_applied() {
        let (db, path) = test_db("schema");
        let status = db.status().expect("status");
        assert_eq!(status.schema_version, MIGRATIONS.last().unwrap().version);
        assert_eq!(status.schema_version, 19);
        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn membership_lifecycle_tracks_charges_and_debt() {
        let (db, path) = test_db("lifecycle");
        let organization_id = setup(&db);
        let actor_id = owner_id(&db, &organization_id);

        let member = db
            .create_member(
                &organization_id,
                NewMember {
                    first_name: "Ana".into(),
                    last_name: "López".into(),
                    phone: Some("555-0101".into()),
                    email: Some("ANA@example.com".into()),
                    birth_date: Some("1990-05-10".into()),
                    notes: None,
                },
            )
            .expect("member");
        assert_eq!(member.member_number, "SOC-00001");
        assert_eq!(member.email.as_deref(), Some("ana@example.com"));

        let plan = db
            .create_membership_plan(
                &organization_id,
                NewMembershipPlan {
                    name: "Mensual".into(),
                    description: Some("30 días".into()),
                    duration_days: 30,
                    price_minor: 50_000,
                    grace_days: 3,
                    access_days: "mon,tue,wed,thu,fri".into(),
                    access_start_minute: 360,
                    access_end_minute: 1320,
                    min_age: None,
                    max_age: None,
                },
            )
            .expect("plan");

        let membership = db
            .create_membership(&organization_id, &actor_id, &member.id, &plan.id, None)
            .expect("membership");
        assert_eq!(membership.status, "active");
        assert_eq!(membership.member_id, member.id);

        let account = db
            .member_account(&organization_id, &member.id)
            .expect("account");
        assert_eq!(account.charges.len(), 1);
        assert_eq!(account.balance_minor, 50_000);

        let charge_id = account.charges[0].id.clone();
        let cash_session = db
            .open_cash_session(&organization_id, &actor_id, 0)
            .expect("cash session");
        let partial = db
            .record_membership_payment(
                &organization_id,
                &actor_id,
                NewMembershipPayment {
                    charge_id: charge_id.clone(),
                    amount_minor: 20_000,
                    payment_method: "cash".into(),
                    reference: None,
                    idempotency_key: "payment-1".into(),
                },
            )
            .expect("partial payment");
        assert_eq!(partial.amount_minor, 20_000);
        assert_eq!(partial.receipt_number, "REC-00001");
        assert_eq!(
            partial.cash_session_id.as_deref(),
            Some(cash_session.id.as_str())
        );
        let movements = db
            .list_cash_movements(&organization_id, &cash_session.id, 10)
            .expect("cash movements");
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].category, "membership_payment");
        assert_eq!(movements[0].amount_minor, 20_000);
        assert!(movements[0].evidence.contains("REC-00001"));
        assert_eq!(
            db.member_account(&organization_id, &member.id)
                .expect("account")
                .balance_minor,
            30_000
        );

        let duplicate = db
            .record_membership_payment(
                &organization_id,
                &actor_id,
                NewMembershipPayment {
                    charge_id: charge_id.clone(),
                    amount_minor: 20_000,
                    payment_method: "cash".into(),
                    reference: None,
                    idempotency_key: "payment-1".into(),
                },
            )
            .expect("idempotent payment");
        assert_eq!(duplicate.id, partial.id);

        db.record_membership_payment(
            &organization_id,
            &actor_id,
            NewMembershipPayment {
                charge_id,
                amount_minor: 30_000,
                payment_method: "transfer".into(),
                reference: Some("TRX-1".into()),
                idempotency_key: "payment-2".into(),
            },
        )
        .expect("full payment");
        let account = db
            .member_account(&organization_id, &member.id)
            .expect("account");
        assert_eq!(account.balance_minor, 0);
        assert_eq!(account.charges[0].status, "paid");

        let renewed = db
            .renew_membership(&organization_id, &actor_id, &membership.id, None)
            .expect("renewal");
        assert_eq!(renewed.status, "active");
        let account = db
            .member_account(&organization_id, &member.id)
            .expect("account");
        assert_eq!(account.charges.len(), 2);
        assert_eq!(account.balance_minor, 50_000);

        let paused = db
            .pause_membership(
                &organization_id,
                &actor_id,
                &membership.id,
                &date_string(today() + chrono::TimeDelta::days(3)),
            )
            .expect("pause");
        assert_eq!(paused.status, "paused");
        let resumed = db
            .resume_membership(&organization_id, &actor_id, &membership.id)
            .expect("resume");
        assert_eq!(resumed.status, "active");

        let cancelled = db
            .cancel_membership(
                &organization_id,
                &actor_id,
                &membership.id,
                "Cambio de plan",
            )
            .expect("cancel");
        assert_eq!(cancelled.status, "cancelled");
        assert_eq!(
            cancelled.cancellation_reason.as_deref(),
            Some("Cambio de plan")
        );

        let dashboard = db
            .membership_dashboard(&organization_id)
            .expect("dashboard");
        assert_eq!(dashboard.active_members, 1);
        assert_eq!(dashboard.active_memberships, 0);
        assert_eq!(dashboard.debt_minor, 50_000);
        assert_eq!(dashboard.new_members_this_month, 1);

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn plan_rules_are_versioned_and_age_gated() {
        let (db, path) = test_db("plan-rules");
        let organization_id = setup(&db);
        let actor_id = owner_id(&db, &organization_id);

        let member = db
            .create_member(
                &organization_id,
                NewMember {
                    first_name: "Luis".into(),
                    last_name: "Pérez".into(),
                    phone: None,
                    email: None,
                    birth_date: Some("2000-01-15".into()),
                    notes: None,
                },
            )
            .expect("member");
        let plan = db
            .create_membership_plan(
                &organization_id,
                NewMembershipPlan {
                    name: "Matutino".into(),
                    description: None,
                    duration_days: 30,
                    price_minor: 40_000,
                    grace_days: 2,
                    access_days: "mon,tue,wed,thu,fri".into(),
                    access_start_minute: 360,
                    access_end_minute: 720,
                    min_age: Some(18),
                    max_age: Some(60),
                },
            )
            .expect("plan");
        assert_eq!(plan.version, 1);

        let membership = db
            .create_membership(&organization_id, &actor_id, &member.id, &plan.id, None)
            .expect("membership");
        assert_eq!(membership.plan_version, 1);
        assert_eq!(membership.access_days, "mon,tue,wed,thu,fri");
        assert_eq!(membership.price_minor, 40_000);

        let updated = db
            .update_membership_plan(
                &organization_id,
                &plan.id,
                UpdateMembershipPlan {
                    name: "Matutino".into(),
                    description: None,
                    duration_days: 30,
                    price_minor: 45_000,
                    grace_days: 2,
                    access_days: "mon,tue,wed,thu,fri,sat".into(),
                    access_start_minute: 360,
                    access_end_minute: 780,
                    min_age: Some(18),
                    max_age: Some(60),
                    active: true,
                },
            )
            .expect("updated plan");
        assert_eq!(updated.version, 2);
        assert_eq!(updated.price_minor, 45_000);

        let snapshot = db
            .membership(&organization_id, &membership.id)
            .expect("membership snapshot");
        assert_eq!(snapshot.plan_version, 1);
        assert_eq!(snapshot.price_minor, 40_000);
        assert_eq!(snapshot.access_days, "mon,tue,wed,thu,fri");

        let renewed = db
            .renew_membership(&organization_id, &actor_id, &membership.id, None)
            .expect("renewal");
        assert_eq!(renewed.plan_version, 2);
        assert_eq!(renewed.price_minor, 45_000);
        assert_eq!(renewed.access_days, "mon,tue,wed,thu,fri,sat");
        assert_eq!(renewed.access_end_minute, 780);

        let underage = db
            .create_member(
                &organization_id,
                NewMember {
                    first_name: "Menor".into(),
                    last_name: "Edad".into(),
                    phone: None,
                    email: None,
                    birth_date: Some("2015-01-15".into()),
                    notes: None,
                },
            )
            .expect("underage member");
        let underage_error = db
            .create_membership(&organization_id, &actor_id, &underage.id, &plan.id, None)
            .expect_err("underage membership");
        assert!(matches!(underage_error, DataError::InvalidValue(_)));

        let no_birth_date = db
            .create_member(
                &organization_id,
                NewMember {
                    first_name: "Sin".into(),
                    last_name: "Nacimiento".into(),
                    phone: None,
                    email: None,
                    birth_date: None,
                    notes: None,
                },
            )
            .expect("member without birth date");
        let missing_birth_date_error = db
            .create_membership(
                &organization_id,
                &actor_id,
                &no_birth_date.id,
                &plan.id,
                None,
            )
            .expect_err("membership without birth date");
        assert!(matches!(
            missing_birth_date_error,
            DataError::InvalidValue(_)
        ));

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn membership_payments_require_cash_and_feed_dashboard() {
        let (db, path) = test_db("payment-dashboard");
        let organization_id = setup(&db);
        let actor_id = owner_id(&db, &organization_id);

        let member = db
            .create_member(
                &organization_id,
                NewMember {
                    first_name: "Sofía".into(),
                    last_name: "Ramos".into(),
                    phone: None,
                    email: None,
                    birth_date: Some("1995-03-20".into()),
                    notes: None,
                },
            )
            .expect("member");
        let plan = db
            .create_membership_plan(
                &organization_id,
                NewMembershipPlan {
                    name: "Semanal".into(),
                    description: None,
                    duration_days: 5,
                    price_minor: 15_000,
                    grace_days: 0,
                    access_days: "mon,tue,wed,thu,fri,sat,sun".into(),
                    access_start_minute: 0,
                    access_end_minute: 1440,
                    min_age: None,
                    max_age: None,
                },
            )
            .expect("plan");
        let membership = db
            .create_membership(&organization_id, &actor_id, &member.id, &plan.id, None)
            .expect("membership");
        let charge_id = db
            .member_account(&organization_id, &member.id)
            .expect("account")
            .charges[0]
            .id
            .clone();

        let no_cash = db
            .record_membership_payment(
                &organization_id,
                &actor_id,
                NewMembershipPayment {
                    charge_id: charge_id.clone(),
                    amount_minor: 5_000,
                    payment_method: "cash".into(),
                    reference: None,
                    idempotency_key: "cash-without-session".into(),
                },
            )
            .expect_err("cash payment without session");
        assert!(matches!(no_cash, DataError::CashSessionNotOpen));

        let cash_session = db
            .open_cash_session(&organization_id, &actor_id, 0)
            .expect("cash session");
        let payment = db
            .record_membership_payment(
                &organization_id,
                &actor_id,
                NewMembershipPayment {
                    charge_id,
                    amount_minor: 5_000,
                    payment_method: "cash".into(),
                    reference: None,
                    idempotency_key: "cash-with-session".into(),
                },
            )
            .expect("cash payment");
        assert_eq!(payment.receipt_number, "REC-00001");
        assert_eq!(
            payment.cash_session_id.as_deref(),
            Some(cash_session.id.as_str())
        );

        let receipt = db
            .membership_payment_receipt(&organization_id, &payment.id)
            .expect("receipt");
        assert_eq!(receipt.receipt_number, "REC-00001");
        assert_eq!(receipt.member_number, member.member_number);
        assert_eq!(receipt.amount_minor, 5_000);
        assert_eq!(receipt.gym_name, "Test Gym");

        let dashboard = db
            .membership_dashboard(&organization_id)
            .expect("dashboard");
        assert_eq!(dashboard.income_this_month_minor, 5_000);
        assert_eq!(dashboard.recent_payments.len(), 1);
        assert_eq!(dashboard.recent_payments[0].payment_id, payment.id);
        assert_eq!(dashboard.expiring_soon, 1);
        assert_eq!(dashboard.expiring_memberships.len(), 1);
        assert_eq!(dashboard.expiring_memberships[0].id, membership.id);
        assert_eq!(dashboard.trend.len(), 6);
        assert_eq!(
            dashboard.trend.last().map(|point| point.income_minor),
            Some(5_000)
        );
        assert_eq!(dashboard.period_start.len(), 10);
        assert_eq!(dashboard.timezone, "UTC");

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn prospect_conversion_creates_member() {
        let (db, path) = test_db("prospect");
        let organization_id = setup(&db);
        let prospect = db
            .create_prospect(
                &organization_id,
                NewProspect {
                    name: "Carlos Ruiz".into(),
                    phone: Some("555-0202".into()),
                    email: None,
                    source: Some("Instagram".into()),
                    notes: None,
                },
            )
            .expect("prospect");
        let member = db
            .convert_prospect(&organization_id, &prospect.id)
            .expect("conversion");
        assert_eq!(member.first_name, "Carlos");
        assert_eq!(member.last_name, "Ruiz");
        let prospects = db
            .list_prospects(&organization_id, None)
            .expect("prospects");
        assert_eq!(prospects[0].status, "converted");
        assert_eq!(
            prospects[0].converted_member_id.as_deref(),
            Some(member.id.as_str())
        );
        let events = db
            .list_sync_outbox(&organization_id, 10)
            .expect("outbox")
            .into_iter()
            .map(|event| event.event_type)
            .collect::<Vec<_>>();
        assert!(events.contains(&"prospect.created".to_string()));
        assert!(events.contains(&"member.created".to_string()));
        assert!(events.contains(&"prospect.converted".to_string()));
        drop(db);
        let _ = std::fs::remove_file(path);
    }
}
