//! Account management and user data structures.
//!
//! This module provides the core [`Account`] type and services for managing user accounts,
//! including creation, deletion, and repository abstractions for data persistence.
//!
//! # Quick Start
//!
//! ```rust
//! use axum_gate::accounts::{Account, AccountInsertService};
//! use axum_gate::prelude::{Role, Group};
//! use axum_gate::permissions::Permissions;
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository};
//! use std::sync::Arc;
//!
//! # tokio_test::block_on(async {
//! // Create repositories
//! let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
//! let secret_repo = Arc::new(MemorySecretRepository::new_with_argon2_hasher().unwrap());
//!
//! // Create a new account
//! let account = AccountInsertService::insert("user@example.com", "password")
//!     .with_roles(vec![Role::User, Role::Reporter])
//!     .with_groups(vec![Group::new("engineering"), Group::new("backend-team")])
//!     .with_permissions(Permissions::from_iter(["read:api", "write:docs"]))
//!     .into_repositories(account_repo, secret_repo)
//!     .await;
//! # });
//! ```

#[cfg(feature = "server")]
mod server_impl {
    pub use super::account_delete::AccountDeleteService;
    pub use super::account_insert::AccountInsertService;
    pub use super::account_repository::AccountRepository;
    pub use super::errors::{AccountOperation, AccountsError};
    #[cfg(any(feature = "storage-seaorm", feature = "storage-seaorm-v2"))]
    pub use crate::comma_separated_value::CommaSeparatedValue;
}

#[cfg(feature = "server")]
pub use server_impl::*;

use crate::authz::AccessHierarchy;
use crate::permissions::{PermissionId, Permissions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "server")]
mod account_delete;
#[cfg(feature = "server")]
mod account_insert;
#[cfg(feature = "server")]
mod account_repository;
#[cfg(feature = "server")]
pub mod errors;

/// An account contains authorization information about a user.
///
/// Accounts store user identification, roles, groups, and permissions. They are the
/// core entity for authorization decisions in axum-gate.
///
/// # Creating Accounts
///
/// ```rust
/// use axum_gate::accounts::Account;
/// use axum_gate::prelude::{Role, Group};
/// use axum_gate::permissions::Permissions;
///
/// // Create a basic account
/// let account = Account::new("user123", &[Role::User], &[Group::new("staff")]);
///
/// // Create account with permissions
/// let permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
/// let account = Account::<Role, Group>::new("admin@example.com", &[Role::Admin], &[])
///     .with_permissions(permissions);
/// ```
///
/// # Working with Permissions
///
/// ```rust
/// # use axum_gate::accounts::Account;
/// # use axum_gate::prelude::{Role, Group};
/// # use axum_gate::permissions::PermissionId;
/// # let mut account = Account::<Role, Group>::new("user", &[], &[]);
/// // Grant permissions
/// account.grant_permission("read:api");
/// account.grant_permission(PermissionId::from("write:api"));
///
/// // Check permissions directly
/// if account.permissions.has("read:api") {
///     println!("User can read API");
/// }
///
/// // Revoke permissions
/// account.revoke_permission("write:api");
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Account<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq + Clone,
{
    /// The unique identifier of the account generated during registration.
    ///
    /// This UUID links the account to its corresponding authentication secret
    /// in the secret repository. The separation of account data from secrets
    /// enhances security by allowing different storage backends and access controls.
    pub account_id: Uuid,
    /// The user identifier for this account (e.g., email, username).
    ///
    /// This should be unique within your application and is typically what users
    /// provide during login. It's used to look up accounts in the repository.
    pub user_id: String,
    /// Roles assigned to this account.
    ///
    /// Roles determine what actions a user can perform. If your roles implement
    /// `AccessHierarchy`, supervisor roles automatically inherit subordinate permissions.
    pub roles: Vec<R>,
    /// Groups this account belongs to.
    ///
    /// Groups provide another dimension of access control, allowing you to grant
    /// permissions based on team membership, department, or other organizational units.
    pub groups: Vec<G>,
    /// Custom permissions granted to this account.
    ///
    /// Uses a compressed bitmap for efficient storage and fast permission checks.
    /// Permissions are automatically available when referenced by name using
    /// deterministic hashing - no coordination between nodes required.
    pub permissions: Permissions,
}

