/// Ordered, forward-only migrations. Each version is applied once inside a
/// transaction; `down` documents the manual rollback path for runbooks.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: &'static str,
    pub down: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_core_schema",
        up: include_str!("../migrations/0001_initial_core_schema.sql"),
        down: "DROP TABLE IF EXISTS module_events; \
               DROP TABLE IF EXISTS audit_events; \
               DROP TABLE IF EXISTS backup_snapshots; \
               DROP TABLE IF EXISTS staff_memberships; \
               DROP TABLE IF EXISTS sites; \
               DROP TABLE IF EXISTS organizations;",
    },
    Migration {
        version: 2,
        name: "local_users",
        up: include_str!("../migrations/0002_local_users.sql"),
        down: "DROP INDEX IF EXISTS idx_staff_user; \
               DROP INDEX IF EXISTS idx_staff_one_active_owner; \
               DROP INDEX IF EXISTS idx_users_organization; \
               DROP INDEX IF EXISTS idx_users_organization_name; \
               DROP TABLE IF EXISTS users;",
    },
    Migration {
        version: 3,
        name: "sites_staff",
        up: include_str!("../migrations/0003_sites_staff.sql"),
        down: "DROP INDEX IF EXISTS idx_staff_one_membership_per_user; \
               DROP INDEX IF EXISTS idx_sites_organization_name;",
    },
];
