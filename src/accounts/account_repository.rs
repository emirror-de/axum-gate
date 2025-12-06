use crate::errors::Result;
use crate::{accounts::Account, authz::AccessHierarchy};

use std::future::Future;
use uuid::Uuid;

/// Repository abstraction for persisting and retrieving `Account` entities.
///
/// This trait is intentionally small and focused on the basic operations required
/// by the rest of the application: create, update, delete and lookups by either
/// the logical login identifier (`user_id`) or the stable internal identifier
/// (`account_id`).
///
/// Key design points:
/// - `user_id` is treated as the logical login identifier (email / username).
///   It is useful for authentication and user-facing flows and therefore there
///   are query methods that accept `&str` for that purpose.
/// - `account_id` is treated as the stable internal identifier (UUID). It is
///   the recommended identifier for persistence operations (deletes, secret
///   storage, cross-table references). To reduce accidental coupling to mutable
///   login identifiers, the repository API accepts `&Uuid` for `query_account_by_id`
///   and `delete_account`.
/// - Implementations SHOULD enforce uniqueness of `user_id` at the storage
///   layer where possible and SHOULD document timing characteristics for lookups
///   when used in authentication flows (to avoid user enumeration via timing).
///
/// Error handling:
/// - Return `Ok(Some(account))` on successful materialization of an account.
/// - Return `Ok(None)` when the requested account does not exist (not an error).
/// - Return `Err(..)` for exceptional backend failures (I/O, serialization, constraint violation).
pub trait AccountRepository<R, G>
where
    Self: Send + Sync,
    R: AccessHierarchy + Eq,
    G: Eq + Clone,
{
    /// Persist a new account.
    ///
    /// Implementations SHOULD enforce uniqueness of `user_id`. Returning
    /// `Ok(Some(account))` indicates success. Returning `Ok(None)` is
    /// discouraged unless the backend intentionally exposes conditional insert
    /// semantics.
    fn store_account(
        &self,
        account: Account<R, G>,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Delete an account identified by its stable `account_id` (UUID).
    ///
    /// Returns:
    /// - `Ok(Some(account))` if the account existed and was removed
    /// - `Ok(None)` if no account matched `account_id`
    /// - `Err(e)` on backend error
    fn delete_account(
        &self,
        account_id: &Uuid,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Update an existing account.
    ///
    /// Implementations may perform either full replacement or partial persistence
    /// depending on backend capabilities (document non-standard behavior).
    /// Returns:
    /// - `Ok(Some(updated_account))` on success
    /// - `Ok(None)` if the account does not exist
    /// - `Err(e)` on failure
    fn update_account(
        &self,
        account: Account<R, G>,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Fetch an account by its logical user identifier (`user_id`).
    ///
    /// This lookup is commonly used during authentication flows. Implementations
    /// SHOULD take care to avoid leaking timing differences that could be used
    /// to enumerate existing user_ids.
    fn query_account_by_user_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Fetch an account by its stable internal identifier (`account_id` / UUID).
    ///
    /// This is the recommended lookup form for operations that must operate on
    /// the canonical, immutable account identifier (deletions, secret operations,
    /// cross-table references).
    fn query_account_by_id(
        &self,
        account_id: &Uuid,
    ) -> impl Future<Output = Result<Option<Account<R, G>>>> + Send;

    /// Query all accounts in the repository.
    ///
    /// Implementations SHOULD document ordering semantics if any. For large
    /// datasets consider offering a paginated variant rather than returning all
    /// accounts in memory.
    fn query_all_accounts(&self) -> impl Future<Output = Result<Vec<Account<R, G>>>> + Send;
}
