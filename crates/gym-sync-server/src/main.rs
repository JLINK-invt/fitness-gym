use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gym_sync_server::app::AppState;
use gym_sync_server::store::SyncStore;
use gym_sync_server::tls::load_server_tls;
use tracing_subscriber::EnvFilter;

fn required_env(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("falta la variable de entorno {name}"))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let listen: SocketAddr = std::env::var("GYM_SYNC_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:8443".to_string())
        .parse()
        .map_err(|error| format!("GYM_SYNC_LISTEN no es una dirección válida: {error}"))?;
    let database_path =
        PathBuf::from(std::env::var("GYM_SYNC_DB").unwrap_or_else(|_| "gym-sync.db".to_string()));
    let certificate_path = PathBuf::from(required_env("GYM_SYNC_CERT")?);
    let private_key_path = PathBuf::from(required_env("GYM_SYNC_KEY")?);
    let client_authority_path = PathBuf::from(required_env("GYM_SYNC_CLIENT_CA")?);

    let acceptor = load_server_tls(&certificate_path, &private_key_path, &client_authority_path)
        .map_err(|error| error.to_string())?;
    let store = SyncStore::open(&database_path).map_err(|error| error.to_string())?;
    let state = AppState {
        store: Arc::new(Mutex::new(store)),
    };
    let router = gym_sync_server::router(state);

    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .map_err(|error| format!("no se pudo escuchar en {listen}: {error}"))?;
    tracing::info!(%listen, "el servidor de sincronización está escuchando");

    let (sender, receiver) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = sender.send(true);
        }
    });

    gym_sync_server::server::serve(listener, acceptor, router, receiver).await;
    Ok(())
}
