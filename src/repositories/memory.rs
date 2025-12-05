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
use crate::accounts::Account;
use crate::accounts::AccountRepository;
use crate::authz::AccessHierarchy;
use crate::credentials::Credentials;
use crate::credentials::CredentialsVerifier;
use crate::errors::{Error, Result};
use crate::groups::{GroupEntity, GroupRepository};
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;
use crate::permissions::PermissionId;
use crate::permissions::mapping::PermissionMapping;
use crate::permissions::mapping::PermissionMappingRepository;
use crate::repositories::{RepositoriesError, RepositoryOperation, RepositoryType};
use crate::secrets::Secret;
use crate::secrets::SecretRepository;
use crate::verification_result::VerificationResult;
use serde::{Serialize, de::DeserializeOwned};

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

/// In-memory repository for storing and retrieving user accounts.
///
/// This repository stores all account data in memory using a HashMap with the user ID
/// as the key. It's thread-safe and supports concurrent access through async read/write locks.
///
/// # Performance Characteristics
/// - O(1) lookup by user ID
/// - Thread-safe with RwLock
/// - No persistence (data lost when dropped)
/// - Suitable for up to thousands of accounts
///
/// # Example
/// ```rust
/// use axum_gate::accounts::Account;
/// use axum_gate::prelude::{Role, Group};
/// use axum_gate::accounts::AccountRepository;
/// use axum_gate::repositories::memory::MemoryAccountRepository;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(async {
/// let repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
///
/// // Store an account
/// let account = Account::new("user@example.com", &[Role::User], &[]);
/// let stored = repo.store_account(account).await.unwrap();
///
/// // Query the account
/// let found = repo.query_account_by_user_id("user@example.com").await.unwrap();
/// assert!(found.is_some());
/// # });
/// ```
#[derive(Clone)]
pub struct MemoryAccountRepository<R, G>
where
    R: AccessHierarchy + Eq + Send + Sync + 'static,
    G: Eq + Clone + Send + Sync + 'static,
{
    accounts: Arc<RwLock<HashMap<String, Account<R, G>>>>,
}

impl<R, G> Default for MemoryAccountRepository<R, G>
where
    R: AccessHierarchy + Eq + Send + Sync + 'static,
    G: Eq + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<R, G> From<Vec<Account<R, G>>> for MemoryAccountRepository<R, G>
where
    R: AccessHierarchy + Eq + Send + Sync + 'static,
    G: Eq + Clone + Send + Sync + 'static,
{
    fn from(value: Vec<Account<R, G>>) -> Self {
        let mut accounts = HashMap::new();
        for val in value {
            let id = val.user_id.clone();
            accounts.insert(id, val);
        }
        let accounts = Arc::new(RwLock::new(accounts));
        Self { accounts }
    }
}

impl<R, G> AccountRepository<R, G> for MemoryAccountRepository<R, G>
where
    Account<R, G>: Clone,
    R: AccessHierarchy + Eq + Send + Sync + 'static,
    G: Eq + Clone + Send + Sync + 'static,
{
    async fn query_account_by_user_id(&self, user_id: &str) -> Result<Option<Account<R, G>>> {
        let read = self.accounts.read().await;
        Ok(read.get(user_id).cloned())
    }
    async fn store_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        let id = account.user_id.clone();
        let mut write = self.accounts.write().await;
        write.insert(id, account.clone());
        Ok(Some(account))
    }
    async fn delete_account(&self, account_id: &str) -> Result<Option<Account<R, G>>> {
        let mut write = self.accounts.write().await;
        if !write.contains_key(account_id) {
            return Ok(None);
        }
        Ok(write.remove(account_id))
    }
    async fn update_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        self.store_account(account).await
    }

    async fn query_all_accounts(&self) -> Result<Vec<Account<R, G>>> {
        let read = self.accounts.read().await;
        // Collect cloned account values into a Vec
        Ok(read.values().cloned().collect())
    }
}

/// In-memory repository for group entities.
///
/// This implementation is generic over `T` where `T: Serialize + DeserializeOwned + GroupEntity`.
/// It provides the same semantics as other in-memory repositories: thread-safe, fast,
/// and suitable for tests and development.
#[derive(Clone)]
pub struct MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    store: Arc<RwLock<HashMap<String, T>>>,
}

impl<T> Default for MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> From<Vec<T>> for MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    fn from(value: Vec<T>) -> Self {
        let mut map = HashMap::new();
        for v in value {
            map.insert(v.group_id().to_string(), v);
        }
        Self {
            store: Arc::new(RwLock::new(map)),
        }
    }
}

