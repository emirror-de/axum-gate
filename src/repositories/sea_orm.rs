//! SeaORM repository integration providing account & credential persistence with constant‑time verification.
//!
//! This repository includes constant-time credential verification to
//! mitigate user enumeration via timing differences. A dummy Argon2
//! hash (built with the active build-mode preset) is precomputed at
//! construction and used whenever a secret for a given account id
//! does not exist, ensuring the Argon2 verification path is always
//! executed.

use crate::errors::Result;
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;

use sea_orm::DatabaseConnection;

/// SeaORM persistence entities (database models) used by `SeaOrmRepository`.
///
/// These are thin schemas mapping relational rows to structures convertible
/// to and from the domain layer (`Account`, `Secret`).
pub mod models;

mod account;
mod group;
mod permission_mapping;
mod secret;

/// Repository implementation for [SeaORM](sea_orm).
///
/// # Responsibilities
/// * Translate between domain `Account` / `Secret` and SeaORM models
/// * Provide CRUD operations required by higher‑level services
/// * Perform constant‑time credential verification
///
/// # Timing Side‑Channel Mitigation
/// A precomputed dummy Argon2 hash (same parameters as production hashes)
/// is always verified when an account's secret is missing. Existence and
/// hash‑match results are combined using bitwise operations on `subtle::Choice`
/// to avoid branching that could leak information.
///
/// # Concurrency
/// `SeaOrmRepository` is cheaply cloneable (internally holds a `DatabaseConnection`).
/// Clones share the same underlying pool and are `Send + Sync`.
///
/// # Error Semantics
/// Each DB interaction maps the concrete SeaORM / driver error into an
/// `DatabaseError::Operation` variant enriched with: operation, table,
/// and record identifier (when available).
///
/// # Usage
/// ```rust
/// # #[cfg(feature="storage-seaorm")]
/// # {
/// use axum_gate::repositories::sea_orm::SeaOrmRepository;
/// use sea_orm::Database;
/// # #[tokio::test] async fn usage_sea_orm() -> anyhow::Result<()> {
/// let db = Database::connect("sqlite::memory:").await?;
/// let repo = SeaOrmRepository::new(&db);
/// # Ok(()) }
/// # }
/// ```
///
/// # Extensibility
/// * To add new persisted aggregates: create a new model module, implement
///   conversions, and extend the repository or introduce a new trait.
/// * For multi‑tenant separation consider separate schemas / databases at
///   the connection level; this struct does not enforce tenant isolation.
///
/// # Security Considerations
/// * Still pair with rate limiting & structured logging
/// * Keep Argon2 parameters strong and consistent
/// * Secrets are assumed already hashed (insertion path uses hashed values)
pub struct SeaOrmRepository {
    db: DatabaseConnection,
    /// Precomputed dummy Argon2 hash used for nonexistent accounts to keep
    /// verification timing consistent.
    dummy_hash: String,
}

impl SeaOrmRepository {
    /// Creates a new repository that uses the given database connection as backend.
    pub fn new(db: &DatabaseConnection) -> Result<Self> {
        let hasher = Argon2Hasher::new_recommended()?;
        let dummy_hash = hasher.hash_value("dummy_password")?;
        Ok(Self {
            db: db.clone(),
            dummy_hash,
        })
    }
}
