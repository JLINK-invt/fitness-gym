/// Ordered, forward-only migrations. Each version is applied once inside a
/// transaction; `down` documents the manual rollback path for runbooks.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: &'static str,
    pub down: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial_core_schema",
    up: include_str!("../migrations/0001_initial_core_schema.sql"),
    down: "DROP TABLE IF EXISTS module_events; \
           DROP TABLE IF EXISTS audit_events; \
           DROP TABLE IF EXISTS backup_snapshots; \
           DROP TABLE IF EXISTS staff_memberships; \
           DROP TABLE IF EXISTS sites; \
           DROP TABLE IF EXISTS organizations;",
}];