impl<T> GroupRepository<T> for MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    async fn store_group(&self, group: T) -> Result<bool> {
        let id = group.group_id().to_string();

        // Fast read check to avoid acquiring write lock unnecessarily
        {
            let read = self.store.read().await;
            if read.contains_key(&id) {
                return Ok(false);
            }
        }

        let mut write = self.store.write().await;
        write.insert(id, group);
        Ok(true)
    }

    async fn delete_group(&self, id: &str) -> Result<Option<T>> {
        let mut write = self.store.write().await;
        Ok(write.remove(id))
    }

    async fn update_group(&self, group: T) -> Result<Option<T>> {
        let id = group.group_id().to_string();
        let mut write = self.store.write().await;
        if write.contains_key(&id) {
            write.insert(id.clone(), group.clone());
            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    async fn query_group_by_id(&self, id: &str) -> Result<Option<T>> {
        let read = self.store.read().await;
        Ok(read.get(id).cloned())
    }

    async fn query_all_groups(&self) -> Result<Vec<T>> {
        let read = self.store.read().await;
        Ok(read.values().cloned().collect())
    }
}
/// In-memory repository for storing and managing user authentication secrets.
///
/// This repository stores password hashes and other authentication secrets in memory.
/// It's designed to work alongside `MemoryAccountRepository` and implements both
/// `SecretRepository` and `CredentialsVerifier` traits for complete authentication support.
///
/// # Security Note
/// While this stores password hashes (not plain passwords), the data is kept in memory
/// and will be lost when the application stops. For production use, consider persistent
/// storage implementations.
///
/// # Example Usage
/// ```rust
/// use axum_gate::prelude::Credentials;
/// use axum_gate::secrets::Secret; use axum_gate::verification_result::VerificationResult; use axum_gate::hashing::argon2::Argon2Hasher; use axum_gate::secrets::SecretRepository; use axum_gate::credentials::CredentialsVerifier;
/// use axum_gate::repositories::memory::MemorySecretRepository;
/// use uuid::Uuid;
///
/// # tokio_test::block_on(async {
/// let repo = MemorySecretRepository::new_with_argon2_hasher().unwrap();
/// let account_id = Uuid::now_v7();
///
/// // Store a secret (password hash)
/// let secret = Secret::new(&account_id, "user_password", Argon2Hasher::new_recommended().unwrap()).unwrap();
/// repo.store_secret(secret).await.unwrap();
///
/// // Verify credentials
/// let credentials = Credentials::new(&account_id, "user_password");
/// let result = repo.verify_credentials(credentials).await.unwrap();
/// assert_eq!(result, VerificationResult::Ok);
///
/// // Test wrong password
/// let wrong_creds = Credentials::new(&account_id, "wrong_password");
/// let result = repo.verify_credentials(wrong_creds).await.unwrap();
/// assert_eq!(result, VerificationResult::Unauthorized);
/// # });
/// ```
///
/// # Creating from Existing Data
/// ```rust
/// use axum_gate::secrets::Secret; use axum_gate::hashing::argon2::Argon2Hasher;
/// use axum_gate::repositories::memory::MemorySecretRepository;
/// use uuid::Uuid;
///
/// let secrets = vec![
///     Secret::new(&Uuid::now_v7(), "admin_pass", Argon2Hasher::new_recommended().unwrap()).unwrap(),
///     Secret::new(&Uuid::now_v7(), "user_pass", Argon2Hasher::new_recommended().unwrap()).unwrap(),
/// ];
/// let repo = MemorySecretRepository::try_from(secrets).unwrap();
/// ```
#[derive(Clone)]
pub struct MemorySecretRepository {
    store: Arc<RwLock<HashMap<Uuid, Secret>>>,
    /// Precomputed dummy hash produced with the same Argon2 preset that `Secret::new`
    /// used (via `Argon2Hasher::new_recommended()`) in this build configuration. This keeps
    /// timing of nonexistent-account verifications aligned with existing-account
    /// verifications to mitigate user enumeration via timing side channels.
    dummy_hash: String,
}

impl MemorySecretRepository {
    /// Creates a new instance with [Argon2Hasher].
    pub fn new_with_argon2_hasher() -> Result<Self> {
        let hasher = Argon2Hasher::new_recommended()?;
        let dummy_hash = hasher.hash_value("dummy_password")?;
        Ok(Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            dummy_hash,
        })
    }
}

