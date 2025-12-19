use crate::credentials::Credentials;
use crate::credentials::CredentialsVerifier;
use crate::errors::{Error, Result};
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;
use crate::repositories::{RepositoriesError, RepositoryOperation, RepositoryType};
use crate::secrets::Secret;
use crate::secrets::SecretRepository;
use crate::verification_result::VerificationResult;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

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
