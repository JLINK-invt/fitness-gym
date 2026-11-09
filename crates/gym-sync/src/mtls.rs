use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::wire::{PullResponse, PushRequest, PushResponse};
use crate::{InboundEvent, OutboundEvent, PushAck, SyncTransport};

/// Connection settings for the mutual-TLS transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncClientConfig {
    pub base_url: String,
    pub client_cert_path: PathBuf,
    pub client_key_path: PathBuf,
    pub ca_cert_path: PathBuf,
    pub timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("could not read the TLS material: {0}")]
    Io(#[from] std::io::Error),

    #[error("the TLS material is invalid: {0}")]
    InvalidMaterial(String),

    #[error("the sync request failed: {0}")]
    Request(String),

    #[error("the sync response could not be parsed: {0}")]
    Response(String),
}

/// Transport that talks to the sync server over mutual TLS.
///
/// The client certificate identifies the organization: its common name must
/// match the organization id, and the server rejects anything else.
pub struct MtlsSyncTransport {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl MtlsSyncTransport {
    pub fn new(config: &SyncClientConfig) -> Result<Self, TransportError> {
        let base_url = config.base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Err(TransportError::InvalidMaterial(
                "the sync server url is empty".into(),
            ));
        }
        if !base_url.starts_with("https://") {
            return Err(TransportError::InvalidMaterial(
                "the sync server url must use https".into(),
            ));
        }

        let certificate = fs::read(&config.client_cert_path)?;
        let key = fs::read(&config.client_key_path)?;
        let mut identity_pem = certificate;
        identity_pem.push(b'\n');
        identity_pem.extend_from_slice(&key);
        let identity = reqwest::Identity::from_pem(&identity_pem).map_err(|error| {
            TransportError::InvalidMaterial(format!("the client certificate is invalid: {error}"))
        })?;

        let authority = fs::read(&config.ca_cert_path)?;
        let authority = reqwest::Certificate::from_pem(&authority).map_err(|error| {
            TransportError::InvalidMaterial(format!(
                "the authority certificate is invalid: {error}"
            ))
        })?;

        let client = reqwest::blocking::Client::builder()
            .identity(identity)
            .add_root_certificate(authority)
            .timeout(config.timeout)
            .build()
            .map_err(|error| TransportError::InvalidMaterial(error.to_string()))?;

        Ok(Self { client, base_url })
    }

    fn check(
        response: reqwest::blocking::Response,
    ) -> Result<reqwest::blocking::Response, TransportError> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let detail = response.text().unwrap_or_default();
        Err(TransportError::Request(if detail.is_empty() {
            format!("the server answered {status}")
        } else {
            format!("the server answered {status}: {detail}")
        }))
    }
}

impl SyncTransport for MtlsSyncTransport {
    fn push(
        &mut self,
        organization_id: &str,
        events: &[OutboundEvent],
    ) -> Result<Vec<PushAck>, String> {
        let body = PushRequest {
            organization_id: organization_id.to_string(),
            events: events.to_vec(),
        };
        let response = self
            .client
            .post(format!("{}/v1/events/push", self.base_url))
            .header("X-Organization-Id", organization_id)
            .json(&body)
            .send()
            .map_err(|error| TransportError::Request(error.to_string()).to_string())?;
        let response = Self::check(response).map_err(|error| error.to_string())?;
        let parsed: PushResponse = response
            .json()
            .map_err(|error| TransportError::Response(error.to_string()).to_string())?;
        Ok(parsed.acks)
    }

    fn pull(
        &mut self,
        organization_id: &str,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<InboundEvent>, String> {
        let url = format!(
            "{}/v1/events/pull?after={after_sequence}&limit={}",
            self.base_url,
            limit.clamp(1, 200)
        );
        let response = self
            .client
            .get(url)
            .header("X-Organization-Id", organization_id)
            .send()
            .map_err(|error| TransportError::Request(error.to_string()).to_string())?;
        let response = Self::check(response).map_err(|error| error.to_string())?;
        let parsed: PullResponse = response
            .json()
            .map_err(|error| TransportError::Response(error.to_string()).to_string())?;
        Ok(parsed.events)
    }
}
