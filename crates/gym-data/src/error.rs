use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

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

    #[error("a cash session is already open")]
    CashSessionAlreadyOpen,

    #[error("the cash session does not exist in this gym")]
    CashSessionNotFound,

    #[error("open a cash session before selling")]
    CashSessionRequired,

    #[error("the cash session is not open")]
    CashSessionNotOpen,

    #[error("the cash movement does not exist in this gym")]
    CashMovementNotFound,

    #[error("the idempotency key was already used with a different sale")]
    SaleIdempotencyConflict,

    #[error("the sale does not exist in this gym")]
    SaleNotFound,

    #[error("the sale return does not exist in this gym")]
    SaleReturnNotFound,

    #[error("the inventory movement does not exist in this gym")]
    InventoryMovementNotFound,

    #[error("a membership plan with that name already exists")]
    MembershipPlanAlreadyExists,

    #[error("the membership plan does not exist in this gym")]
    MembershipPlanNotFound,

    #[error("the member does not exist in this gym")]
    MemberNotFound,

    #[error("the member already has a current membership")]
    MembershipAlreadyCurrent,

    #[error("the membership does not exist in this gym")]
    MembershipNotFound,

    #[error("the membership cannot be changed from its current state")]
    MembershipStateConflict,

    #[error("the prospect does not exist in this gym")]
    ProspectNotFound,

    #[error("the prospect was already converted")]
    ProspectAlreadyConverted,

    #[error("the membership charge does not exist in this gym")]
    MembershipChargeNotFound,

    #[error("the membership charge is not payable")]
    MembershipChargeNotPayable,

    #[error("the membership payment does not exist in this gym")]
    MembershipPaymentNotFound,

    #[error("the payment exceeds the outstanding charge balance")]
    MembershipPaymentExceedsBalance,

    #[error("the idempotency key was already used with a different membership payment")]
    MembershipPaymentIdempotencyConflict,

    #[error("the member credential does not exist in this gym")]
    MemberCredentialNotFound,

    #[error("the idempotency key was already used with a different access event")]
    AccessIdempotencyConflict,

    #[error("the report schedule does not exist in this gym")]
    ReportScheduleNotFound,
}
