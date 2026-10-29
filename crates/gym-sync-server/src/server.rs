use std::net::SocketAddr;

use axum::Router;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower::ServiceExt;

use crate::app::PeerCertificate;

/// Accepts connections and serves them over mutual TLS.
///
/// The peer certificate is injected in the request extensions so the handlers
/// can scope every operation to the verified organization.
pub async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    router: Router,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, peer_address) = match accepted {
                    Ok(value) => value,
                    Err(error) => {
                        tracing::warn!(%error, "no se pudo aceptar la conexión");
                        continue;
                    }
                };
                let acceptor = acceptor.clone();
                let router = router.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(stream, acceptor, router, peer_address).await {
                        tracing::debug!(%error, %peer_address, "la conexión se cerró con error");
                    }
                });
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    tracing::info!("el servidor de sincronización se está apagando");
                    return;
                }
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    acceptor: TlsAcceptor,
    router: Router,
    _peer_address: SocketAddr,
) -> Result<(), String> {
    let tls_stream = acceptor
        .accept(stream)
        .await
        .map_err(|error| format!("el handshake mTLS falló: {error}"))?;
    let peer_certificates = tls_stream
        .get_ref()
        .1
        .peer_certificates()
        .map(|certificates| {
            certificates
                .iter()
                .map(|certificate| certificate.to_vec())
                .collect::<Vec<Vec<u8>>>()
        });

    let service = service_fn(move |mut request: Request<Incoming>| {
        let router = router.clone();
        let peer_certificates = peer_certificates.clone();
        async move {
            if let Some(certificates) = peer_certificates {
                if let Some(der) = certificates.first() {
                    request
                        .extensions_mut()
                        .insert(PeerCertificate { der: der.clone() });
                }
            }
            let request = request.map(axum::body::Body::new);
            let response = router.oneshot(request).await?;
            Ok::<_, std::convert::Infallible>(response)
        }
    });

    hyper::server::conn::http1::Builder::new()
        .serve_connection(TokioIo::new(tls_stream), service)
        .await
        .map_err(|error| format!("la conexión falló: {error}"))?;

    Ok(())
}
