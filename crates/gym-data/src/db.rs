use gym_contracts::DatabaseStatus;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::DataError;
use crate::key::DbKey;
use crate::migrations::MIGRATIONS;

pub type LocalDatabaseStatus = DatabaseStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationRecord {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserCredentialRecord {
    pub user_id: String,
    pub organization_id: String,
    pub name: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffRecord {
    pub user_id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub last_login_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogPriceRecord {
    pub id: String,
    pub currency: String,
    pub amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub effective_from: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogVariantRecord {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub status: String,
    pub current_price: Option<CatalogPriceRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProductRecord {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub status: String,
    pub variants: Vec<CatalogVariantRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogPrice {
    pub amount_minor: i64,
    pub tax_rate_bps: i64,
    pub price_includes_tax: bool,
    pub effective_from: Option<String>,
}

struct ValidatedCatalogPrice {
    amount_minor: i64,
    tax_rate_bps: i64,
    price_includes_tax: bool,
    effective_from: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogItem {
    pub name: String,
    pub category: Option<String>,
    pub variant_name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub price: NewCatalogPrice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogVariant {
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub price: NewCatalogPrice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogProductUpdate {
    pub name: String,
    pub category: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCatalogVariantUpdate {
    pub name: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_minor: i64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventRecord {
    pub id: String,
    pub sequence: i64,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub origin: String,
    pub correlation_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditIntegrityRecord {
    pub valid: bool,
    pub checked_events: i64,
    pub first_broken_sequence: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEventRecord {
    pub id: String,
    pub event_type: String,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedModuleEvent {
    pub event: ModuleEventRecord,
    pub inserted: bool,
}

/// Handle to the local encrypted database. `Mutex<Option<Connection>>` keeps
/// Tauri state `Send + Sync`, serializes writes and allows closing every file
/// handle before a restore replaces the database on disk.
pub struct Db {
    conn: Mutex<Option<Connection>>,
    path: PathBuf,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Db")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Db {
    pub fn open(path: impl AsRef<Path>, key: &DbKey) -> Result<Self, DataError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Self::open_connection(&path, key)?;
        let db = Self {
            conn: Mutex::new(Some(conn)),
            path,
        };
        db.migrate()?;
        Ok(db)
    }

    fn open_connection(path: &Path, key: &DbKey) -> Result<Connection, DataError> {
        let conn = Connection::open(path)?;

        let pragma_key = key.to_pragma_hex();
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", pragma_key.as_str()))?;
        conn.execute_batch("PRAGMA cipher_memory_security = ON;")?;

        let cipher_version: Option<String> = conn
            .query_row("PRAGMA cipher_version", [], |row| row.get(0))
            .optional()?;
        match cipher_version {
            Some(version) if !version.trim().is_empty() => {}
            _ => return Err(DataError::EncryptionUnavailable),
        }

        // Touch the schema so a wrong key fails now, not on the first query.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|_| {
            DataError::InvalidKey("the database cannot be opened with the provided key".into())
        })?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        Ok(conn)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn migrate(&self) -> Result<(), DataError> {
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 version INTEGER PRIMARY KEY NOT NULL,
                 name TEXT NOT NULL,
                 checksum TEXT NOT NULL,
                 applied_at TEXT NOT NULL
             ) STRICT;",
        )?;

        let applied = {
            let mut statement =
                transaction.prepare("SELECT version, checksum FROM schema_migrations")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut map = HashMap::new();
            for row in rows {
                let (version, checksum) = row?;
                map.insert(version, checksum);
            }
            map
        };

        for migration in MIGRATIONS {
            let checksum = hex::encode(Sha256::digest(migration.up.as_bytes()));
            if let Some(existing) = applied.get(&migration.version) {
                if existing != &checksum {
                    return Err(DataError::MigrationDrift {
                        version: migration.version,
                        expected: checksum,
                        found: existing.clone(),
                    });
                }
                continue;
            }

            transaction.execute_batch(migration.up)?;
            transaction.execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    migration.version,
                    migration.name,
                    checksum,
                    chrono::Utc::now().to_rfc3339()
                ],
            )?;
        }

        let latest = MIGRATIONS
            .iter()
            .map(|migration| migration.version)
            .max()
            .unwrap_or(0);
        transaction.pragma_update(None, "user_version", latest)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn status(&self) -> Result<LocalDatabaseStatus, DataError> {
        self.with_connection(|conn| {
            let cipher_version: Option<String> = conn
                .query_row("PRAGMA cipher_version", [], |row| row.get(0))
                .optional()?;
            let integrity: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
            let schema_version: i64 =
                conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
            let organizations: i64 =
                conn.query_row("SELECT count(*) FROM organizations", [], |row| row.get(0))?;

            Ok(LocalDatabaseStatus {
                path: self.path.display().to_string(),
                encrypted: cipher_version.is_some(),
                cipher_version,
                schema_version,
                integrity_ok: integrity.eq_ignore_ascii_case("ok"),
                organizations,
            })
        })
    }

    pub fn create_organization(&self, name: &str) -> Result<String, DataError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(DataError::InvalidValue(
                "organization name cannot be empty".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        self.with_connection(|conn| {
            let exists: bool =
                conn.query_row("SELECT EXISTS(SELECT 1 FROM organizations)", [], |row| {
                    row.get(0)
                })?;
            if exists {
                return Err(DataError::OrganizationAlreadyExists);
            }

            conn.execute(
                "INSERT INTO organizations (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, chrono::Utc::now().to_rfc3339()],
            )
            .map_err(DataError::from)
        })?;
        Ok(id)
    }

    pub fn organization(&self) -> Result<Option<OrganizationRecord>, DataError> {
        self.with_connection(|conn| {
            conn.query_row("SELECT id, name FROM organizations LIMIT 1", [], |row| {
                Ok(OrganizationRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .optional()
            .map_err(DataError::from)
        })
    }

    pub fn has_owner(&self) -> Result<bool, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM staff_memberships
                     WHERE role = 'owner' AND status = 'active'
                 )",
                [],
                |row| row.get(0),
            )
            .map_err(DataError::from)
        })
    }

    pub fn list_staff(&self, organization_id: &str) -> Result<Vec<StaffRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT u.id, u.name, s.role, s.status, u.last_login_at, u.created_at
                 FROM users u
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1
                 ORDER BY CASE s.role
                     WHEN 'owner' THEN 0
                     WHEN 'manager' THEN 1
                     WHEN 'cashier' THEN 2
                     ELSE 3
                 END, u.name COLLATE NOCASE",
            )?;
            let rows = statement.query_map(params![organization_id], |row| {
                Ok(StaffRecord {
                    user_id: row.get(0)?,
                    name: row.get(1)?,
                    role: row.get(2)?,
                    status: row.get(3)?,
                    last_login_at: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    pub fn create_staff(
        &self,
        organization_id: &str,
        name: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<StaffRecord, DataError> {
        let name = name.trim();
        if name.is_empty() || password_hash.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "staff name and password hash are required".into(),
            ));
        }
        if !matches!(role, "manager" | "cashier" | "accounting") {
            return Err(DataError::InvalidValue("invalid staff role".into()));
        }

        let user_id = uuid::Uuid::new_v4().to_string();
        let membership_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        let user_exists: bool = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM users
                 WHERE organization_id = ?1 AND name = ?2 COLLATE NOCASE
             )",
            params![organization_id, name],
            |row| row.get(0),
        )?;
        if user_exists {
            return Err(DataError::UserAlreadyExists);
        }

        transaction.execute(
            "INSERT INTO users
                 (id, organization_id, name, password_hash, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
            params![user_id, organization_id, name, password_hash, created_at],
        )?;
        transaction.execute(
            "INSERT INTO staff_memberships
                 (id, organization_id, user_id, role, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5)",
            params![membership_id, organization_id, user_id, role, created_at],
        )?;
        transaction.commit()?;

        Ok(StaffRecord {
            user_id,
            name: name.to_string(),
            role: role.to_string(),
            status: "active".into(),
            last_login_at: None,
            created_at,
        })
    }

    pub fn update_staff(
        &self,
        organization_id: &str,
        user_id: &str,
        role: &str,
        status: &str,
    ) -> Result<StaffRecord, DataError> {
        if !matches!(role, "manager" | "cashier" | "accounting") {
            return Err(DataError::InvalidValue("invalid staff role".into()));
        }
        if !matches!(status, "active" | "revoked") {
            return Err(DataError::InvalidValue("invalid staff status".into()));
        }

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        let existing: Option<(String, String, String)> = transaction
            .query_row(
                "SELECT s.role, u.name, u.created_at
                 FROM users u
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1 AND u.id = ?2",
                params![organization_id, user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((current_role, name, created_at)) = existing else {
            return Err(DataError::StaffNotFound);
        };
        if current_role == "owner" {
            return Err(DataError::InvalidValue(
                "the owner membership cannot be modified".into(),
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        transaction.execute(
            "UPDATE staff_memberships
             SET role = ?3, status = ?4
             WHERE organization_id = ?1 AND user_id = ?2",
            params![organization_id, user_id, role, status],
        )?;
        transaction.execute(
            "UPDATE users SET status = ?3, updated_at = ?4
             WHERE organization_id = ?1 AND id = ?2",
            params![
                organization_id,
                user_id,
                if status == "active" {
                    "active"
                } else {
                    "disabled"
                },
                now
            ],
        )?;
        let last_login_at: Option<String> = transaction.query_row(
            "SELECT last_login_at FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        transaction.commit()?;

        Ok(StaffRecord {
            user_id: user_id.to_string(),
            name,
            role: role.to_string(),
            status: status.to_string(),
            last_login_at,
            created_at,
        })
    }

    pub fn list_catalog(
        &self,
        organization_id: &str,
    ) -> Result<Vec<CatalogProductRecord>, DataError> {
        self.load_catalog(organization_id, None)
    }

    pub fn catalog_product(
        &self,
        organization_id: &str,
        product_id: &str,
    ) -> Result<CatalogProductRecord, DataError> {
        self.load_catalog(organization_id, Some(product_id))?
            .into_iter()
            .next()
            .ok_or(DataError::CatalogProductNotFound)
    }

    fn load_catalog(
        &self,
        organization_id: &str,
        product_id: Option<&str>,
    ) -> Result<Vec<CatalogProductRecord>, DataError> {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.with_connection(|conn| {
            let mut product_statement = conn.prepare(
                "SELECT id, name, category, status, created_at, updated_at
                 FROM catalog_products
                 WHERE organization_id = ?1
                   AND (?2 IS NULL OR id = ?2)
                 ORDER BY (status = 'active') DESC, name COLLATE NOCASE",
            )?;
            let products = product_statement
                .query_map(params![organization_id, product_id], |row| {
                    Ok(CatalogProductRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        category: row.get(2)?,
                        status: row.get(3)?,
                        variants: Vec::new(),
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut variant_statement = conn.prepare(
                "SELECT v.id, v.product_id, v.name, v.sku, v.barcode, v.unit,
                        v.cost_minor, v.status, v.created_at, v.updated_at,
                        p.id, p.currency, p.amount_minor, p.tax_rate_bps,
                        p.price_includes_tax, p.effective_from, p.created_at
                 FROM catalog_variants v
                 LEFT JOIN catalog_price_versions p
                   ON p.id = (
                       SELECT id
                       FROM catalog_price_versions
                       WHERE variant_id = v.id AND effective_from <= ?2
                       ORDER BY effective_from DESC, created_at DESC
                       LIMIT 1
                   )
                 WHERE v.organization_id = ?1
                   AND (?3 IS NULL OR v.product_id = ?3)
                 ORDER BY v.name COLLATE NOCASE, v.created_at",
            )?;
            let variants = variant_statement
                .query_map(params![organization_id, now, product_id], |row| {
                    let price_id: Option<String> = row.get(10)?;
                    Ok(CatalogVariantRecord {
                        id: row.get(0)?,
                        product_id: row.get(1)?,
                        name: row.get(2)?,
                        sku: row.get(3)?,
                        barcode: row.get(4)?,
                        unit: row.get(5)?,
                        cost_minor: row.get(6)?,
                        status: row.get(7)?,
                        current_price: price_id
                            .map(|id| {
                                Ok::<CatalogPriceRecord, rusqlite::Error>(CatalogPriceRecord {
                                    id,
                                    currency: row.get(11)?,
                                    amount_minor: row.get(12)?,
                                    tax_rate_bps: row.get(13)?,
                                    price_includes_tax: row.get::<_, i64>(14)? != 0,
                                    effective_from: row.get(15)?,
                                    created_at: row.get(16)?,
                                })
                            })
                            .transpose()?,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut products: Vec<CatalogProductRecord> = products;
            let positions: HashMap<String, usize> = products
                .iter()
                .enumerate()
                .map(|(index, product)| (product.id.clone(), index))
                .collect();
            for variant in variants {
                if let Some(index) = positions.get(&variant.product_id) {
                    products[*index].variants.push(variant);
                }
            }
            Ok(products)
        })
    }

    pub fn create_catalog_item(
        &self,
        organization_id: &str,
        created_by: &str,
        item: NewCatalogItem,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&item.name, "product name", 2, 80)?;
        let category = optional_text(item.category.as_deref(), "category", 2, 60)?;
        let variant_name = required_text(&item.variant_name, "variant name", 1, 80)?;
        let sku = required_text(&item.sku, "SKU", 1, 40)?;
        let barcode = optional_text(item.barcode.as_deref(), "barcode", 1, 64)?;
        validate_catalog_unit(&item.unit)?;
        if item.cost_minor < 0 {
            return Err(DataError::InvalidValue(
                "cost must be zero or greater".into(),
            ));
        }
        let price = validate_new_price(item.price)?;

        let product_id = uuid::Uuid::new_v4().to_string();
        let variant_id = uuid::Uuid::new_v4().to_string();
        let price_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let currency = organization_currency(&transaction, organization_id)?;

            ensure_unique_product_name(&transaction, organization_id, &name, None)?;
            ensure_unique_sku(&transaction, organization_id, &sku, None)?;
            ensure_unique_barcode(&transaction, organization_id, barcode.as_deref(), None)?;

            transaction.execute(
                "INSERT INTO catalog_products
                     (id, organization_id, name, category, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
                params![product_id, organization_id, name, category, now],
            )?;
            transaction.execute(
                "INSERT INTO catalog_variants
                     (id, organization_id, product_id, name, sku, barcode, unit,
                      cost_minor, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9)",
                params![
                    variant_id,
                    organization_id,
                    product_id,
                    variant_name,
                    sku,
                    barcode,
                    item.unit,
                    item.cost_minor,
                    now
                ],
            )?;
            transaction.execute(
                "INSERT INTO catalog_price_versions
                     (id, organization_id, variant_id, currency, amount_minor,
                      tax_rate_bps, price_includes_tax, effective_from, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    price_id,
                    organization_id,
                    variant_id,
                    currency,
                    price.amount_minor,
                    price.tax_rate_bps,
                    i64::from(price.price_includes_tax),
                    price.effective_from,
                    now,
                    created_by
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, &product_id)
    }

    pub fn add_catalog_variant(
        &self,
        organization_id: &str,
        created_by: &str,
        product_id: &str,
        variant: NewCatalogVariant,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&variant.name, "variant name", 1, 80)?;
        let sku = required_text(&variant.sku, "SKU", 1, 40)?;
        let barcode = optional_text(variant.barcode.as_deref(), "barcode", 1, 64)?;
        validate_catalog_unit(&variant.unit)?;
        if variant.cost_minor < 0 {
            return Err(DataError::InvalidValue(
                "cost must be zero or greater".into(),
            ));
        }
        let price = validate_new_price(variant.price)?;
        let variant_id = uuid::Uuid::new_v4().to_string();
        let price_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let currency = organization_currency(&transaction, organization_id)?;
            let product_status: Option<String> = transaction
                .query_row(
                    "SELECT status FROM catalog_products
                     WHERE id = ?1 AND organization_id = ?2",
                    params![product_id, organization_id],
                    |row| row.get(0),
                )
                .optional()?;
            match product_status.as_deref() {
                Some("active") => {}
                Some(_) => {
                    return Err(DataError::InvalidValue(
                        "cannot add variants to an archived product".into(),
                    ))
                }
                None => return Err(DataError::CatalogProductNotFound),
            }

            ensure_unique_sku(&transaction, organization_id, &sku, None)?;
            ensure_unique_barcode(&transaction, organization_id, barcode.as_deref(), None)?;

            transaction.execute(
                "INSERT INTO catalog_variants
                     (id, organization_id, product_id, name, sku, barcode, unit,
                      cost_minor, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9)",
                params![
                    variant_id,
                    organization_id,
                    product_id,
                    name,
                    sku,
                    barcode,
                    variant.unit,
                    variant.cost_minor,
                    now
                ],
            )?;
            transaction.execute(
                "INSERT INTO catalog_price_versions
                     (id, organization_id, variant_id, currency, amount_minor,
                      tax_rate_bps, price_includes_tax, effective_from, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    price_id,
                    organization_id,
                    variant_id,
                    currency,
                    price.amount_minor,
                    price.tax_rate_bps,
                    i64::from(price.price_includes_tax),
                    price.effective_from,
                    now,
                    created_by
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, product_id)
    }

    pub fn update_catalog_product(
        &self,
        organization_id: &str,
        product_id: &str,
        update: NewCatalogProductUpdate,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&update.name, "product name", 2, 80)?;
        let category = optional_text(update.category.as_deref(), "category", 2, 60)?;
        validate_catalog_status(&update.status)?;
        let now = chrono::Utc::now().to_rfc3339();

        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM catalog_products
                     WHERE id = ?1 AND organization_id = ?2
                 )",
                params![product_id, organization_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(DataError::CatalogProductNotFound);
            }
            ensure_unique_product_name(&transaction, organization_id, &name, Some(product_id))?;
            transaction.execute(
                "UPDATE catalog_products
                 SET name = ?3, category = ?4, status = ?5, updated_at = ?6
                 WHERE id = ?1 AND organization_id = ?2",
                params![
                    product_id,
                    organization_id,
                    name,
                    category,
                    update.status,
                    now
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, product_id)
    }

    pub fn update_catalog_variant(
        &self,
        organization_id: &str,
        variant_id: &str,
        update: NewCatalogVariantUpdate,
    ) -> Result<CatalogProductRecord, DataError> {
        let name = required_text(&update.name, "variant name", 1, 80)?;
        let sku = required_text(&update.sku, "SKU", 1, 40)?;
        let barcode = optional_text(update.barcode.as_deref(), "barcode", 1, 64)?;
        validate_catalog_unit(&update.unit)?;
        validate_catalog_status(&update.status)?;
        if update.cost_minor < 0 {
            return Err(DataError::InvalidValue(
                "cost must be zero or greater".into(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();

        let product_id: String;
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            product_id = transaction
                .query_row(
                    "SELECT product_id FROM catalog_variants
                     WHERE id = ?1 AND organization_id = ?2",
                    params![variant_id, organization_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or(DataError::CatalogVariantNotFound)?;

            ensure_unique_sku(&transaction, organization_id, &sku, Some(variant_id))?;
            ensure_unique_barcode(
                &transaction,
                organization_id,
                barcode.as_deref(),
                Some(variant_id),
            )?;
            transaction.execute(
                "UPDATE catalog_variants
                 SET name = ?3, sku = ?4, barcode = ?5, unit = ?6,
                     cost_minor = ?7, status = ?8, updated_at = ?9
                 WHERE id = ?1 AND organization_id = ?2",
                params![
                    variant_id,
                    organization_id,
                    name,
                    sku,
                    barcode,
                    update.unit,
                    update.cost_minor,
                    update.status,
                    now
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, &product_id)
    }

    pub fn add_catalog_price(
        &self,
        organization_id: &str,
        created_by: &str,
        variant_id: &str,
        price: NewCatalogPrice,
    ) -> Result<CatalogProductRecord, DataError> {
        let price = validate_new_price(price)?;
        let price_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let product_id: String;
        {
            let mut guard = self.lock()?;
            let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
            let transaction = conn.transaction()?;
            let (variant_product_id, variant_status, product_status): (String, String, String) =
                transaction
                    .query_row(
                        "SELECT v.product_id, v.status, p.status
                         FROM catalog_variants v
                         JOIN catalog_products p ON p.id = v.product_id
                         WHERE v.id = ?1 AND v.organization_id = ?2",
                        params![variant_id, organization_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?
                    .ok_or(DataError::CatalogVariantNotFound)?;
            if variant_status != "active" || product_status != "active" {
                return Err(DataError::InvalidValue(
                    "cannot add prices to an archived product or variant".into(),
                ));
            }
            product_id = variant_product_id;
            let currency = organization_currency(&transaction, organization_id)?;
            transaction.execute(
                "INSERT INTO catalog_price_versions
                     (id, organization_id, variant_id, currency, amount_minor,
                      tax_rate_bps, price_includes_tax, effective_from, created_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    price_id,
                    organization_id,
                    variant_id,
                    currency,
                    price.amount_minor,
                    price.tax_rate_bps,
                    i64::from(price.price_includes_tax),
                    price.effective_from,
                    now,
                    created_by
                ],
            )?;
            transaction.commit()?;
        }

        self.catalog_product(organization_id, &product_id)
    }

    /// Creates the first owner and its organization-wide membership atomically.
    /// There is no default user or universal password.
    pub fn create_owner(
        &self,
        organization_id: &str,
        name: &str,
        password_hash: &str,
    ) -> Result<UserRecord, DataError> {
        let name = name.trim();
        if name.is_empty() || password_hash.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "owner name and password hash are required".into(),
            ));
        }

        let user_id = uuid::Uuid::new_v4().to_string();
        let membership_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;

        let owner_exists: bool = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM staff_memberships
                 WHERE organization_id = ?1 AND role = 'owner' AND status = 'active'
             )",
            params![organization_id],
            |row| row.get(0),
        )?;
        if owner_exists {
            return Err(DataError::OwnerAlreadyExists);
        }

        let user_exists: bool = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM users
                 WHERE organization_id = ?1 AND name = ?2 COLLATE NOCASE
             )",
            params![organization_id, name],
            |row| row.get(0),
        )?;
        if user_exists {
            return Err(DataError::UserAlreadyExists);
        }

        transaction.execute(
            "INSERT INTO users
                 (id, organization_id, name, password_hash, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
            params![user_id, organization_id, name, password_hash, created_at],
        )?;
        transaction.execute(
            "INSERT INTO staff_memberships
                 (id, organization_id, user_id, role, status, created_at)
             VALUES (?1, ?2, ?3, 'owner', 'active', ?4)",
            params![membership_id, organization_id, user_id, created_at],
        )?;
        transaction.commit()?;

        Ok(UserRecord {
            id: user_id,
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            role: "owner".into(),
        })
    }

    /// Returns the active membership used for login. Lookup is scoped to the
    /// installation's organization and is case-insensitive for the name.
    pub fn find_user_credentials(
        &self,
        organization_id: &str,
        name: &str,
    ) -> Result<Option<UserCredentialRecord>, DataError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT u.id, u.organization_id, u.name, u.password_hash, s.role
                 FROM users u
                 JOIN staff_memberships s
                   ON s.organization_id = u.organization_id AND s.user_id = u.id
                 WHERE u.organization_id = ?1
                   AND u.name = ?2 COLLATE NOCASE
                   AND u.status = 'active'
                   AND s.status = 'active'
                 LIMIT 1",
                params![organization_id, name.trim()],
                |row| {
                    Ok(UserCredentialRecord {
                        user_id: row.get(0)?,
                        organization_id: row.get(1)?,
                        name: row.get(2)?,
                        password_hash: row.get(3)?,
                        role: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(DataError::from)
        })
    }

    pub fn mark_login(&self, user_id: &str) -> Result<(), DataError> {
        self.with_connection(|conn| {
            conn.execute(
                "UPDATE users SET last_login_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![user_id, chrono::Utc::now().to_rfc3339()],
            )?;
            Ok(())
        })
    }

    /// Appends a tamper-evident audit event. The hash chain is computed inside
    /// the same transaction as the insert, so each row commits with its
    /// predecessor hash. Triggers in migration 4 reject updates and deletes.
    #[allow(clippy::too_many_arguments)]
    pub fn record_audit_event(
        &self,
        organization_id: &str,
        actor_id: Option<&str>,
        action: &str,
        resource: &str,
        result: &str,
        origin: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), DataError> {
        if !matches!(result, "allowed" | "denied" | "failed") {
            return Err(DataError::InvalidValue("invalid audit result".into()));
        }
        if action.trim().is_empty() || resource.trim().is_empty() || origin.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "audit action, resource and origin cannot be empty".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let previous: Option<(i64, Option<String>)> = transaction
            .query_row(
                "SELECT sequence, event_hash
                 FROM audit_events
                 WHERE organization_id = ?1
                 ORDER BY sequence DESC
                 LIMIT 1",
                params![organization_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let sequence = previous.as_ref().map_or(1, |(sequence, _)| sequence + 1);
        let prev_hash = previous.and_then(|(_, hash)| hash);
        let event_hash = audit_event_hash(
            sequence,
            organization_id,
            &id,
            actor_id,
            action,
            resource,
            result,
            origin,
            correlation_id,
            &created_at,
            prev_hash.as_deref(),
        );
        transaction.execute(
            "INSERT INTO audit_events
                 (id, organization_id, actor_id, action, resource, result,
                  origin, correlation_id, sequence, prev_hash, event_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id,
                organization_id,
                actor_id,
                action,
                resource,
                result,
                origin,
                correlation_id,
                sequence,
                prev_hash,
                event_hash,
                created_at
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    /// Returns newest audit rows first for the whole installation.
    pub fn list_audit_events(
        &self,
        organization_id: &str,
        limit: i64,
    ) -> Result<Vec<AuditEventRecord>, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT a.id, a.sequence, a.actor_id, u.name, a.action,
                        a.resource, a.result, a.origin, a.correlation_id, a.created_at
                 FROM audit_events a
                 LEFT JOIN users u ON u.id = a.actor_id
                 WHERE a.organization_id = ?1
                 ORDER BY a.sequence DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(params![organization_id, limit], |row| {
                Ok(AuditEventRecord {
                    id: row.get(0)?,
                    sequence: row.get(1)?,
                    actor_id: row.get(2)?,
                    actor_name: row.get(3)?,
                    action: row.get(4)?,
                    resource: row.get(5)?,
                    result: row.get(6)?,
                    origin: row.get(7)?,
                    correlation_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    /// Recomputes the hash chain from the oldest hashed event to the newest.
    /// Legacy rows migrated from schema 3 have no hash and are skipped; the
    /// chain starts at the first post-migration append.
    pub fn verify_audit_chain(
        &self,
        organization_id: &str,
    ) -> Result<AuditIntegrityRecord, DataError> {
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT sequence, id, actor_id, action, resource, result,
                        origin, correlation_id, created_at, prev_hash, event_hash
                 FROM audit_events
                 WHERE organization_id = ?1 AND event_hash IS NOT NULL
                 ORDER BY sequence ASC",
            )?;
            let mut rows = statement.query(params![organization_id])?;
            let mut checked_events = 0;
            let mut previous_sequence: Option<i64> = None;
            let mut previous_hash: Option<String> = None;
            while let Some(row) = rows.next()? {
                checked_events += 1;
                let sequence: i64 = row.get(0)?;
                let id: String = row.get(1)?;
                let actor_id: Option<String> = row.get(2)?;
                let action: String = row.get(3)?;
                let resource: String = row.get(4)?;
                let result: String = row.get(5)?;
                let origin: String = row.get(6)?;
                let correlation_id: Option<String> = row.get(7)?;
                let created_at: String = row.get(8)?;
                let prev_hash: Option<String> = row.get(9)?;
                let event_hash: String = row.get(10)?;
                let expected = audit_event_hash(
                    sequence,
                    organization_id,
                    &id,
                    actor_id.as_deref(),
                    &action,
                    &resource,
                    &result,
                    &origin,
                    correlation_id.as_deref(),
                    &created_at,
                    prev_hash.as_deref(),
                );
                let sequence_ok = previous_sequence.is_none_or(|value| sequence == value + 1);
                let hash_ok = expected == event_hash && prev_hash == previous_hash;
                if !sequence_ok || !hash_ok {
                    return Ok(AuditIntegrityRecord {
                        valid: false,
                        checked_events,
                        first_broken_sequence: Some(sequence),
                    });
                }
                previous_sequence = Some(sequence);
                previous_hash = Some(event_hash);
            }
            Ok(AuditIntegrityRecord {
                valid: true,
                checked_events,
                first_broken_sequence: None,
            })
        })
    }

    /// Inserts an outbox event once per `(organization, idempotency_key)` and
    /// returns the durable row, including when it already existed.
    #[allow(clippy::too_many_arguments)]
    pub fn record_module_event(
        &self,
        organization_id: &str,
        direction: &str,
        event_type: &str,
        aggregate: &str,
        idempotency_key: &str,
        payload_hash: &str,
    ) -> Result<RecordedModuleEvent, DataError> {
        if idempotency_key.trim().is_empty() {
            return Err(DataError::InvalidValue(
                "idempotency key cannot be empty".into(),
            ));
        }

        let mut guard = self.lock()?;
        let conn = guard.as_mut().ok_or(DataError::DatabaseClosed)?;
        let transaction = conn.transaction()?;
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let inserted = transaction.execute(
            "INSERT INTO module_events
                 (id, organization_id, direction, event_type, aggregate,
                  idempotency_key, payload_hash, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'applied', ?8)
             ON CONFLICT (organization_id, idempotency_key) DO NOTHING",
            params![
                id,
                organization_id,
                direction,
                event_type,
                aggregate,
                idempotency_key,
                payload_hash,
                created_at
            ],
        )?;

        let event = transaction.query_row(
            "SELECT id, event_type, idempotency_key, payload_hash, created_at
             FROM module_events
             WHERE organization_id = ?1 AND idempotency_key = ?2",
            params![organization_id, idempotency_key],
            |row| {
                Ok(ModuleEventRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    idempotency_key: row.get(2)?,
                    payload_hash: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )?;
        transaction.commit()?;

        Ok(RecordedModuleEvent {
            event,
            inserted: inserted == 1,
        })
    }

    pub fn list_module_events(
        &self,
        aggregate: &str,
        limit: usize,
    ) -> Result<Vec<ModuleEventRecord>, DataError> {
        let limit = limit.clamp(1, 100) as i64;
        self.with_connection(|conn| {
            let mut statement = conn.prepare(
                "SELECT id, event_type, idempotency_key, payload_hash, created_at
                 FROM module_events
                 WHERE aggregate = ?1
                 ORDER BY rowid DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(params![aggregate, limit], |row| {
                Ok(ModuleEventRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    idempotency_key: row.get(2)?,
                    payload_hash: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DataError::from)
        })
    }

    /// Copies the open encrypted database to `destination` without exposing
    /// plaintext. SQLCipher re-encrypts with the current key.
    pub fn vacuum_into(&self, destination: &Path) -> Result<(), DataError> {
        if destination.exists() {
            fs::remove_file(destination)?;
        }

        self.with_connection(|conn| {
            conn.execute(
                "VACUUM INTO ?1",
                params![destination.to_string_lossy().as_ref()],
            )
            .map_err(DataError::from)
        })?;
        Ok(())
    }

    pub fn integrity_check(&self) -> Result<bool, DataError> {
        self.with_connection(|conn| {
            let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
            Ok(result.eq_ignore_ascii_case("ok"))
        })
    }

    /// Closes the connection and its WAL after a best-effort checkpoint. Every
    /// file handle is released so the database can be replaced on any platform.
    pub fn close(&self) -> Result<(), DataError> {
        let mut guard = self.lock()?;
        if let Some(conn) = guard.take() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            drop(conn);
        }
        Ok(())
    }

    /// Reopens the live connection after a restore. The database file is
    /// assumed to be already authenticated by the restore routine.
    pub fn reopen(&self, key: &DbKey) -> Result<(), DataError> {
        let conn = Self::open_connection(&self.path, key)?;
        {
            let mut guard = self.lock()?;
            *guard = Some(conn);
        }
        self.migrate()
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, DataError>,
    ) -> Result<T, DataError> {
        let guard = self.lock()?;
        let conn = guard.as_ref().ok_or(DataError::DatabaseClosed)?;
        operation(conn)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Option<Connection>>, DataError> {
        self.conn.lock().map_err(|_| DataError::LockPoisoned)
    }
}

fn required_text(
    value: &str,
    field: &str,
    min_chars: usize,
    max_chars: usize,
) -> Result<String, DataError> {
    let value = value.trim();
    let length = value.chars().count();
    if length < min_chars || length > max_chars {
        return Err(DataError::InvalidValue(format!(
            "{field} must be between {min_chars} and {max_chars} characters"
        )));
    }
    Ok(value.to_string())
}

fn optional_text(
    value: Option<&str>,
    field: &str,
    min_chars: usize,
    max_chars: usize,
) -> Result<Option<String>, DataError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let length = value.chars().count();
    if length < min_chars || length > max_chars {
        return Err(DataError::InvalidValue(format!(
            "{field} must be between {min_chars} and {max_chars} characters"
        )));
    }
    Ok(Some(value.to_string()))
}

fn validate_catalog_unit(value: &str) -> Result<(), DataError> {
    if matches!(value, "piece" | "service" | "session" | "package") {
        Ok(())
    } else {
        Err(DataError::InvalidValue("invalid catalog unit".into()))
    }
}

fn validate_catalog_status(value: &str) -> Result<(), DataError> {
    if matches!(value, "active" | "archived") {
        Ok(())
    } else {
        Err(DataError::InvalidValue("invalid catalog status".into()))
    }
}

fn validate_new_price(price: NewCatalogPrice) -> Result<ValidatedCatalogPrice, DataError> {
    if price.amount_minor < 0 {
        return Err(DataError::InvalidValue(
            "price must be zero or greater".into(),
        ));
    }
    if !(0..=10_000).contains(&price.tax_rate_bps) {
        return Err(DataError::InvalidValue(
            "tax rate must be between 0 and 10000 basis points".into(),
        ));
    }
    let effective_from = price
        .effective_from
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|date| {
                    date.with_timezone(&chrono::Utc)
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                })
                .map_err(|_| {
                    DataError::InvalidValue("price effective date must be RFC 3339".into())
                })
        })
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    Ok(ValidatedCatalogPrice {
        amount_minor: price.amount_minor,
        tax_rate_bps: price.tax_rate_bps,
        price_includes_tax: price.price_includes_tax,
        effective_from,
    })
}

fn organization_currency(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
) -> Result<String, DataError> {
    conn.query_row(
        "SELECT currency FROM organizations WHERE id = ?1",
        params![organization_id],
        |row| row.get(0),
    )
    .optional()?
    .ok_or_else(|| DataError::InvalidValue("the gym does not exist".into()))
}

fn ensure_unique_product_name(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM catalog_products
             WHERE organization_id = ?1
               AND name = ?2 COLLATE NOCASE
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, name, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(DataError::CatalogProductAlreadyExists);
    }
    Ok(())
}

fn ensure_unique_sku(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    sku: &str,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM catalog_variants
             WHERE organization_id = ?1
               AND sku = ?2 COLLATE NOCASE
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, sku, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(DataError::CatalogSkuAlreadyExists);
    }
    Ok(())
}

fn ensure_unique_barcode(
    conn: &rusqlite::Transaction<'_>,
    organization_id: &str,
    barcode: Option<&str>,
    exclude_id: Option<&str>,
) -> Result<(), DataError> {
    let Some(barcode) = barcode else {
        return Ok(());
    };
    let duplicate: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM catalog_variants
             WHERE organization_id = ?1
               AND barcode = ?2
               AND (?3 IS NULL OR id <> ?3)
         )",
        params![organization_id, barcode, exclude_id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(DataError::CatalogBarcodeAlreadyExists);
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn audit_event_hash(
    sequence: i64,
    organization_id: &str,
    id: &str,
    actor_id: Option<&str>,
    action: &str,
    resource: &str,
    result: &str,
    origin: &str,
    correlation_id: Option<&str>,
    created_at: &str,
    prev_hash: Option<&str>,
) -> String {
    fn push_field(buffer: &mut String, label: &str, value: Option<&str>) {
        buffer.push_str(label);
        buffer.push('=');
        match value {
            Some(value) => {
                buffer.push_str(&value.len().to_string());
                buffer.push(':');
                buffer.push_str(value);
            }
            None => buffer.push('-'),
        }
        buffer.push('\n');
    }

    let mut buffer = String::with_capacity(512);
    buffer.push_str("fitness-gym-audit-v2\n");
    push_field(&mut buffer, "sequence", Some(&sequence.to_string()));
    push_field(&mut buffer, "organization", Some(organization_id));
    push_field(&mut buffer, "id", Some(id));
    push_field(&mut buffer, "actor", actor_id);
    push_field(&mut buffer, "action", Some(action));
    push_field(&mut buffer, "resource", Some(resource));
    push_field(&mut buffer, "result", Some(result));
    push_field(&mut buffer, "origin", Some(origin));
    push_field(&mut buffer, "correlation", correlation_id);
    push_field(&mut buffer, "created_at", Some(created_at));
    push_field(&mut buffer, "prev_hash", prev_hash);
    crate::sha256_hex(buffer)
}

/// Opens an encrypted database with `key` and runs `PRAGMA integrity_check`.
/// Used by restore to validate a snapshot before replacing the live file.
pub fn verify_encrypted_database(path: &Path, key: &DbKey) -> Result<(), DataError> {
    let conn = Db::open_connection(path, key)?;
    let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result.eq_ignore_ascii_case("ok") {
        Ok(())
    } else {
        Err(DataError::InvalidValue(format!(
            "integrity check failed: {result}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp(dir: &tempfile::TempDir, key: &DbKey) -> Db {
        Db::open(dir.path().join("gym.db"), key).expect("opens database")
    }

    #[test]
    fn database_is_encrypted_on_disk() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        db.create_organization("Gimnasio Centro")
            .expect("creates org");

        let path = db.path().to_path_buf();
        drop(db);

        let header = &fs::read(&path).expect("reads file")[..16];
        assert_ne!(header, b"SQLite format 3\0");
    }

    #[test]
    fn migrations_are_reproducible() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let path = dir.path().join("gym.db");

        let first = Db::open(&path, &key).expect("first open");
        let first_status = first.status().expect("first status");
        drop(first);

        let second = Db::open(&path, &key).expect("second open");
        let second_status = second.status().expect("second status");

        assert_eq!(first_status.schema_version, second_status.schema_version);
        assert_eq!(second_status.schema_version, 6);
        assert_eq!(second_status.organizations, 0);
        assert!(second_status.encrypted);
        assert!(second_status.integrity_ok);
    }

    #[test]
    fn only_one_organization_per_installation() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);

        db.create_organization("Gimnasio Centro")
            .expect("creates the first organization");
        let error = db
            .create_organization("Gimnasio Duplicado")
            .expect_err("must reject a second organization");
        assert!(matches!(error, DataError::OrganizationAlreadyExists));
        assert_eq!(db.status().expect("status").organizations, 1);
    }

    #[test]
    fn owner_bootstrap_is_atomic_and_unique() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");

        assert!(!db.has_owner().expect("owner query"));
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$dummy")
            .expect("creates owner");
        assert_eq!(owner.name, "Ana Pérez");
        assert_eq!(owner.role, "owner");
        assert!(db.has_owner().expect("owner query"));

        let duplicate = db
            .create_owner(&organization_id, "Luis", "$argon2id$dummy")
            .expect_err("must reject a second owner");
        assert!(matches!(duplicate, DataError::OwnerAlreadyExists));

        let credentials = db
            .find_user_credentials(&organization_id, "ana pérez")
            .expect("looks up user")
            .expect("user exists");
        assert_eq!(credentials.user_id, owner.id);
        assert_eq!(credentials.role, "owner");
        assert_eq!(credentials.password_hash, "$argon2id$dummy");
    }

    #[test]
    fn staff_memberships_are_updatable() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        db.create_owner(&organization_id, "Ana Pérez", "$argon2id$owner")
            .expect("creates owner");

        let staff = db
            .create_staff(
                &organization_id,
                "Carlos Caja",
                "$argon2id$staff",
                "cashier",
            )
            .expect("creates staff");
        assert_eq!(staff.role, "cashier");
        assert_eq!(staff.status, "active");
        assert_eq!(db.list_staff(&organization_id).expect("lists").len(), 2);
        assert!(matches!(
            db.create_staff(&organization_id, "carlos caja", "$argon2id$x", "cashier")
                .expect_err("duplicate user"),
            DataError::UserAlreadyExists
        ));

        let credentials = db
            .find_user_credentials(&organization_id, "carlos caja")
            .expect("looks up staff")
            .expect("staff exists");
        assert_eq!(credentials.role, "cashier");

        let updated = db
            .update_staff(&organization_id, &staff.user_id, "accounting", "revoked")
            .expect("revokes staff");
        assert_eq!(updated.role, "accounting");
        assert_eq!(updated.status, "revoked");
        assert!(db
            .find_user_credentials(&organization_id, "Carlos Caja")
            .expect("looks up revoked staff")
            .is_none());

        db.update_staff(&organization_id, &staff.user_id, "manager", "active")
            .expect("reactivates staff");
        assert_eq!(
            db.find_user_credentials(&organization_id, "Carlos Caja")
                .expect("looks up staff")
                .expect("staff exists")
                .role,
            "manager"
        );
        assert!(matches!(
            db.update_staff(&organization_id, "missing", "cashier", "active")
                .expect_err("missing staff"),
            DataError::StaffNotFound
        ));
    }

    #[test]
    fn catalog_prices_are_versioned_and_append_only() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner")
            .expect("creates owner");

        let product = db
            .create_catalog_item(
                &organization_id,
                &owner.id,
                NewCatalogItem {
                    name: "Membresía mensual".into(),
                    category: Some("Membresías".into()),
                    variant_name: "Mensual".into(),
                    sku: "MEM-MEN".into(),
                    barcode: Some("750000000001".into()),
                    unit: "service".into(),
                    cost_minor: 0,
                    price: NewCatalogPrice {
                        amount_minor: 10_000,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: Some("2020-01-01T00:00:00Z".into()),
                    },
                },
            )
            .expect("creates product");
        assert_eq!(product.variants.len(), 1);
        let variant = &product.variants[0];
        assert_eq!(variant.sku, "MEM-MEN");
        assert_eq!(
            variant
                .current_price
                .as_ref()
                .map(|price| price.amount_minor),
            Some(10_000)
        );
        let original_price_id = variant
            .current_price
            .as_ref()
            .expect("initial price")
            .id
            .clone();

        db.update_catalog_product(
            &organization_id,
            &product.id,
            NewCatalogProductUpdate {
                name: "Membresía mensual premium".into(),
                category: Some("Membresías".into()),
                status: "active".into(),
            },
        )
        .expect("renames product");
        let after_rename = db
            .catalog_product(&organization_id, &product.id)
            .expect("reads product");
        assert_eq!(after_rename.name, "Membresía mensual premium");
        assert_eq!(
            after_rename.variants[0]
                .current_price
                .as_ref()
                .map(|price| price.amount_minor),
            Some(10_000)
        );

        db.add_catalog_price(
            &organization_id,
            &owner.id,
            &variant.id,
            NewCatalogPrice {
                amount_minor: 12_000,
                tax_rate_bps: 1_600,
                price_includes_tax: true,
                effective_from: Some("2020-06-01T02:00:00+02:00".into()),
            },
        )
        .expect("adds a new price version");
        let current = db
            .catalog_product(&organization_id, &product.id)
            .expect("reads product");
        let current_price = current.variants[0]
            .current_price
            .as_ref()
            .expect("current price");
        assert_eq!(current_price.amount_minor, 12_000);
        assert_ne!(current_price.id, original_price_id);
        let stored_effective: String = db
            .with_connection(|conn| {
                conn.query_row(
                    "SELECT effective_from FROM catalog_price_versions
                     WHERE amount_minor = 12000",
                    [],
                    |row| row.get(0),
                )
                .map_err(DataError::from)
            })
            .expect("reads normalized effective date");
        assert_eq!(stored_effective, "2020-06-01T00:00:00Z");

        db.add_catalog_price(
            &organization_id,
            &owner.id,
            &variant.id,
            NewCatalogPrice {
                amount_minor: 13_000,
                tax_rate_bps: 1_600,
                price_includes_tax: true,
                effective_from: Some("2999-01-01T00:00:00Z".into()),
            },
        )
        .expect("adds a future price version");
        let future = db
            .catalog_product(&organization_id, &product.id)
            .expect("reads product");
        assert_eq!(
            future.variants[0]
                .current_price
                .as_ref()
                .map(|price| price.amount_minor),
            Some(12_000),
            "a future price must not become current yet"
        );

        let version_count: i64 = db
            .with_connection(|conn| {
                conn.query_row("SELECT count(*) FROM catalog_price_versions", [], |row| {
                    row.get(0)
                })
                .map_err(DataError::from)
            })
            .expect("counts versions");
        assert_eq!(version_count, 3);

        let update = db.with_connection(|conn| {
            conn.execute("UPDATE catalog_price_versions SET amount_minor = 1", [])
                .map_err(DataError::from)
        });
        assert!(update.is_err(), "price versions must be immutable");
        let delete = db.with_connection(|conn| {
            conn.execute("DELETE FROM catalog_price_versions", [])
                .map_err(DataError::from)
        });
        assert!(delete.is_err(), "price versions must not be deleted");
    }

    #[test]
    fn catalog_rejects_duplicate_identifiers_and_archived_products() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");
        let owner = db
            .create_owner(&organization_id, "Ana Pérez", "$argon2id$owner")
            .expect("creates owner");
        let item = || NewCatalogItem {
            name: "Agua".into(),
            category: Some("Bebidas".into()),
            variant_name: "600 ml".into(),
            sku: "AGUA-600".into(),
            barcode: Some("750000000002".into()),
            unit: "piece".into(),
            cost_minor: 500,
            price: NewCatalogPrice {
                amount_minor: 1_500,
                tax_rate_bps: 1_600,
                price_includes_tax: true,
                effective_from: Some("2020-01-01T00:00:00Z".into()),
            },
        };
        let product = db
            .create_catalog_item(&organization_id, &owner.id, item())
            .expect("creates product");

        let mut duplicate_sku = item();
        duplicate_sku.name = "Agua mineral".into();
        duplicate_sku.barcode = None;
        assert!(matches!(
            db.create_catalog_item(&organization_id, &owner.id, duplicate_sku)
                .expect_err("rejects duplicate SKU"),
            DataError::CatalogSkuAlreadyExists
        ));

        let mut duplicate_barcode = item();
        duplicate_barcode.name = "Agua gasificada".into();
        duplicate_barcode.sku = "AGUA-600-GAS".into();
        assert!(matches!(
            db.create_catalog_item(&organization_id, &owner.id, duplicate_barcode)
                .expect_err("rejects duplicate barcode"),
            DataError::CatalogBarcodeAlreadyExists
        ));

        db.update_catalog_product(
            &organization_id,
            &product.id,
            NewCatalogProductUpdate {
                name: product.name.clone(),
                category: product.category.clone(),
                status: "archived".into(),
            },
        )
        .expect("archives product");
        assert!(matches!(
            db.add_catalog_variant(
                &organization_id,
                &owner.id,
                &product.id,
                NewCatalogVariant {
                    name: "1 L".into(),
                    sku: "AGUA-1L".into(),
                    barcode: None,
                    unit: "piece".into(),
                    cost_minor: 800,
                    price: NewCatalogPrice {
                        amount_minor: 2_500,
                        tax_rate_bps: 1_600,
                        price_includes_tax: true,
                        effective_from: None,
                    },
                }
            )
            .expect_err("archived product rejects variants"),
            DataError::InvalidValue(_)
        ));
        assert!(matches!(
            db.add_catalog_price(
                &organization_id,
                &owner.id,
                &product.variants[0].id,
                NewCatalogPrice {
                    amount_minor: 1_800,
                    tax_rate_bps: 1_600,
                    price_includes_tax: true,
                    effective_from: None,
                },
            )
            .expect_err("archived product rejects prices"),
            DataError::InvalidValue(_)
        ));
    }

    #[test]
    fn audit_chain_is_append_only_and_tamper_evident() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        let organization_id = db
            .create_organization("Gimnasio Centro")
            .expect("creates organization");

        db.record_audit_event(
            &organization_id,
            Some("user-1"),
            "auth.login",
            "session",
            "allowed",
            "tauri",
            None,
        )
        .expect("records login");
        db.record_audit_event(
            &organization_id,
            Some("user-1"),
            "staff.updated",
            "staff_membership",
            "allowed",
            "tauri",
            Some("corr-1"),
        )
        .expect("records update");
        db.record_audit_event(
            &organization_id,
            None,
            "auth.login",
            "session",
            "failed",
            "tauri",
            None,
        )
        .expect("records failure");

        let events = db
            .list_audit_events(&organization_id, 10)
            .expect("lists audit");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence, 3);
        assert_eq!(events[2].sequence, 1);
        assert_eq!(events[1].correlation_id.as_deref(), Some("corr-1"));

        let integrity = db
            .verify_audit_chain(&organization_id)
            .expect("verifies chain");
        assert!(integrity.valid);
        assert_eq!(integrity.checked_events, 3);

        let update = db.with_connection(|conn| {
            conn.execute("UPDATE audit_events SET result = 'denied'", [])
                .map_err(DataError::from)
        });
        assert!(update.is_err(), "updates must be rejected");
        let delete = db.with_connection(|conn| {
            conn.execute("DELETE FROM audit_events", [])
                .map_err(DataError::from)
        });
        assert!(delete.is_err(), "deletes must be rejected");

        db.with_connection(|conn| {
            conn.execute("DROP TRIGGER audit_events_no_update", [])?;
            conn.execute(
                "UPDATE audit_events SET result = 'denied' WHERE sequence = 2",
                [],
            )?;
            Ok(())
        })
        .expect("simulates local tampering");
        let tampered = db
            .verify_audit_chain(&organization_id)
            .expect("verifies tampered chain");
        assert!(!tampered.valid);
        assert_eq!(tampered.first_broken_sequence, Some(2));
    }

    #[test]
    fn module_events_are_idempotent_and_listed_once() {
        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        assert_eq!(db.organization().expect("organization query"), None);
        db.create_organization("Gimnasio Centro")
            .expect("creates organization");
        let organization = db
            .organization()
            .expect("organization query")
            .expect("organization exists");

        let first = db
            .record_module_event(
                &organization.id,
                "outbound",
                "access.allowed",
                "access_decision",
                "req-1",
                "hash-a",
            )
            .expect("records first event");
        assert!(first.inserted);

        let replay = db
            .record_module_event(
                &organization.id,
                "outbound",
                "access.allowed",
                "access_decision",
                "req-1",
                "hash-a",
            )
            .expect("replays event");
        assert!(!replay.inserted);
        assert_eq!(replay.event.id, first.event.id);

        let conflict = db
            .record_module_event(
                &organization.id,
                "outbound",
                "access.denied",
                "access_decision",
                "req-1",
                "hash-b",
            )
            .expect("returns existing event");
        assert!(!conflict.inserted);
        assert_eq!(conflict.event.payload_hash, "hash-a");

        let events = db
            .list_module_events("access_decision", 20)
            .expect("lists events");
        assert_eq!(events.len(), 1);
    }

    #[test]
    #[ignore = "latency smoke; run with --ignored --nocapture"]
    fn access_event_latency_p95_is_under_300ms() {
        use std::time::{Duration, Instant};

        let dir = tempfile::tempdir().expect("temp dir");
        let key = DbKey::generate();
        let db = open_temp(&dir, &key);
        db.create_organization("Gimnasio Centro")
            .expect("creates organization");
        let organization = db
            .organization()
            .expect("organization query")
            .expect("organization exists");

        let mut samples = Vec::with_capacity(100);
        for index in 0..100 {
            let started = Instant::now();
            db.record_module_event(
                &organization.id,
                "outbound",
                "access.allowed",
                "access_decision",
                &format!("req-{index}"),
                "hash",
            )
            .expect("records event");
            db.list_module_events("access_decision", 20)
                .expect("refreshes event list");
            samples.push(started.elapsed());
        }
        samples.sort_unstable();
        let p95 = samples[94];
        println!("access p95 = {p95:?}");
        assert!(p95 < Duration::from_millis(300));
    }

    #[test]
    fn wrong_key_is_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("gym.db");
        let key = DbKey::generate();
        drop(Db::open(&path, &key).expect("opens"));

        let error = Db::open(&path, &DbKey::generate()).expect_err("must fail");
        assert!(matches!(error, DataError::InvalidKey(_)));
    }
}
