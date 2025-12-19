use crate::accounts::Account;
use crate::accounts::AccountRepository;
use crate::authz::AccessHierarchy;
use crate::errors::Result;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
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
    /// Lookup by the logical login identifier (`user_id`).
    ///
    /// The in-memory store's primary key is the stable `account_id` (UUID string).
    /// To fetch by `user_id` we scan the values; this is acceptable for tests and
    /// small datasets but should not be used as a model for production storage.
    async fn query_account_by_user_id(&self, user_id: &str) -> Result<Option<Account<R, G>>> {
        let read = self.accounts.read().await;
        for acc in read.values() {
            if acc.user_id == user_id {
                return Ok(Some(acc.clone()));
            }
        }
        Ok(None)
    }

    /// Query an account by its `account_id` field.
    ///
    /// The in-memory repository stores accounts keyed by the stable `account_id`
    /// (UUID string). This makes direct lookups efficient.
    async fn query_account_by_id(&self, account_id: &Uuid) -> Result<Option<Account<R, G>>> {
        let read = self.accounts.read().await;
        let key = account_id.to_string();
        Ok(read.get(&key).cloned())
    }

    /// Store an account using the stable `account_id` as the map key while
    /// preserving the `user_id` field inside the `Account`.
    async fn store_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        let id = account.account_id.to_string();
        let mut write = self.accounts.write().await;
        write.insert(id, account.clone());
        Ok(Some(account))
    }

    /// Delete an account by its stable `account_id` (UUID).
    async fn delete_account(&self, account_id: &Uuid) -> Result<Option<Account<R, G>>> {
        let mut write = self.accounts.write().await;
        let key = account_id.to_string();
        if !write.contains_key(&key) {
            return Ok(None);
        }
        Ok(write.remove(&key))
    }

    async fn update_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        // Reuse store semantics: upsert by account_id
        self.store_account(account).await
    }

    async fn query_all_accounts(&self) -> Result<Vec<Account<R, G>>> {
        let read = self.accounts.read().await;
        // Collect cloned account values into a Vec
        Ok(read.values().cloned().collect())
    }
}
