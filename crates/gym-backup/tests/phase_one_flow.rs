use std::fs;

use gym_backup::{create_backup, list_backups, restore_backup};
use gym_data::{Db, DbKey};

#[test]
fn local_flow_restores_without_duplicating_events() {
    let dir = tempfile::tempdir().expect("temp dir");
    let data_dir = dir.path().join("data");
    fs::create_dir_all(&data_dir).expect("data dir");
    let database_path = data_dir.join("fitness-gym.db");
    let backups_dir = data_dir.join("backups");
    let key = DbKey::generate();

    let db = Db::open(&database_path, &key).expect("opens encrypted database");
    let organization_id = db
        .create_organization("Gimnasio Centro")
        .expect("configures gym");

    let first = db
        .record_module_event(
            &organization_id,
            "outbound",
            "access.allowed",
            "access_decision",
            "req-1",
            "hash",
        )
        .expect("records first access");
    assert!(first.inserted);

    let duplicate = db
        .record_module_event(
            &organization_id,
            "outbound",
            "access.allowed",
            "access_decision",
            "req-1",
            "hash",
        )
        .expect("replays first access");
    assert!(!duplicate.inserted);
    assert_eq!(first.event.id, duplicate.event.id);
    assert_eq!(
        db.list_module_events("access_decision", 20)
            .expect("lists events")
            .len(),
        1
    );

    create_backup(&db, &backups_dir, "clave-segura-123").expect("creates backup");

    db.record_module_event(
        &organization_id,
        "outbound",
        "access.denied",
        "access_decision",
        "req-2",
        "other-hash",
    )
    .expect("records second access");
    assert_eq!(
        db.list_module_events("access_decision", 20)
            .expect("lists events")
            .len(),
        2
    );

    db.close().expect("closes database");

    let backup = list_backups(&backups_dir)
        .expect("lists backups")
        .into_iter()
        .next()
        .expect("backup exists");
    let outcome = restore_backup(
        &backups_dir.join(&backup.file_name),
        &database_path,
        "clave-segura-123",
        &key,
    )
    .expect("restores backup");
    assert!(outcome.integrity_ok);

    db.reopen(&key).expect("reopens restored database");
    let events = db
        .list_module_events("access_decision", 20)
        .expect("lists restored events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, first.event.id);
    assert_eq!(
        db.organization()
            .expect("reads organization")
            .expect("organization persists")
            .id,
        organization_id
    );
    assert!(!fs::read(&database_path)
        .expect("reads encrypted database")
        .starts_with(b"SQLite format 3"));
}
