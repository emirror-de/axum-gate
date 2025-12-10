use super::{Account, AccountRepository};
use crate::accounts::{AccountOperation, AccountsError};
use crate::authz::AccessHierarchy;
use crate::errors::Error;
use crate::secrets::SecretRepository;

use std::sync::Arc;

#[cfg(feature = "audit-logging")]
use crate::audit;
use crate::errors::Result;
use tracing::{debug, error, info, warn};

/// Removes the given account and its corresponding secret from repositories.
/// Implements a compensating action: if account deletion fails after removing
/// the secret, the secret is restored (best-effort) and an error is returned.
pub struct AccountDeleteService<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq + Clone,
{
    account: Account<R, G>,
}

impl<R, G> AccountDeleteService<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq + Clone,
{
    /// Creates a deletion service for the given account.
    ///
    /// This constructor is side-effect free; it does not touch any repositories.
    /// Invoke [`from_repositories`](Self::from_repositories) to perform the actual deletion with
    /// compensating secret restoration if the account removal fails.
    pub fn delete(account: Account<R, G>) -> Self {
        Self { account }
    }

    /// Performs the deletion workflow against the provided repositories.
    ///
    /// Workflow:
    /// 1. Removes (and returns) the secret associated with the account, caching it locally.
    /// 2. Attempts to delete the account.
    /// 3. If the account deletion fails, a compensating action re-inserts the cached secret
    ///    (best-effort) and an error is returned describing the restoration result.
    ///
    /// Returns:
    /// - Ok(()) if both secret and account are deleted successfully.
    /// - Err(_) if either removing the secret failed (None returned) or account deletion
    ///   failed (with secret restoration attempted).
    pub async fn from_repositories<AccRepo, SecRepo>(
        self,
        account_repository: Arc<AccRepo>,
        secret_repository: Arc<SecRepo>,
    ) -> Result<()>
    where
        AccRepo: AccountRepository<R, G>,
        SecRepo: SecretRepository,
    {
        let user_id = &self.account.user_id;
        let account_id = &self.account.account_id;

        // Remove and cache the secret so it can be restored if account deletion fails.
        info!(%user_id, %account_id, "Starting account deletion");
        #[cfg(feature = "audit-logging")]
        audit::account_delete_start(user_id, account_id);
        let Some(secret) = secret_repository.delete_secret(account_id).await? else {
            error!(%user_id, %account_id, "Secret missing for account deletion attempt");
            #[cfg(feature = "audit-logging")]
            audit::account_delete_failure(user_id, account_id, None, "secret_missing");
            return Err(Error::Accounts(AccountsError::operation(
                AccountOperation::Delete,
                "Secret not found",
                Some(account_id.to_string()),
            )));
        };
        debug!(%user_id, %account_id, "Secret removed for account");

        // Delete the account second. If this fails, attempt to restore the secret.
        // Use the stable internal `account_id` (UUID) for repository deletion so storage
        // backends operate on the immutable PK and secrets can be referenced reliably.
        if account_repository
            .delete_account(account_id)
            .await?
            .is_none()
        {
            error!(%user_id, %account_id, "Account deletion failed; attempting secret restore");
            let restore_result = secret_repository.store_secret(secret).await;
            match restore_result {
                Ok(true) => {
                    warn!(%user_id, %account_id, "Secret restored after account deletion failure");
                    #[cfg(feature = "audit-logging")]
                    audit::account_delete_failure(
                        user_id,
                        account_id,
                        Some(true),
                        "account_deletion_failed_secret_restored",
                    );
                }
                Ok(false) => {
                    error!(%user_id, %account_id, "Secret restore reported false after account deletion failure");
                    #[cfg(feature = "audit-logging")]
                    audit::account_delete_failure(
                        user_id,
                        account_id,
                        Some(false),
                        "account_deletion_failed_restore_reported_false",
                    );
                }
                Err(ref e) => {
                    error!(error = %e, %user_id, %account_id, "Secret restore failed after account deletion failure");
                    #[cfg(feature = "audit-logging")]
                    audit::account_delete_failure(
                        user_id,
                        account_id,
                        None,
                        &format!("account_deletion_failed_secret_restore_error: {}", e),
                    );
                }
            }

            return Err(Error::Accounts(AccountsError::operation(
                AccountOperation::Delete,
                "Account deletion failed",
                Some(account_id.to_string()),
            )));
        }

        info!(%user_id, %account_id, "Account deletion succeeded");
        #[cfg(feature = "audit-logging")]
        audit::account_delete_success(user_id, account_id);
        Ok(())
    }
}
