//! Repository implementations for account and secret storage.
//!
//! This module provides repository abstractions and implementations for storing
//! user accounts and authentication secrets across different storage backends.
//!
//! # Available Backends
//!
//! - [`memory`] - In-memory storage (development, testing)
//! - `surrealdb` - SurrealDB backend (feature: `storage-surrealdb`)
//! - `sea_orm` - SeaORM backend (feature: `storage-seaorm`)
//!
//! # Usage
//!
//! ## In-Memory (Development)
//! ```rust
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository};
//! use axum_gate::prelude::{Role, Group};
//! use std::sync::Arc;
//!
//! let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
//! let secret_repo = Arc::new(MemorySecretRepository::new_with_argon2_hasher().unwrap());
//! ```
//!
//! ## SurrealDB (Production)
//! ```rust
//! # #[cfg(feature = "storage-surrealdb")]
//! # {
//! use axum_gate::repositories::surrealdb::{SurrealDbRepository, DatabaseScope};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let db = surrealdb::Surreal::new::<surrealdb::engine::local::Mem>(()).await?;
//! let scope = DatabaseScope::default();
//! let repo = Arc::new(SurrealDbRepository::new(db, scope));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## SeaORM (SQL Databases)
//! ```rust
//! # #[cfg(feature = "storage-seaorm")]
//! # {
//! use axum_gate::repositories::sea_orm::SeaOrmRepository;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let db = sea_orm::Database::connect("sqlite::memory:").await?;
//! let repo = Arc::new(SeaOrmRepository::new(&db));
//! # Ok(())
//! # }
//! # }
//! ```

pub mod errors;
pub mod memory;
#[cfg(any(feature = "storage-seaorm", feature = "storage-seaorm-v2"))]
pub mod sea_orm;
#[cfg(feature = "storage-surrealdb")]
pub mod surrealdb;
pub use errors::{
    DatabaseError, DatabaseOperation, RepositoriesError, RepositoryOperation, RepositoryType,
};

/// Table names used by the storage backends.
///
/// Added table names MUST remain stable across backends. New storage backends
/// or features that require additional tables should add a corresponding entry
/// here so the scope configuration can reference them in a type-safe manner.
#[cfg(any(
    feature = "storage-surrealdb",
    feature = "storage-seaorm",
    feature = "storage-seaorm-v2"
))]
#[derive(strum::Display, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum TableName {
    /// Account storage table name.
    AxumGateAccounts,
    /// Credentials storage table name.
    AxumGateCredentials,
    /// Permission mappings storage table name.
    AxumGatePermissionMappings,
    /// Groups storage table name (used by group repository implementations).
    AxumGateGroups,
}
