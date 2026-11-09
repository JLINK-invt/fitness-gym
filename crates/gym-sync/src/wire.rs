use serde::{Deserialize, Serialize};

use crate::{InboundEvent, OutboundEvent, PushAck};

/// Body sent by the client when it pushes local events to the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushRequest {
    pub organization_id: String,
    pub events: Vec<OutboundEvent>,
}

/// Acknowledgements returned by the server for every pushed event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushResponse {
    pub acks: Vec<PushAck>,
}

/// Events returned by the server for the requested cursor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullResponse {
    pub events: Vec<InboundEvent>,
}

/// Health payload exposed by the server for a verified client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncServerHealth {
    pub status: String,
    pub organization_id: String,
    pub last_sequence: i64,
}
