use crate::errors::Result;
use crate::{accounts::Account, authz::AccessHierarchy};

use std::future::Future;

/// Repository abstraction for persisting and retrieving [`Account`] entities.
///
/// This trait is implemented by storage backends (e.g. in‑memory, SurrealDB, SeaORM).
/// It deliberately uses an `Option<Account<..>>` in results for operations where
/// absence is a normal outcome (delete / update / query) so callers can
/// distinguish “not found” from actual errors (`Result::Err`).
///
/// # Semantics
///
/// | Method                | Success (`Ok`) Return Value                        | Typical `None` Meaning                    | Error (`Err`) Meaning                            |
/// |-----------------------|----------------------------------------------------|-------------------------------------------|--------------------------------------------------|
/// | `store_account`       | `Some(Account)` if stored                          | `None` only if backend chooses (rare)     | Persistence / connectivity / constraint failure  |
/// | `delete_account`      | `Some(Account)` = deleted & returned               | `None` = no account with that user id     | Backend / IO failure                             |
/// | `update_account`      | `Some(Account)` = updated                          | `None` = no existing account to update    | Backend / IO / optimistic concurrency failure    |
/// | `query_account_by_user_id` | `Some(Account)` = found                      | `None` = not found                        | Backend / IO failure                             |
/// | `query_all_accounts`  | `Vec<Account>` containing zero or more accounts    | N/A                                       | Backend / IO failure                             |
///
/// Backends SHOULD:
/// - Treat `user_id` as a logical unique key
/// - Enforce uniqueness at storage level where possible
/// - Return **identical timing characteristics** for “found” vs “not found” where feasible
///   (helps upstream login logic resist user enumeration timing attacks)
///
/// # Concurrency & Consistency
///
/// This trait does not prescribe isolation semantics. Implementations should document:
/// - Whether updates are last‑write‑wins
/// - Whether optimistic locking / versioning is applied
///
/// # Example (generic usage)
/// ```rust
///
/// use axum_gate::accounts::Account;
/// use axum_gate::prelude::{Role, Group};
/// use axum_gate::accounts::AccountRepository;
/// use axum_gate::repositories::memory::MemoryAccountRepository;
///
/// # #[tokio::test]
/// async fn load_or_create(
///     repo: &MemoryAccountRepository<Role, Group>,
///     template: Account<Role, Group>
/// ) -> axum_gate::errors::Result<Account<Role, Group>> {
///     if let Some(existing) = repo.query_account_by_user_id(&template.user_id).await? {
///         Ok(existing)
///     } else {
///         Ok(repo.store_account(template).await?.expect("store returned None"))
///     }
/// }
/// ```
///
/// # Error Handling
///
/// Return `Err` only for exceptional backend failures (connectivity, serialization,
/// constraint violation, etc.). Use `Ok(None)` for “not found” / “no-op” outcomes.
///
/// # Extensibility
///
/// If you add methods (e.g. pagination, search), prefer separate traits to avoid forcing
/// all backends to implement optional features.
pub trait AccountRepository<R, G>
where
    Self: Send + Sync,
    R: AccessHierarchy + Eq,
    G: Eq + Clone,
{
    /// Persist a new account.
    ///
    /// Implementations SHOULD enforce uniqueness of `user_id`. Returning `Ok(Some(account))`
    /// indicates success. Returning `Ok(None)` is discouraged unless there is a documented
    /// race / conditional insert semantics the backend wishes to expose.
    fn store_account(
        &self,
        account: Account<R, G>,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Delete an account identified by its `user_id`.
    ///
    /// Returns:
    /// - `Ok(Some(account))` if the account existed and was removed
    /// - `Ok(None)` if no account matched `user_id`
    /// - `Err(e)` on backend error
    fn delete_account(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Update an existing account.
    ///
    /// Implementations may perform either full replacement or partial persistence depending
    /// on backend capabilities (document if non‑standard). Returns:
    /// - `Ok(Some(updated_account))` on success
    /// - `Ok(None)` if the account does not exist
    /// - `Err(e)` on failure
    fn update_account(
        &self,
        account: Account<R, G>,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Fetch an account by its logical user identifier.
    ///
    /// This must **not** leak timing differences exploitable for enumeration if used
    /// together with authentication flows relying on indistinguishable “not found”.
    ///
    /// Returns:
    /// - `Ok(Some(account))` if found
    /// - `Ok(None)` if not found
    /// - `Err(e)` on backend failure
    fn query_account_by_user_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Fetch an account by its stable account identifier (`account_id`).
    ///
    /// Backends SHOULD treat `account_id` as an alternate stable unique key and
    /// SHOULD provide consistent timing characteristics with `query_account_by_user_id`
    /// to avoid user/ID enumeration via timing attacks.
    ///
    /// Returns:
    /// - `Ok(Some(account))` if found
    /// - `Ok(None)` if not found
    /// - `Err(e)` on backend failure
    fn query_account_by_id(
        &self,
        account_id: &str,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Query all accounts in the repository.
    ///
    /// Returns:
    /// - `Ok(vec)` containing zero or more accounts on success
    /// - `Err(e)` on backend failure
    ///
    /// Note: backends SHOULD document ordering semantics (if any). For large datasets,
    /// implementors MAY provide a separate paginated trait to avoid forcing all backends
    /// to materialize the entire dataset in memory.
    fn query_all_accounts(&self) -> impl Future<Output = Result<Vec<Account<R, G>>>> + Send;
}