impl<R, G> Account<R, G>
where
    R: AccessHierarchy + Eq + Clone,
    G: Eq + Clone,
{
    /// Creates a new account with the specified user ID, roles, and groups.
    ///
    /// A random UUID is automatically generated for the account ID. The account
    /// starts with no permissions - use `with_permissions()` or `grant_permission()`
    /// to add them.
    ///
    /// # Arguments
    /// * `user_id` - Unique identifier for the user (e.g., email or username)
    /// * `roles` - Roles to assign to this account
    /// * `groups` - Groups this account should belong to
    ///
    /// # Example
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    ///
    /// let account = Account::new(
    ///     "user@example.com",
    ///     &[Role::User, Role::Reporter],
    ///     &[Group::new("engineering"), Group::new("backend-team")]
    /// );
    /// ```
    pub fn new(user_id: &str, roles: &[R], groups: &[G]) -> Self {
        let roles = roles.to_vec();
        let groups = groups.to_vec();
        Self {
            account_id: Uuid::now_v7(),
            user_id: user_id.to_owned(),
            groups,
            roles,
            permissions: Permissions::new(),
        }
    }

    /// Creates a new account with the specified account ID.
    ///
    /// This constructor is primarily used internally when loading accounts from
    /// repositories. Most applications should use `new()` which generates a random ID.
    ///
    /// # Arguments
    /// * `account_id` - The UUID to use for this account
    /// * `user_id` - Unique identifier for the user
    /// * `roles` - Roles to assign to this account
    /// * `groups` - Groups this account should belong to
    #[cfg(any(feature = "storage-seaorm", feature = "storage-seaorm-v2"))]
    pub(crate) fn new_with_account_id(
        account_id: &Uuid,
        user_id: &str,
        roles: &[R],
        groups: &[G],
    ) -> Self {
        let roles = roles.to_vec();
        let groups = groups.to_vec();
        Self {
            account_id: account_id.to_owned(),
            user_id: user_id.to_owned(),
            groups,
            roles,
            permissions: Permissions::new(),
        }
    }

    /// Consumes this account and returns it with the specified permissions.
    ///
    /// This is useful when building accounts with specific permission sets.
    ///
    /// # Example
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    /// use axum_gate::permissions::Permissions;
    ///
    /// // Create permissions
    /// let permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// let account = Account::<Role, Group>::new("user@example.com", &[Role::User], &[])
    ///     .with_permissions(permissions);
    /// ```
    pub fn with_permissions(self, permissions: Permissions) -> Self {
        Self {
            permissions,
            ..self
        }
    }

    /// Grants a permission to this account.
    ///
    /// # Example
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    /// use axum_gate::permissions::PermissionId;
    ///
    /// let mut account = Account::<Role, Group>::new("user", &[], &[]);
    /// account.grant_permission("read:profile");
    /// account.grant_permission(PermissionId::from("write:profile"));
    /// ```
    pub fn grant_permission<P>(&mut self, permission: P)
    where
        P: Into<PermissionId>,
    {
        self.permissions.grant(permission);
    }

    /// Revokes a permission from this account.
    ///
    /// # Example
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    /// use axum_gate::permissions::PermissionId;
    ///
    /// let mut account = Account::<Role, Group>::new("user", &[], &[]);
    /// account.grant_permission("write:profile");
    /// account.revoke_permission(PermissionId::from("write:profile"));
    /// ```
    pub fn revoke_permission<P>(&mut self, permission: P)
    where
        P: Into<PermissionId>,
    {
        self.permissions.revoke(permission);
    }

    /// Returns true if this account has the given role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    ///
    /// let account = Account::<Role, Group>::new(
    ///     "user@example.com",
    ///     &[Role::User],
    ///     &[Group::new("engineering")]
    /// );
    ///
    /// assert!(account.has_role(&Role::User));
    /// assert!(!account.has_role(&Role::Admin));
    /// ```
    pub fn has_role(&self, role: &R) -> bool {
        self.roles.contains(role)
    }

    /// Returns true if this account is a member of the given group.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    ///
    /// let account = Account::<Role, Group>::new(
    ///     "user@example.com",
    ///     &[Role::User],
    ///     &[Group::new("engineering")]
    /// );
    ///
    /// assert!(account.is_member_of(&Group::new("engineering")));
    /// assert!(!account.is_member_of(&Group::new("marketing")));
    /// ```
    pub fn is_member_of(&self, group: &G) -> bool {
        self.groups.contains(group)
    }

    /// Returns true if this account has the specified permission.
    ///
    /// Accepts any type that converts into `PermissionId` (e.g., `&str`, `PermissionId`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_gate::accounts::Account;
    /// use axum_gate::prelude::{Role, Group};
    /// use axum_gate::permissions::PermissionId;
    ///
    /// let mut account = Account::<Role, Group>::new("user@example.com", &[], &[]);
    /// account.grant_permission("read:api");
    /// account.grant_permission(PermissionId::from("write:docs"));
    ///
    /// assert!(account.has_permission("read:api"));
    /// assert!(account.has_permission(PermissionId::from("write:docs")));
    /// assert!(!account.has_permission("admin:system"));
    /// ```
    pub fn has_permission<P>(&self, permission: P) -> bool
    where
        P: Into<PermissionId>,
    {
        self.permissions.has(permission)
    }
}

#[cfg(any(feature = "storage-seaorm", feature = "storage-seaorm-v2"))]
impl<R, G> TryFrom<crate::repositories::sea_orm::models::account::Model> for Account<R, G>
where
    R: AccessHierarchy + Eq + std::fmt::Display + Clone,
    Vec<R>: CommaSeparatedValue,
    G: Eq + Clone,
    Vec<G>: CommaSeparatedValue,
{
    type Error = String;

    fn try_from(
        value: crate::repositories::sea_orm::models::account::Model,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new_with_account_id(
            &value.account_id,
            &value.user_id,
            &Vec::<R>::from_csv(&value.roles)?,
            &Vec::<G>::from_csv(&value.groups)?,
        ))
    }
}
