#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("the sync store could not be used: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("the sync store could not be prepared: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("the TLS material could not be read: {0}")]
    Io(#[from] std::io::Error),

    #[error("the TLS material is invalid: {0}")]
    InvalidMaterial(String),

    #[error("the sync store could not be used: {0}")]
    Store(#[from] StoreError),

    #[error("the request could not be served: {0}")]
    Request(String),
}
