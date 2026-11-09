use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gym_sync::mtls::{MtlsSyncTransport, SyncClientConfig};
use gym_sync::{OutboundEvent, PushOutcome, SyncTransport};
use gym_sync_server::app::AppState;
use gym_sync_server::store::SyncStore;
use gym_sync_server::tls::load_server_tls;
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose, SanType,
};

struct Pki {
    directory: tempfile::TempDir,
    authority_path: PathBuf,
    server_certificate_path: PathBuf,
    server_key_path: PathBuf,
    client_certificate_path: PathBuf,
    client_key_path: PathBuf,
}

fn build_pki(organization_id: &str) -> Pki {
    let directory = tempfile::tempdir().expect("temp dir");

    let mut authority_params = CertificateParams::new(Vec::new()).expect("authority params");
    authority_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    authority_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    authority_params
        .distinguished_name
        .push(DnType::CommonName, "Gym Sync Test Authority");
    let authority_key = KeyPair::generate().expect("authority key");
    let authority = authority_params
        .self_signed(&authority_key)
        .expect("authority certificate");

    let mut server_params = CertificateParams::new(Vec::new()).expect("server params");
    server_params
        .distinguished_name
        .push(DnType::CommonName, "sync.test");
    server_params
        .subject_alt_names
        .push(SanType::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST)));
    server_params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);
    let server_key = KeyPair::generate().expect("server key");
    let server_certificate = server_params
        .signed_by(&server_key, &authority, &authority_key)
        .expect("server certificate");

    let mut client_params = CertificateParams::new(Vec::new()).expect("client params");
    client_params
        .distinguished_name
        .push(DnType::CommonName, organization_id);
    client_params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ClientAuth);
    let client_key = KeyPair::generate().expect("client key");
    let client_certificate = client_params
        .signed_by(&client_key, &authority, &authority_key)
        .expect("client certificate");

    let authority_path = directory.path().join("authority.pem");
    let server_certificate_path = directory.path().join("server.pem");
    let server_key_path = directory.path().join("server.key");
    let client_certificate_path = directory.path().join("client.pem");
    let client_key_path = directory.path().join("client.key");
    std::fs::write(&authority_path, authority.pem()).expect("writes authority");
    std::fs::write(&server_certificate_path, server_certificate.pem()).expect("writes server cert");
    std::fs::write(&server_key_path, server_key.serialize_pem()).expect("writes server key");
    std::fs::write(&client_certificate_path, client_certificate.pem()).expect("writes client cert");
    std::fs::write(&client_key_path, client_key.serialize_pem()).expect("writes client key");

    Pki {
        directory,
        authority_path,
        server_certificate_path,
        server_key_path,
        client_certificate_path,
        client_key_path,
    }
}

fn event(idempotency_key: &str, payload: serde_json::Value) -> OutboundEvent {
    OutboundEvent {
        event_id: format!("event-{idempotency_key}"),
        event_type: "member.updated".into(),
        aggregate_type: "member".into(),
        aggregate_id: "member-1".into(),
        idempotency_key: idempotency_key.into(),
        payload,
        attempts: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

#[test]
fn pushes_and_pulls_events_over_mutual_tls() {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let _guard = runtime.enter();

    let organization_id = "org-1";
    let pki = build_pki(organization_id);
    let acceptor = load_server_tls(
        &pki.server_certificate_path,
        &pki.server_key_path,
        &pki.authority_path,
    )
    .expect("loads tls material");
    let store = SyncStore::open(&pki.directory.path().join("sync.db")).expect("opens store");
    let router = gym_sync_server::router(AppState {
        store: Arc::new(Mutex::new(store)),
    });
    let listener = runtime
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .expect("binds listener");
    let address = listener.local_addr().expect("reads address");
    let (_sender, receiver) = tokio::sync::watch::channel(false);
    runtime.spawn(async move {
        gym_sync_server::server::serve(listener, acceptor, router, receiver).await;
    });

    let mut transport = MtlsSyncTransport::new(&SyncClientConfig {
        base_url: format!("https://127.0.0.1:{}", address.port()),
        client_cert_path: pki.client_certificate_path.clone(),
        client_key_path: pki.client_key_path.clone(),
        ca_cert_path: pki.authority_path.clone(),
        timeout: Duration::from_secs(10),
    })
    .expect("builds transport");

    let events = vec![
        event("key-1", serde_json::json!({ "name": "Ana" })),
        event("key-2", serde_json::json!({ "name": "Luis" })),
    ];
    let acks = transport
        .push(organization_id, &events)
        .expect("pushes events");
    assert_eq!(acks.len(), 2);
    assert!(acks
        .iter()
        .all(|ack| matches!(ack.outcome, PushOutcome::Accepted)));

    let pulled = transport
        .pull(organization_id, 0, 50)
        .expect("pulls events");
    assert_eq!(pulled.len(), 2);
    assert_eq!(pulled[0].idempotency_key, "key-1");
    assert_eq!(pulled[0].server_sequence, 1);
    assert_eq!(pulled[1].server_sequence, 2);
    assert_eq!(pulled[0].payload["name"], "Ana");

    let repeated = transport
        .push(organization_id, &events)
        .expect("repeats push");
    assert!(repeated
        .iter()
        .all(|ack| matches!(ack.outcome, PushOutcome::Accepted)));
    let pulled_again = transport
        .pull(organization_id, 0, 50)
        .expect("pulls events again");
    assert_eq!(pulled_again.len(), 2);

    let after_cursor = transport
        .pull(organization_id, 2, 50)
        .expect("pulls from cursor");
    assert!(after_cursor.is_empty());
}

#[test]
fn rejects_clients_without_a_certificate() {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let _guard = runtime.enter();

    let pki = build_pki("org-1");
    let acceptor = load_server_tls(
        &pki.server_certificate_path,
        &pki.server_key_path,
        &pki.authority_path,
    )
    .expect("loads tls material");
    let store = SyncStore::open(&pki.directory.path().join("sync.db")).expect("opens store");
    let router = gym_sync_server::router(AppState {
        store: Arc::new(Mutex::new(store)),
    });
    let listener = runtime
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .expect("binds listener");
    let address = listener.local_addr().expect("reads address");
    let (_sender, receiver) = tokio::sync::watch::channel(false);
    runtime.spawn(async move {
        gym_sync_server::server::serve(listener, acceptor, router, receiver).await;
    });

    let anonymous = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("builds client");
    let response = anonymous
        .get(format!("https://127.0.0.1:{}/v1/health", address.port()))
        .send();
    assert!(
        response.is_err(),
        "the server must refuse anonymous clients"
    );
}
