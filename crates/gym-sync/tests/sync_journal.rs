use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use gym_data::{
    Db, DbKey, InboundSyncEvent, NewCashMovement, NewCatalogItem, NewCatalogPrice,
    NewInventoryAdjustment, NewSale, NewSaleItem, NewSaleReturn, NewSaleReturnItem,
    SyncOutboxRecord,
};
use gym_sync::{InboundEvent, OutboundEvent, PushAck, PushOutcome, SyncEngine, SyncTransport};
use tempfile::TempDir;

#[derive(Default)]
struct FakeServer {
    accepted: BTreeMap<String, OutboundEvent>,
    inbox: Vec<InboundEvent>,
    fail_after_record: bool,
    fail_before_record: bool,
    push_calls: usize,
}

impl SyncTransport for FakeServer {
    fn push(
        &mut self,
        _organization_id: &str,
        events: &[OutboundEvent],
    ) -> Result<Vec<PushAck>, String> {
        self.push_calls += 1;
        if self.fail_before_record {
            return Err("server unavailable".into());
        }
        let acks = events
            .iter()
            .map(|event| {
                self.accepted
                    .entry(event.idempotency_key.clone())
                    .or_insert_with(|| event.clone());
                PushAck {
                    idempotency_key: event.idempotency_key.clone(),
                    outcome: PushOutcome::Accepted,
                }
            })
            .collect();
        if self.fail_after_record {
            self.fail_after_record = false;
            return Err("ack lost".into());
        }
        Ok(acks)
    }

    fn pull(
        &mut self,
        _organization_id: &str,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<InboundEvent>, String> {
        Ok(self
            .inbox
            .iter()
            .filter(|event| event.server_sequence > after_sequence)
            .take(limit)
            .cloned()
            .collect())
    }
}

struct Fixture {
    db: Db,
    _dir: TempDir,
    organization_id: String,
    owner_id: String,
    tracked_variant_id: String,
    sale_id: String,
    sale_item_id: String,
    cash_session_id: String,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("temp dir");
    let db = Db::open(dir.path().join("gym.db"), &DbKey::generate()).expect("opens database");
    let organization_id = db
        .create_organization("Gimnasio Sync")
        .expect("creates organization");
    let owner = db
        .create_owner(&organization_id, "Ana Sync", "$argon2id$owner", None)
        .expect("creates owner");
    let tracked = db
        .create_catalog_item(
            &organization_id,
            &owner.id,
            NewCatalogItem {
                name: "Playera".into(),
                category: None,
                variant_name: "M".into(),
                sku: "PLAY-M".into(),
                barcode: None,
                unit: "piece".into(),
                cost_minor: 4_000,
                price: NewCatalogPrice {
                    amount_minor: 10_000,
                    tax_rate_bps: 1_600,
                    price_includes_tax: true,
                    effective_from: Some("2020-01-01T00:00:00Z".into()),
                },
            },
        )
        .expect("creates tracked product");
    let service = db
        .create_catalog_item(
            &organization_id,
            &owner.id,
            NewCatalogItem {
                name: "Visita".into(),
                category: None,
                variant_name: "Día".into(),
                sku: "VIS-DIA".into(),
                barcode: None,
                unit: "service".into(),
                cost_minor: 0,
                price: NewCatalogPrice {
                    amount_minor: 5_000,
                    tax_rate_bps: 1_600,
                    price_includes_tax: true,
                    effective_from: Some("2020-01-01T00:00:00Z".into()),
                },
            },
        )
        .expect("creates service");
    let tracked_variant_id = tracked.variants[0].id.clone();
    db.record_inventory_entry(
        &organization_id,
        &owner.id,
        gym_data::NewInventoryEntry {
            variant_id: tracked_variant_id.clone(),
            quantity: 5,
            unit_cost_minor: 4_000,
            reference: None,
            note: None,
        },
    )
    .expect("records stock");
    let cash_session = db
        .open_cash_session(&organization_id, &owner.id, 5_000)
        .expect("opens cash");
    let sale = db
        .create_sale(
            &organization_id,
            &owner.id,
            NewSale {
                items: vec![
                    NewSaleItem {
                        variant_id: tracked_variant_id.clone(),
                        quantity: 2,
                    },
                    NewSaleItem {
                        variant_id: service.variants[0].id.clone(),
                        quantity: 1,
                    },
                ],
                payment_method: "cash".into(),
                cash_received_minor: Some(30_000),
                payment_reference: None,
                idempotency_key: "sale-1".into(),
            },
        )
        .expect("creates sale");
    Fixture {
        db,
        _dir: dir,
        organization_id,
        owner_id: owner.id,
        tracked_variant_id,
        sale_id: sale.id.clone(),
        sale_item_id: sale.items[0].id.clone(),
        cash_session_id: cash_session.id,
    }
}

fn pending(db: &Db, organization_id: &str) -> Vec<SyncOutboxRecord> {
    db.pending_sync_events(
        organization_id,
        &(Utc::now() + Duration::days(1)).to_rfc3339(),
        100,
    )
    .expect("lists pending events")
}

#[test]
fn sale_and_movements_are_journaled_and_pushed_once() {
    let fixture = fixture();
    let events = pending(&fixture.db, &fixture.organization_id);
    let types = events
        .iter()
        .map(|event| event.event_type.as_str())
        .collect::<Vec<_>>();
    assert!(types.contains(&"cash.session_opened"));
    assert!(types.contains(&"inventory.entry_recorded"));
    assert!(types.contains(&"sale.created"));

    let sale_event = events
        .iter()
        .find(|event| event.event_type == "sale.created")
        .expect("sale event");
    let payload: serde_json::Value =
        serde_json::from_str(&sale_event.payload_json).expect("sale payload json");
    assert_eq!(payload["saleId"], fixture.sale_id);
    assert_eq!(payload["totalMinor"], 25_000);
    assert_eq!(payload["items"].as_array().expect("items").len(), 2);

    let mut server = FakeServer::default();
    let report = SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, Utc::now())
        .expect("syncs");
    assert_eq!(report.pushed, events.len());
    assert_eq!(report.accepted, events.len());
    assert_eq!(report.retried, 0);
    assert!(report.transport_error.is_none());
    assert_eq!(server.accepted.len(), events.len());
    assert!(pending(&fixture.db, &fixture.organization_id).is_empty());

