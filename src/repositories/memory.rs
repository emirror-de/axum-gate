//! In-memory storage implementations for development and testing.
//!
//! This module provides repository implementations that store all data in memory.
//! These are ideal for development, testing, and small applications that don't
//! require persistent storage.
//!
//! # Features
//! - Zero configuration required
//! - Fast operations (no I/O)
//! - Perfect for unit tests and development
//! - Thread-safe with async support
//! - Automatic cleanup when dropped
//!
//! # Quick Start
//!
//! ```rust
//! use axum_gate::accounts::Account;
//! use axum_gate::prelude::{Role, Group};
//! use axum_gate::hashing::argon2::Argon2Hasher;
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository, MemoryPermissionMappingRepository};
//! use axum_gate::secrets::Secret;
//! use axum_gate::accounts::AccountRepository;
//! use axum_gate::secrets::SecretRepository;
//! use std::sync::Arc;
//!
//! # tokio_test::block_on(async {
//! // Create repositories
//! let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
//! let secret_repo = Arc::new(MemorySecretRepository::new_with_argon2_hasher().unwrap());
//! let mapping_repo = Arc::new(MemoryPermissionMappingRepository::default());
//!
//! // Create an account
//! let account = Account::new("user@example.com", &[Role::User], &[Group::new("staff")]);
//! let stored_account = account_repo.store_account(account).await.unwrap().unwrap();
//!
//! // Create corresponding secret
//! let secret = Secret::new(&stored_account.account_id, "password", Argon2Hasher::new_recommended().unwrap()).unwrap();
//! secret_repo.store_secret(secret).await.unwrap();
//!
//! // Query the account
//! let found = account_repo.query_account_by_user_id("user@example.com").await.unwrap();
//! assert!(found.is_some());
//! # });
//! ```
//!
//! # Creating from Existing Data
//!
//! ```rust
//! use axum_gate::accounts::Account;
//! use axum_gate::prelude::{Role, Group};
//! use axum_gate::secrets::Secret;
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository};
//!
//! // Create repositories with pre-populated data
//! let accounts = vec![
//!     Account::new("admin@example.com", &[Role::Admin], &[]),
//!     Account::new("user@example.com", &[Role::User], &[Group::new("staff")]),
//! ];
//! let account_repo = MemoryAccountRepository::from(accounts);
//!
//! let secrets = vec![/* your secrets */];
//! let secret_repo = MemorySecretRepository::try_from(secrets).unwrap();
//! ```
pub use self::account::*;
pub use self::group::*;
pub use self::permission_mapping::*;
pub use self::secret::*;

mod account;
mod group;
mod permission_mapping;
mod secret;
