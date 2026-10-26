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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMembershipPlan {
    pub name: String,
    pub description: Option<String>,
    pub duration_days: i64,
    pub price_minor: i64,
    pub grace_days: i64,
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
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub price_minor: i64,
    pub currency: String,
    pub grace_days: i64,
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
    pub received_by: Option<String>,
    pub received_by_name: Option<String>,
    pub created_at: String,
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
    pub active_members: i64,
    pub active_memberships: i64,
    pub expiring_soon: i64,
    pub past_due: i64,
    pub debt_minor: i64,
    pub new_members_this_month: i64,
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
                        grace_days, active, created_at, updated_at
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
                      price_minor, currency, grace_days, active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    id,
                    organization_id,
                    plan.name,
                    plan.description,
                    plan.duration_days,
                    plan.price_minor,
                    currency,
                    plan.grace_days,
                    i64::from(plan.active),
                    now
                ],
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
            let changed = transaction.execute(
                "UPDATE membership_plans
                 SET name = ?1, description = ?2, duration_days = ?3,
                     price_minor = ?4, grace_days = ?5, active = ?6, updated_at = ?7
                 WHERE organization_id = ?8 AND id = ?9",
                params![
                    plan.name,
                    plan.description,
                    plan.duration_days,
                    plan.price_minor,
                    plan.grace_days,
                    i64::from(plan.active),
                    now,
                    organization_id,
                    plan_id
                ],
            )?;
            if changed == 0 {
                return Err(DataError::MembershipPlanNotFound);
            }
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
                        m.plan_id, m.plan_name, m.status, m.start_date, m.end_date,
                        m.price_minor, m.currency, m.grace_days, m.paused_at,
                        m.pause_until, m.cancellation_reason, m.created_at, m.updated_at
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
                     (id, organization_id, member_id, plan_id, plan_name, status,
                      start_date, end_date, price_minor, currency, grace_days,
                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
                params![
                    membership_id,
                    organization_id,
                    member_id,
                    plan.id,
                    plan.name,
                    status,
                    date_string(start),
                    date_string(end),
                    plan.price_minor,
                    plan.currency,
                    plan.grace_days,
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
            let end = period_start + chrono::TimeDelta::days(plan.duration_days - 1);
            let membership_start = parse_date(&membership.start_date, "membership start date")?;
            let status = if membership_start > today() {
                "scheduled"
            } else {
                "active"
            };
            transaction.execute(
                "UPDATE memberships
                 SET plan_id = ?1, plan_name = ?2, status = ?3, end_date = ?4,
                     price_minor = ?5, currency = ?6, grace_days = ?7,
                     paused_at = NULL, pause_until = NULL,
                     cancellation_reason = NULL, updated_at = ?8
                 WHERE organization_id = ?9 AND id = ?10",
                params![
                    plan.id,
                    plan.name,
                    status,
                    date_string(end),
                    plan.price_minor,
                    plan.currency,
                    plan.grace_days,
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
                transaction.execute(
                    "INSERT INTO membership_payments
                         (id, organization_id, charge_id, membership_id, member_id,
                          amount_minor, payment_method, reference, idempotency_key,
                          request_hash, received_by, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        payment_id,
                        organization_id,
                        charge.id,
                        charge.membership_id,
                        charge.member_id,
                        payment.amount_minor,
                        payment.payment_method,
                        reference,
                        idempotency_key,
                        request_hash,
                        received_by,
                        now
                    ],
                )?;
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
                        p.payment_method, p.reference, p.received_by, u.name, p.created_at
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

    pub fn membership_dashboard(
        &self,
        organization_id: &str,
    ) -> Result<MembershipDashboardRecord, DataError> {
        self.refresh_memberships(organization_id)?;
        self.with_connection(|conn| {
            let today = today();
            let month_start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .ok_or_else(|| DataError::InvalidValue("invalid current date".into()))?;
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
                 WHERE organization_id = ?1 AND joined_at >= ?2",
                params![organization_id, date_string(month_start)],
                |row| row.get(0),
            )?;
            Ok(MembershipDashboardRecord {
                active_members,
                active_memberships,
                expiring_soon,
                past_due,
                debt_minor,
                new_members_this_month,
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

fn validate_membership_plan(plan: NewMembershipPlan) -> Result<ValidatedMembershipPlan, DataError> {
    Ok(ValidatedMembershipPlan {
        name: required_text(&plan.name, "plan name", 2, 80)?,
        description: normalize_optional_text(plan.description, 300, "description")?,
        duration_days: validate_duration(plan.duration_days)?,
        price_minor: validate_price(plan.price_minor)?,
        grace_days: validate_grace_days(plan.grace_days)?,
        active: true,
    })
}

fn validate_membership_plan_update(
    plan: UpdateMembershipPlan,
) -> Result<ValidatedMembershipPlan, DataError> {
    Ok(ValidatedMembershipPlan {
        name: required_text(&plan.name, "plan name", 2, 80)?,
        description: normalize_optional_text(plan.description, 300, "description")?,
        duration_days: validate_duration(plan.duration_days)?,
        price_minor: validate_price(plan.price_minor)?,
        grace_days: validate_grace_days(plan.grace_days)?,
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

fn load_membership_plan(
    conn: &Connection,
    organization_id: &str,
    plan_id: &str,
) -> Result<MembershipPlanRecord, DataError> {
    conn.query_row(
        "SELECT id, name, description, duration_days, price_minor, currency,
                grace_days, active, created_at, updated_at
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
        active: row.get::<_, i64>(7)? != 0,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
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
                    m.plan_id, m.plan_name, m.status, m.start_date, m.end_date,
                    m.price_minor, m.currency, m.grace_days, m.paused_at,
                    m.pause_until, m.cancellation_reason, m.created_at, m.updated_at
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
        status: row.get(5)?,
        start_date: row.get(6)?,
        end_date: row.get(7)?,
        price_minor: row.get(8)?,
        currency: row.get(9)?,
        grace_days: row.get(10)?,
        paused_at: row.get(11)?,
        pause_until: row.get(12)?,
        cancellation_reason: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

fn load_membership(
    conn: &Connection,
    organization_id: &str,
    membership_id: &str,
) -> Result<MembershipRecord, DataError> {
    conn.query_row(
        "SELECT m.id, m.member_id, TRIM(mem.first_name || ' ' || mem.last_name),
                m.plan_id, m.plan_name, m.status, m.start_date, m.end_date,
                m.price_minor, m.currency, m.grace_days, m.paused_at,
                m.pause_until, m.cancellation_reason, m.created_at, m.updated_at
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
        received_by: row.get(7)?,
        received_by_name: row.get(8)?,
        created_at: row.get(9)?,
    })
}

fn load_member_payments(
    conn: &Connection,
    organization_id: &str,
    member_id: &str,
) -> Result<Vec<MembershipPaymentRecord>, DataError> {
    let mut statement = conn.prepare(
        "SELECT p.id, p.charge_id, p.membership_id, p.member_id, p.amount_minor,
                p.payment_method, p.reference, p.received_by, u.name, p.created_at
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
    fn schema_17_is_applied() {
        let (db, path) = test_db("schema");
        let status = db.status().expect("status");
        assert_eq!(status.schema_version, MIGRATIONS.last().unwrap().version);
        assert_eq!(status.schema_version, 17);
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