    let second = SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, Utc::now())
        .expect("re-syncs");
    assert_eq!(second.pushed, 0);
    assert!(fixture
        .db
        .list_sync_outbox(&fixture.organization_id, 100)
        .expect("lists outbox")
        .iter()
        .all(|event| event.state == "sent"));
}

#[test]
fn a_lost_ack_retries_with_the_same_idempotency_key() {
    let fixture = fixture();
    let events = pending(&fixture.db, &fixture.organization_id);
    let mut server = FakeServer {
        fail_after_record: true,
        ..FakeServer::default()
    };

    let now = Utc::now();
    let first = SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, now)
        .expect("first sync keeps retrying");
    assert_eq!(first.retried, events.len());
    assert_eq!(first.transport_error.as_deref(), Some("ack lost"));
    assert_eq!(server.accepted.len(), events.len());

    let retried = fixture
        .db
        .list_sync_outbox(&fixture.organization_id, 100)
        .expect("lists outbox");
    assert!(retried.iter().all(|event| event.state == "pending"));
    assert!(retried.iter().all(|event| event.attempts == 1));
    assert!(fixture
        .db
        .pending_sync_events(
            &fixture.organization_id,
            &(now + Duration::seconds(1)).to_rfc3339(),
            100,
        )
        .expect("lists pending")
        .is_empty());

    let second = SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, now + Duration::seconds(31))
        .expect("second sync acknowledges");
    assert_eq!(second.accepted, events.len());
    assert_eq!(server.accepted.len(), events.len());
    assert!(fixture
        .db
        .list_sync_outbox(&fixture.organization_id, 100)
        .expect("lists outbox")
        .iter()
        .all(|event| event.state == "sent"));
}

#[test]
fn exhausted_retries_move_events_to_dead_letter() {
    let fixture = fixture();
    let events = pending(&fixture.db, &fixture.organization_id);
    let mut server = FakeServer {
        fail_before_record: true,
        ..FakeServer::default()
    };
    let report = SyncEngine::new(&fixture.db, &mut server)
        .with_max_attempts(1)
        .sync_once(&fixture.organization_id, Utc::now())
        .expect("syncs");
    assert_eq!(report.dead_lettered, events.len());
    assert_eq!(report.retried, 0);

    let outbox = fixture
        .db
        .list_sync_outbox(&fixture.organization_id, 100)
        .expect("lists outbox");
    assert!(outbox.iter().all(|event| event.state == "dead_letter"));
    assert!(outbox.iter().all(|event| event.attempts == 1));
    assert!(outbox
        .iter()
        .all(|event| event.last_error.as_deref() == Some("server unavailable")));
    assert!(pending(&fixture.db, &fixture.organization_id).is_empty());
}