impl TryFrom<Vec<Secret>> for MemorySecretRepository {
    type Error = crate::errors::Error;
    fn try_from(value: Vec<Secret>) -> Result<Self> {
        let mut store = HashMap::with_capacity(value.len());
        value.into_iter().for_each(|v| {
            store.insert(v.account_id, v);
        });
        let store = Arc::new(RwLock::new(store));
        let dummy_hash = Argon2Hasher::new_recommended()?.hash_value("dummy_password")?;
        Ok(Self { store, dummy_hash })
    }
}

impl SecretRepository for MemorySecretRepository {
    async fn store_secret(&self, secret: Secret) -> Result<bool> {
        let already_present = {
            let read = self.store.read().await;
            read.contains_key(&secret.account_id)
        };

        if already_present {
            return Err(Error::Repositories(RepositoriesError::operation_failed(
                RepositoryType::Secret,
                RepositoryOperation::Insert,
                "AccountID is already present",
                None,
                None,
            )));
        }

        let mut write = self.store.write().await;
        debug!("Got write lock on secret repository.");

        if write.insert(secret.account_id, secret).is_some() {
            return Err(Error::Repositories(RepositoriesError::operation_failed(
                RepositoryType::Secret,
                RepositoryOperation::Insert,
                "This should never occur because it is checked if the key is already present a few lines earlier",
                None,
                Some("store".to_string()),
            )));
        };
        Ok(true)
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<Option<Secret>> {
        // Atomically remove and return the secret (compensating actions can reinsert it)
        let mut write = self.store.write().await;
        Ok(write.remove(id))
    }

    async fn update_secret(&self, secret: Secret) -> Result<()> {
        let mut write = self.store.write().await;
        write.insert(secret.account_id, secret);
        Ok(())
    }
}

impl CredentialsVerifier<Uuid> for MemorySecretRepository {
    async fn verify_credentials(
        &self,
        credentials: Credentials<Uuid>,
    ) -> Result<VerificationResult> {
        use crate::hashing::HashingService;
        use subtle::Choice;

        let read = self.store.read().await;

        // Get stored secret or use precomputed dummy hash to ensure constant-time operation
        let (stored_secret_str, user_exists_choice) = match read.get(&credentials.id) {
            Some(stored_secret) => (stored_secret.secret.as_str(), Choice::from(1u8)),
            None => (self.dummy_hash.as_str(), Choice::from(0u8)),
        };

        // ALWAYS perform Argon2 verification (constant time regardless of user existence)
        let hasher = Argon2Hasher::new_recommended()?;
        let hash_verification_result =
            hasher.verify_value(&credentials.secret, stored_secret_str)?;

        // Convert hash verification result to Choice for constant-time operations
        let hash_matches_choice = Choice::from(match hash_verification_result {
            VerificationResult::Ok => 1u8,
            VerificationResult::Unauthorized => 0u8,
        });

        // Combine results using constant-time AND operation
        // Success only if: user exists AND password hash matches
        let final_success_choice = user_exists_choice & hash_matches_choice;

        // Convert back to VerificationResult
        let final_result = if bool::from(final_success_choice) {
            VerificationResult::Ok
        } else {
            VerificationResult::Unauthorized
        };

        Ok(final_result)
    }
}

/// In-memory implementation of [`PermissionMappingRepository`] for development and testing.
///
/// This repository stores permission mappings in memory using thread-safe data structures.
/// It's ideal for development, testing, and small applications that don't require
/// persistent storage of permission mappings.
///
/// # Thread Safety
///
/// This implementation uses `Arc<RwLock<HashMap>>` for thread-safe access to the
/// stored mappings. Multiple readers can access the data concurrently, while
/// writers have exclusive access.
///
/// # Storage Strategy
///
/// Mappings are stored in two hash maps for efficient lookup:
/// - By permission ID for reverse lookup (primary use case)
/// - By normalized string for string-based queries
///
/// # Example
///
/// ```rust
/// use axum_gate::permissions::mapping::PermissionMapping;
/// use axum_gate::permissions::PermissionId; use axum_gate::repositories::memory::MemoryPermissionMappingRepository;
/// use axum_gate::permissions::mapping::PermissionMappingRepository;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(async {
/// let repo = Arc::new(MemoryPermissionMappingRepository::default());
///
/// // Store a mapping
/// let mapping = PermissionMapping::from("read:api");
/// let stored = repo.store_mapping(mapping.clone()).await.unwrap();
/// assert!(stored.is_some());
///
/// // Query by ID
/// let found = repo.query_mapping_by_id(mapping.permission_id()).await.unwrap();
/// assert!(found.is_some());
/// # });
/// ```
#[derive(Debug)]
pub struct MemoryPermissionMappingRepository {
    /// Storage of all mappings. Lookups are performed via iteration.
    mappings: Arc<RwLock<Vec<PermissionMapping>>>,
}

impl Default for MemoryPermissionMappingRepository {
    fn default() -> Self {
        Self {
            mappings: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl From<Vec<PermissionMapping>> for MemoryPermissionMappingRepository {
    fn from(mappings: Vec<PermissionMapping>) -> Self {
        let mut vec: Vec<PermissionMapping> = Vec::new();

        for mapping in mappings {
            // Validate the mapping before storing
            if let Err(e) = mapping.validate() {
                tracing::warn!("Skipping invalid permission mapping: {}", e);
                continue;
            }

            let id = mapping.permission_id().as_u64();
            let normalized = mapping.normalized_string();

            // avoid duplicates by id or normalized string
            if !vec
                .iter()
                .any(|m| m.permission_id().as_u64() == id || m.normalized_string() == normalized)
            {
                vec.push(mapping);
            }
        }

        Self {
            mappings: Arc::new(RwLock::new(vec)),
        }
    }
}

impl PermissionMappingRepository for MemoryPermissionMappingRepository {
    async fn store_mapping(&self, mapping: PermissionMapping) -> Result<Option<PermissionMapping>> {
        // Validate the mapping first
        if let Err(e) = mapping.validate() {
            return Err(Error::Repositories(RepositoriesError::operation_failed(
                RepositoryType::PermissionMapping,
                RepositoryOperation::Insert,
                format!("Invalid permission mapping: {}", e),
                None,
                Some("store".to_string()),
            )));
        }

        let id = mapping.permission_id().as_u64();
        let normalized = mapping.normalized_string();

        // Check if mapping already exists (by ID or normalized string)
        {
            let vec_read = self.mappings.read().await;
            if vec_read
                .iter()
                .any(|m| m.permission_id().as_u64() == id || m.normalized_string() == normalized)
            {
                return Ok(None); // Mapping already exists
            }
        }

        // Store in vector
        {
            let mut vec_write = self.mappings.write().await;
            vec_write.push(mapping.clone());
        }

        Ok(Some(mapping))
    }

    async fn remove_mapping_by_id(&self, id: PermissionId) -> Result<Option<PermissionMapping>> {
        let id_u64 = id.as_u64();

        let mut vec_write = self.mappings.write().await;
        if let Some(pos) = vec_write
            .iter()
            .position(|m| m.permission_id().as_u64() == id_u64)
        {
            let removed = vec_write.swap_remove(pos);
            Ok(Some(removed))
        } else {
            Ok(None)
        }
    }

    async fn remove_mapping_by_string(
        &self,
        permission: &str,
    ) -> Result<Option<PermissionMapping>> {
        let normalized = normalize_permission(permission);

        let mut vec_write = self.mappings.write().await;
        if let Some(pos) = vec_write
            .iter()
            .position(|m| m.normalized_string() == normalized.as_str())
        {
            let removed = vec_write.swap_remove(pos);
            Ok(Some(removed))
        } else {
            Ok(None)
        }
    }

    async fn query_mapping_by_id(&self, id: PermissionId) -> Result<Option<PermissionMapping>> {
        let vec_read = self.mappings.read().await;
        Ok(vec_read
            .iter()
            .find(|m| m.permission_id().as_u64() == id.as_u64())
            .cloned())
    }

    async fn query_mapping_by_string(&self, permission: &str) -> Result<Option<PermissionMapping>> {
        let normalized = normalize_permission(permission);
        let vec_read = self.mappings.read().await;
        Ok(vec_read
            .iter()
            .find(|m| m.normalized_string() == normalized.as_str())
            .cloned())
    }

    async fn list_all_mappings(&self) -> Result<Vec<PermissionMapping>> {
        let vec_read = self.mappings.read().await;
        Ok(vec_read.iter().cloned().collect())
    }
}

/// Normalize a permission name (trim + lowercase).
///
/// This function implements the same normalization logic used in
/// the PermissionId implementation to ensure consistency.
fn normalize_permission(input: &str) -> String {
    input.trim().to_lowercase()
}
