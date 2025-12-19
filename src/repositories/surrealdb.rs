//! SurrealDB-backed repositories for accounts and secrets with constant-time credential verification.

use super::TableName;
use crate::errors::{Error, Result};
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;
use crate::repositories::{DatabaseError, DatabaseOperation};

use std::default::Default;

use surrealdb::{Connection, Surreal};

mod account;
mod group;
mod permission_mapping;
mod secret;

/// Scope configuration (namespace, database, table names) used by `SurrealDbRepository`.
///
/// Most users can rely on `DatabaseScope::default()`. Override fields only if you
/// need custom namespace / database names or different table naming.
#[derive(Clone, Debug)]
pub struct DatabaseScope {
    /// Accounts table (stores user id, groups, roles).
    pub accounts: String,
    /// Credentials table (stores hashed secrets).
    pub credentials: String,
    /// Permission mappings table (stores normalized string <-> id mapping).
    pub permission_mappings: String,
    /// Groups table (stores persisted group entities).
    pub groups: String,
    /// Namespace where data is stored.
    pub namespace: String,
    /// Database name where data is stored.
    pub database: String,
}

impl Default for DatabaseScope {
    fn default() -> Self {
        Self {
            accounts: TableName::AxumGateAccounts.to_string(),
            credentials: TableName::AxumGateCredentials.to_string(),
            permission_mappings: TableName::AxumGatePermissionMappings.to_string(),
            groups: TableName::AxumGateGroups.to_string(),
            namespace: "axumGate".to_string(),
            database: "axumGate".to_string(),
        }
    }
}

/// SurrealDB-backed repository offering CRUD for accounts & secrets plus constant-time
/// credential verification (uses a precomputed dummy Argon2 hash when a secret is absent).
///
/// Use `SurrealDbRepository::new(db, DatabaseScope::default())` for standard setups.
#[derive(Clone)]
pub struct SurrealDbRepository<S>
where
    S: Connection,
{
    db: Surreal<S>,
    scope_settings: DatabaseScope,
    /// Precomputed dummy Argon2 hash used when a user's secret does not exist.
    /// Ensures the Argon2 verification path is always exercised.
    dummy_hash: String,
}

impl<S> SurrealDbRepository<S>
where
    S: Connection,
{
    /// Creates a new repository that uses the given database connection limited by the given scope.
    pub fn new(db: Surreal<S>, scope_settings: DatabaseScope) -> Result<Self> {
        let hasher = Argon2Hasher::new_recommended()?;
        // Panic on failure here is acceptable: construction failure indicates a
        // fundamental issue (e.g. RNG) and mirrors the in‑memory repo strategy.
        let dummy_hash = hasher.hash_value("dummy_password")?;
        Ok(Self {
            db,
            scope_settings,
            dummy_hash,
        })
    }

    /// Sets the correct namespace and database to use.
    async fn use_ns_db(&self) -> Result<()> {
        self.db
            .use_ns(&self.scope_settings.namespace)
            .use_db(&self.scope_settings.database)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Connect,
                    format!("Failed to set namespace/database: {}", e),
                    None,
                    None,
                ))
            })
    }
}
