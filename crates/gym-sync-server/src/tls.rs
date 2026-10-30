use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use tokio_rustls::TlsAcceptor;
use x509_parser::prelude::{FromDer, X509Certificate};

use crate::error::ServerError;

/// Loads the server certificate, its private key and the authority that signs
/// the client certificates. The server only accepts clients that present a
/// certificate issued by that authority.
pub fn load_server_tls(
    certificate_path: &Path,
    private_key_path: &Path,
    client_authority_path: &Path,
) -> Result<TlsAcceptor, ServerError> {
    let certificates = rustls_pemfile::certs(&mut BufReader::new(File::open(certificate_path)?))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ServerError::InvalidMaterial(error.to_string()))?;
    if certificates.is_empty() {
        return Err(ServerError::InvalidMaterial(
            "the server certificate file has no certificates".into(),
        ));
    }
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(File::open(private_key_path)?))
            .map_err(|error| ServerError::InvalidMaterial(error.to_string()))?
            .ok_or_else(|| {
                ServerError::InvalidMaterial("the private key file has no key".into())
            })?;

    let mut roots = RootCertStore::empty();
    for certificate in
        rustls_pemfile::certs(&mut BufReader::new(File::open(client_authority_path)?))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| ServerError::InvalidMaterial(error.to_string()))?
    {
        roots.add(certificate).map_err(|error| {
            ServerError::InvalidMaterial(format!("the client authority is invalid: {error}"))
        })?;
    }

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .map_err(|error| ServerError::InvalidMaterial(error.to_string()))?;
    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| ServerError::InvalidMaterial(error.to_string()))?
        .with_client_cert_verifier(verifier)
        .with_single_cert(certificates, private_key)
        .map_err(|error| ServerError::InvalidMaterial(error.to_string()))?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Reads the common name of a DER encoded certificate.
pub fn common_name(der: &[u8]) -> Option<String> {
    let (_, certificate) = X509Certificate::from_der(der).ok()?;
    let name = certificate
        .subject()
        .iter_common_name()
        .next()?
        .as_str()
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    name
}