#[test]
fn returns_and_manual_movements_are_journaled() {
    let fixture = fixture();
    fixture
        .db
        .create_sale_return(
            &fixture.organization_id,
            &fixture.owner_id,
            NewSaleReturn {
                sale_id: fixture.sale_id.clone(),
                reason: "Cliente canceló".into(),
                items: vec![NewSaleReturnItem {
                    sale_item_id: fixture.sale_item_id.clone(),
                    quantity: 1,
                }],
                payment_reference: None,
                idempotency_key: "return-1".into(),
            },
        )
        .expect("creates return");
    fixture
        .db
        .record_cash_movement(
            &fixture.organization_id,
            &fixture.owner_id,
            NewCashMovement {
                cash_session_id: fixture.cash_session_id.clone(),
                movement_type: "expense".into(),
                category: "Limpieza".into(),
                amount_minor: 2_000,
                payment_method: "cash".into(),
                evidence: "Ticket de limpieza".into(),
            },
        )
        .expect("records cash movement");
    fixture
        .db
        .adjust_inventory(
            &fixture.organization_id,
            &fixture.owner_id,
            NewInventoryAdjustment {
                variant_id: fixture.tracked_variant_id.clone(),
                quantity_delta: -1,
                unit_cost_minor: Some(4_000),
                reason: "Producto dañado".into(),
            },
        )
        .expect("adjusts inventory");

    let types = pending(&fixture.db, &fixture.organization_id)
        .into_iter()
        .map(|event| event.event_type)
        .collect::<Vec<_>>();
    assert!(types.contains(&"sale.returned".to_string()));
    assert!(types.contains(&"cash.movement_recorded".to_string()));
    assert!(types.contains(&"inventory.adjustment_recorded".to_string()));
}

#[test]
fn inbox_is_idempotent_and_advances_the_server_cursor() {
    let fixture = fixture();
    let mut server = FakeServer::default();
    SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, Utc::now())
        .expect("drains outbox");
    let cursor = fixture
        .db
        .sync_cursor(&fixture.organization_id)
        .expect("cursor");
    assert_eq!(cursor, 0);

    server.inbox.push(InboundEvent {
        event_id: "remote-1".into(),
        server_sequence: 7,
        event_type: "sale.created".into(),
        aggregate_type: "sale".into(),
        aggregate_id: "sale-remote".into(),
        idempotency_key: "remote-sale-1".into(),
        payload: serde_json::json!({ "saleId": "sale-remote", "totalMinor": 12_500 }),
        occurred_at: "2026-01-01T10:00:00Z".into(),
    });
    let report = SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, Utc::now())
        .expect("pulls inbox");
    assert_eq!(report.pulled, 1);
    assert_eq!(report.applied, 1);
    assert_eq!(report.duplicates, 0);
    assert_eq!(
        fixture
            .db
            .sync_cursor(&fixture.organization_id)
            .expect("cursor"),
        7
    );

    let duplicate = fixture
        .db
        .apply_inbox_event(
            &fixture.organization_id,
            InboundSyncEvent {
                event_id: "remote-1".into(),
                server_sequence: 7,
                event_type: "sale.created".into(),
                aggregate_type: "sale".into(),
                aggregate_id: "sale-remote".into(),
                idempotency_key: "remote-sale-1".into(),
                payload_json: "{}".into(),
                occurred_at: "2026-01-01T10:00:00Z".into(),
            },
        )
        .expect("replays inbox");
    assert!(!duplicate);
    assert_eq!(
        fixture
            .db
            .list_sync_inbox(&fixture.organization_id, 10)
            .expect("lists inbox")
            .len(),
        1
    );

    let next = SyncEngine::new(&fixture.db, &mut server)
        .sync_once(&fixture.organization_id, Utc::now())
        .expect("pulls again");
    assert_eq!(next.pulled, 0);
}
