use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid database key: {0}")]
    InvalidKey(String),

    #[error("SQLCipher is not available in this build")]
    EncryptionUnavailable,

    #[error("OS keychain error: {0}")]
    Keyring(String),

    #[error("database lock is poisoned")]
    LockPoisoned,

    #[error("database connection is closed")]
    DatabaseClosed,

    #[error("migration {version} checksum mismatch: expected {expected}, found {found}")]
    MigrationDrift {
        version: i64,
        expected: String,
        found: String,
    },

    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("this installation already has an organization")]
    OrganizationAlreadyExists,

    #[error("this installation already has an active owner")]
    OwnerAlreadyExists,

    #[error("a user with that name already exists")]
    UserAlreadyExists,

    #[error("the staff member does not exist in this gym")]
    StaffNotFound,

    #[error("the recovery code was already used or replaced")]
    RecoveryCodeConflict,

    #[error("a catalog product with that name already exists")]
    CatalogProductAlreadyExists,

    #[error("a catalog variant with that SKU already exists")]
    CatalogSkuAlreadyExists,

    #[error("a catalog variant with that barcode already exists")]
    CatalogBarcodeAlreadyExists,

    #[error("the catalog product does not exist in this gym")]
    CatalogProductNotFound,

    #[error("the catalog variant does not exist in this gym")]
    CatalogVariantNotFound,
}
