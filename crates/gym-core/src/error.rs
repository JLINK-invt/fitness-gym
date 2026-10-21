use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("the role does not allow {0:?}")]
    Forbidden(rbac::Permission),

    #[error("duplicate event with idempotency key {0}")]
    DuplicateEvent(String),

    #[error("invalid domain value: {0}")]
    InvalidValue(String),
}

use crate::rbac;
