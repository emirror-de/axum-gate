//! Storage implementations using the memory as backend.

use crate::hashing::{Argon2Hasher, VerificationResult};
use crate::secrets::Secret;
use crate::services::{AccountStorageService, CredentialsVerifierService, SecretStorageService};
use crate::utils::AccessHierarchy;
use crate::{Account, Credentials, Error};

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

/// A [MemoryAccountStorage] is a data structure where all [Account]s are stored in memory.
#[derive(Clone)]
pub struct MemoryAccountStorage<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq,
{
    accounts: Arc<RwLock<HashMap<String, Account<R, G>>>>,
}

impl<R, G> Default for MemoryAccountStorage<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq,
{
    fn default() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<R, G> From<Vec<Account<R, G>>> for MemoryAccountStorage<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq,
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

impl<R, G> AccountStorageService<R, G> for MemoryAccountStorage<R, G>
where
    Account<R, G>: Clone,
    R: AccessHierarchy + Eq,
    G: Eq,
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
}
/// Stores secrets in memory for authentication.
///
/// # Create and use a credential storage for authentication
/// ```rust
/// # tokio_test::block_on(async move {
/// # use axum_gate::Credentials;
/// # use axum_gate::hashing::{VerificationResult, Argon2Hasher};
/// # use axum_gate::services::SecretStorageService;
/// # use axum_gate::secrets::Secret;
/// # use axum_gate::storage::memory::MemorySecretStorage;
/// # use uuid::Uuid;
/// // The account id needs to be queried from an AccountStorageService.
/// // We generate it for this easy example.
/// let account_id = Uuid::now_v7();
/// let password = "admin_password";
/// let creds = Secret::new(&account_id, password, Argon2Hasher).unwrap();
/// // We can create a storage from a Vec<Secret>.
/// let creds_storage = MemorySecretStorage::try_from(vec![creds.clone()]).unwrap();
/// // We can add another secret.
/// let creds = Secret::new(&Uuid::now_v7(), "changed-admin-password", Argon2Hasher).unwrap();
/// creds_storage.store_secret(creds).await.unwrap();
/// let creds = Secret::new(&account_id, "changed-admin-password", Argon2Hasher).unwrap();
/// // We can update the secret in the storage.
/// creds_storage.update_secret(creds).await.unwrap();
/// // Or we can delete it if we want to.
/// creds_storage.delete_secret(&account_id).await.unwrap();
/// # });
/// ```
#[derive(Clone)]
pub struct MemorySecretStorage {
    store: Arc<RwLock<HashMap<Uuid, Secret>>>,
}

impl Default for MemorySecretStorage {
    fn default() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl From<Vec<Secret>> for MemorySecretStorage {
    fn from(value: Vec<Secret>) -> Self {
        let mut store = HashMap::with_capacity(value.len());
        value.into_iter().for_each(|v| {
            store.insert(v.account_id, v);
        });
        let store = Arc::new(RwLock::new(store));
        Self { store }
    }
}

impl SecretStorageService for MemorySecretStorage {
    async fn store_secret(&self, secret: Secret) -> Result<bool> {
        let already_present = {
            let read = self.store.read().await;
            read.contains_key(&secret.account_id)
        };

        if already_present {
            return Err(anyhow!(Error::SecretStorage(
                "AccountID is already present.".to_string()
            )));
        }

        let mut write = self.store.write().await;
        debug!("Got write lock on secret storage.");

        if write.insert(secret.account_id, secret).is_some() {
            return Err(anyhow!(Error::SecretStorage("This should never occur because it is checked if the key is already present a few lines earlier.".to_string())));
        };
        Ok(true)
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<bool> {
        let mut write = self.store.write().await;
        Ok(write.remove(id).is_some())
    }

    async fn update_secret(&self, secret: Secret) -> Result<()> {
        let mut write = self.store.write().await;
        write.insert(secret.account_id, secret);
        Ok(())
    }
}

impl CredentialsVerifierService<Uuid> for MemorySecretStorage {
    async fn verify_credentials(
        &self,
        credentials: Credentials<Uuid>,
    ) -> Result<VerificationResult> {
        let read = self.store.read().await;
        let Some(stored_secret) = read.get(&credentials.id) else {
            return Ok(VerificationResult::Unauthorized);
        };
        stored_secret.verify(&credentials.secret, Argon2Hasher)
    }
}
