use std::sync::{Arc, Mutex};

use axum::extract::{FromRequestParts, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use gym_sync::wire::{PullResponse, PushRequest, PushResponse, SyncServerHealth};
use gym_sync::{PushAck, PushOutcome};
use serde::Deserialize;

use crate::store::{AppendOutcome, NewSyncEvent, SyncStore};
use crate::tls::common_name;

const MAX_EVENT_BYTES: usize = 256 * 1024;
const MAX_EVENTS_PER_PUSH: usize = 500;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<SyncStore>>,
}

/// The DER encoded certificate presented by the client during the handshake.
#[derive(Clone, Debug)]
pub struct PeerCertificate {
    pub der: Vec<u8>,
}

/// Identity of the verified client, taken from the certificate common name.
#[derive(Clone, Debug)]
pub struct PeerIdentity {
    pub organization_id: String,
}

impl<S> FromRequestParts<S> for PeerIdentity
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let certificate = parts
            .extensions
            .get::<PeerCertificate>()
            .ok_or_else(|| ApiError::unauthorized("the client certificate is required"))?;
        let organization_id = common_name(&certificate.der)
            .ok_or_else(|| ApiError::unauthorized("the client certificate has no common name"))?;
        Ok(Self { organization_id })
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }

    pub fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.to_string(),
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub after: Option<i64>,
    pub limit: Option<usize>,
}

/// Builds the router with every route the desktop client uses.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/events/push", post(push_events))
        .route("/v1/events/pull", get(pull_events))
        .with_state(state)
}

async fn health(
    State(state): State<AppState>,
    identity: PeerIdentity,
) -> Result<Json<SyncServerHealth>, ApiError> {
    let store = state
        .store
        .lock()
        .map_err(|error| ApiError::internal(&format!("the store is unavailable: {error}")))?;
    let last_sequence = store
        .last_sequence(&identity.organization_id)
        .map_err(|error| ApiError::internal(&error.to_string()))?;
    Ok(Json(SyncServerHealth {
        status: "ok".into(),
        organization_id: identity.organization_id,
        last_sequence,
    }))
}

async fn push_events(
    State(state): State<AppState>,
    identity: PeerIdentity,
    headers: HeaderMap,
    Json(request): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiError> {
    if let Some(header) = headers
        .get("x-organization-id")
        .and_then(|value| value.to_str().ok())
    {
        if header != identity.organization_id {
            return Err(ApiError::forbidden(
                "the organization does not match the client certificate",
            ));
        }
    }
    if request.organization_id != identity.organization_id {
        return Err(ApiError::forbidden(
            "the organization does not match the client certificate",
        ));
    }
    if request.events.len() > MAX_EVENTS_PER_PUSH {
        return Err(ApiError::bad_request("too many events in a single push"));
    }

    let mut acks = Vec::with_capacity(request.events.len());
    for event in request.events {
        let payload_json = serde_json::to_string(&event.payload).unwrap_or_default();
        if payload_json.len() > MAX_EVENT_BYTES {
            acks.push(PushAck {
                idempotency_key: event.idempotency_key.clone(),
                outcome: PushOutcome::Rejected("the payload is too large".into()),
            });
            continue;
        }
        if event.event_id.trim().is_empty()
            || event.event_type.trim().is_empty()
            || event.idempotency_key.trim().is_empty()
        {
            acks.push(PushAck {
                idempotency_key: event.idempotency_key.clone(),
                outcome: PushOutcome::Rejected("the event is incomplete".into()),
            });
            continue;
        }

        let mut store = state
            .store
            .lock()
            .map_err(|error| ApiError::internal(&format!("the store is unavailable: {error}")))?;
        let outcome = store
            .append(
                &identity.organization_id,
                NewSyncEvent {
                    event_id: &event.event_id,
                    event_type: &event.event_type,
                    aggregate_type: &event.aggregate_type,
                    aggregate_id: &event.aggregate_id,
                    idempotency_key: &event.idempotency_key,
                    payload_json: &payload_json,
                    occurred_at: &event.created_at,
                },
            )
            .map_err(|error| ApiError::internal(&error.to_string()))?;
        match outcome {
            AppendOutcome::Accepted { .. } | AppendOutcome::Duplicate { .. } => {
                acks.push(PushAck {
                    idempotency_key: event.idempotency_key.clone(),
                    outcome: PushOutcome::Accepted,
                });
            }
        }
    }

    Ok(Json(PushResponse { acks }))
}

async fn pull_events(
    State(state): State<AppState>,
    identity: PeerIdentity,
    headers: HeaderMap,
    Query(query): Query<PullQuery>,
) -> Result<Json<PullResponse>, ApiError> {
    if let Some(header) = headers
        .get("x-organization-id")
        .and_then(|value| value.to_str().ok())
    {
        if header != identity.organization_id {
            return Err(ApiError::forbidden(
                "the organization does not match the client certificate",
            ));
        }
    }
    let after = query.after.unwrap_or(0).max(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let store = state
        .store
        .lock()
        .map_err(|error| ApiError::internal(&format!("the store is unavailable: {error}")))?;
    let events = store
        .events_after(&identity.organization_id, after, limit)
        .map_err(|error| ApiError::internal(&error.to_string()))?;
    Ok(Json(PullResponse { events }))
}
