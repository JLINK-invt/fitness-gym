use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] gym_data::DataError),

    #[error("metadata error: {0}")]
    Metadata(#[from] serde_json::Error),

    #[error("invalid backup format: {0}")]
    InvalidFormat(String),

    #[error("backup checksum mismatch")]
    ChecksumMismatch,

    #[error("invalid passphrase or corrupted snapshot")]
    InvalidPassphrase,

    #[error("passphrase must have at least 8 characters")]
    WeakPassphrase,

    #[error("crypto error: {0}")]
    Crypto(String),
}
