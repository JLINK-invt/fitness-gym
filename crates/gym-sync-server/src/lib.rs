//! Sync server for the fitness gym platform.
//!
//! Every gym connects with a client certificate whose common name is the
//! organization id. The server never trusts the body of a request: the
//! organization is always taken from the verified certificate.

pub mod app;
pub mod error;
pub mod server;
pub mod store;
pub mod tls;

pub use app::{router, AppState, PeerCertificate, PeerIdentity};
pub use error::{ServerError, StoreError};
pub use store::{AppendOutcome, NewSyncEvent, SyncStore};
