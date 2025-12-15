//! Zero-synchronization permission system using deterministic hashing.
//!
//! This module provides a permission system where permission IDs are computed deterministically
//! from permission names using cryptographic hashing. This eliminates the need for synchronization
//! between distributed nodes while maintaining high performance through bitmap operations.
//!
//! # Key Features
//!
//! - **Deterministic hashing** - Same permission names always produce the same IDs
//! - **Zero synchronization** - No coordination needed between distributed nodes
//! - **Fast lookups** - Bitmap-based storage for O(1) permission checks
//! - **Collision detection** - Compile-time and runtime validation to prevent hash collisions
//!
//! # Using Permissions in Your Application
//!
//! ## 1. Validating Permissions at Compile Time
//!
//! ```rust
//! use axum_gate::permissions::PermissionId;
//! axum_gate::validate_permissions![
//!     "read:resource1",
//!     "write:resource1",
//!     "admin:system",
//! ];
//! ```
//!
//! ## 2. Working with Account Permissions
//!
//! ```rust
//! use axum_gate::permissions::{PermissionId, Permissions};
//! use axum_gate::accounts::Account;
//! use axum_gate::prelude::{Role, Group};
//!
//! let mut account = Account::<Role, Group>::new("user123", &[Role::User], &[Group::new("staff")]);
//!
//! // Grant permissions to an account
//! account.grant_permission("read:resource1");
//! account.grant_permission("write:resource1");
//!
//! // Check if account has permission
//! if account.permissions.has("read:resource1") {
//!     // Account has permission
//! }
//!
//! // Revoke permissions from an account
//! account.revoke_permission("write:resource1");
//! ```
//!
//! ## 3. Using Permissions with Access Policies
//!
//! ```rust
//! use axum_gate::authz::AccessPolicy;
//! use axum_gate::permissions::PermissionId;
//! use axum_gate::prelude::{Gate, Role, Group};
//! use axum_gate::codecs::jwt::{JsonWebToken, JwtClaims};
//! use axum_gate::accounts::Account;
//! use std::sync::Arc;
//! use axum::{routing::get, Router};
//!
//! # let jwt_codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! let app = Router::<()>::new()
//!     .route("/protected", get(protected_handler))
//!     .layer(
//!         Gate::cookie("issuer", jwt_codec)
//!             .with_policy(AccessPolicy::<Role, Group>::require_permission(
//!                 PermissionId::from("read:resource1")
//!             ))
//!     );
//!
//! async fn protected_handler() -> &'static str {
//!     "Access granted!"
//! }
//! ```
//!
//! ## 4. Working with Permission Collections
//!
//! ```rust
//! use axum_gate::permissions::Permissions;
//!
//! // Create permissions from an iterator
//! let permissions: Permissions = ["read:api", "write:api", "admin:system"].into_iter().collect();
//!
//! // Check for multiple permissions
//! if permissions.has_all(["read:api", "write:api"]) {
//!     println!("Has both read and write access");
//! }
//!
//! if permissions.has_any(["admin:system", "super:admin"]) {
//!     println!("Has admin access");
//! }
//! ```
//!
//! ## 5. Using Custom Enums with `AsPermissionName`
//!
//! Define your permissions as enums, implement `AsPermissionName` to map them to stable, readable names, and use them anywhere a permission is accepted.
//!
//! ```rust
//! use axum_gate::authz::AccessPolicy;
//! use axum_gate::permissions::{AsPermissionName, Permissions};
//! use axum_gate::prelude::{Role, Group};
//!
//! #[derive(Debug)]
//! enum Api {
//!     Read,
//!     Write,
//! }
//!
//! #[derive(Debug)]
//! enum AppPermission {
//!     Api(Api),
//!     System(&'static str),
//! }
//!
//! // Map enums to stable permission names used across your app
//! impl AsPermissionName for AppPermission {
//!     fn as_permission_name(&self) -> String {
//!         match self {
//!             AppPermission::Api(api) => format!("api:{:?}", api).to_lowercase(),
//!             AppPermission::System(s) => format!("system:{s}"),
//!         }
//!     }
//! }
//!
//! // Grant/check with your enums
//! let mut perms = Permissions::new();
//! perms.grant(&AppPermission::Api(Api::Read));
//! assert!(perms.has(&AppPermission::Api(Api::Read)));
//!
//! // Use in access policies
//! let policy: AccessPolicy<Role, Group> =
//!     AccessPolicy::require_permission(&AppPermission::Api(Api::Read));
//! ```

#[cfg(feature = "server")]
mod server_impl {
    pub use super::application_validator::ApplicationValidator;
    pub use super::collision_checker::PermissionCollisionChecker;
    pub use super::errors::PermissionsError;
    pub use super::permission_collision::PermissionCollision;
    pub use super::validation_report::ValidationReport;
}

#[cfg(feature = "server")]
pub use server_impl::*;

pub use self::as_permission_name::AsPermissionName;
pub use self::permission_id::PermissionId;
use roaring::RoaringTreemap;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "server")]
mod application_validator;
mod as_permission_name;
#[cfg(feature = "server")]
mod collision_checker;
#[cfg(feature = "server")]
pub mod errors;
pub mod mapping;
#[cfg(feature = "server")]
mod permission_collision;
mod permission_id;
#[cfg(feature = "server")]
pub mod validate_permissions;
#[cfg(feature = "server")]
mod validation_report;

/// A collection of permissions with efficient storage and fast operations.
///
/// Uses compressed bitmap storage internally for optimal memory usage and O(1)
/// permission checks. Designed for high-performance authorization systems that
/// need to handle thousands of permissions per user efficiently.
///
/// # Examples
///
/// ```rust
/// use axum_gate::permissions::Permissions;
///
/// // Create and populate permissions
/// let mut permissions = Permissions::new();
/// permissions
///     .grant("read:profile")
///     .grant("write:profile")
///     .grant("delete:profile");
///
/// // Check individual permissions
/// assert!(permissions.has("read:profile"));
/// assert!(!permissions.has("admin:users"));
///
/// // Check multiple permissions
/// assert!(permissions.has_all(["read:profile", "write:profile"]));
/// assert!(permissions.has_any(["read:profile", "admin:users"]));
/// ```
///
/// # Builder Pattern
///
/// ```rust
/// use axum_gate::permissions::Permissions;
///
/// let permissions = Permissions::new()
///     .with("read:api")
///     .with("write:api")
///     .build();
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Permissions {
    bitmap: RoaringTreemap,
}

impl Permissions {
    /// Creates a new empty permission set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let permissions = Permissions::new();
    /// assert!(permissions.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            bitmap: RoaringTreemap::new(),
        }
    }

    /// Grants a permission to this permission set.
    ///
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::{Permissions, PermissionId};
    ///
    /// let mut permissions = Permissions::new();
    /// permissions
    ///     .grant("read:profile")
    ///     .grant(PermissionId::from("write:profile"));
    ///
    /// assert!(permissions.has("read:profile"));
    /// assert!(permissions.has(PermissionId::from("write:profile")));
    /// ```
    pub fn grant<P>(&mut self, permission: P) -> &mut Self
    where
        P: Into<PermissionId>,
    {
        let permission_id = permission.into();
        self.bitmap.insert(permission_id.as_u64());
        self
    }

    /// Revokes a permission from this permission set.
    ///
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::{Permissions, PermissionId};
    ///
    /// let mut permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// permissions.revoke(PermissionId::from("write:profile"));
    ///
    /// assert!(permissions.has("read:profile"));
    /// assert!(!permissions.has("write:profile"));
    /// ```
    pub fn revoke<P>(&mut self, permission: P) -> &mut Self
    where
        P: Into<PermissionId>,
    {
        let permission_id = permission.into();
        self.bitmap.remove(permission_id.as_u64());
        self
    }

    /// Checks if a specific permission is granted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::{Permissions, PermissionId};
    ///
    /// let permissions: Permissions = ["read:profile"].into_iter().collect();
    ///
    /// assert!(permissions.has("read:profile"));
    /// assert!(permissions.has(PermissionId::from("read:profile")));
    /// assert!(!permissions.has("write:profile"));
    /// ```
    pub fn has<P>(&self, permission: P) -> bool
    where
        P: Into<PermissionId>,
    {
        let permission_id = permission.into();
        self.bitmap.contains(permission_id.as_u64())
    }

    /// Checks if all of the specified permissions are granted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::{Permissions, PermissionId};
    ///
    /// let permissions: Permissions = [
    ///     "read:profile",
    ///     "write:profile",
    ///     "read:posts",
    /// ].into_iter().collect();
    ///
    /// assert!(permissions.has_all(["read:profile", "write:profile"]));
    /// assert!(permissions.has_all([PermissionId::from("read:profile")]));
    /// assert!(!permissions.has_all(["read:profile", "admin:users"]));
    /// ```
    pub fn has_all<I, P>(&self, permissions: I) -> bool
    where
        I: IntoIterator<Item = P>,
        P: Into<PermissionId>,
    {
        permissions.into_iter().all(|p| self.has(p))
    }

    /// Checks if any of the specified permissions are granted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::{Permissions, PermissionId};
    ///
    /// let permissions: Permissions = ["read:profile"].into_iter().collect();
    ///
    /// assert!(permissions.has_any(["read:profile", "write:profile"]));
    /// assert!(permissions.has_any([PermissionId::from("read:profile")]));
    /// assert!(!permissions.has_any(["write:profile", "admin:users"]));
    /// ```
    pub fn has_any<I, P>(&self, permissions: I) -> bool
    where
        I: IntoIterator<Item = P>,
        P: Into<PermissionId>,
    {
        permissions.into_iter().any(|p| self.has(p))
    }

    /// Returns the number of permissions in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// assert_eq!(permissions.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.bitmap.len() as usize
    }

    /// Returns `true` if the permission set contains no permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let permissions = Permissions::new();
    /// assert!(permissions.is_empty());
    ///
    /// let mut permissions = Permissions::new();
    /// permissions.grant("read:profile");
    /// assert!(!permissions.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.bitmap.is_empty()
    }

    /// Removes all permissions from this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let mut permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// assert!(!permissions.is_empty());
    ///
    /// permissions.clear();
    /// assert!(permissions.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.bitmap.clear();
    }

    /// Computes the union of this permission set with another.
    ///
    /// This grants all permissions that exist in either set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let mut permissions1: Permissions = ["read:profile"].into_iter().collect();
    /// let permissions2: Permissions = ["write:profile"].into_iter().collect();
    ///
    /// permissions1.union(&permissions2);
    ///
    /// assert!(permissions1.has("read:profile"));
    /// assert!(permissions1.has("write:profile"));
    /// ```
    pub fn union(&mut self, other: &Permissions) -> &mut Self {
        self.bitmap |= &other.bitmap;
        self
    }

    /// Computes the intersection of this permission set with another.
    ///
    /// This keeps only permissions that exist in both sets.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let mut permissions1: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// let permissions2: Permissions = ["read:profile", "admin:users"].into_iter().collect();
    ///
    /// permissions1.intersection(&permissions2);
    ///
    /// assert!(permissions1.has("read:profile"));
    /// assert!(!permissions1.has("write:profile"));
    /// assert!(!permissions1.has("admin:users"));
    /// ```
    pub fn intersection(&mut self, other: &Permissions) -> &mut Self {
        self.bitmap &= &other.bitmap;
        self
    }

    /// Computes the difference of this permission set with another.
    ///
    /// This removes all permissions that exist in the other set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let mut permissions1: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// let permissions2: Permissions = ["write:profile"].into_iter().collect();
    ///
    /// permissions1.difference(&permissions2);
    ///
    /// assert!(permissions1.has("read:profile"));
    /// assert!(!permissions1.has("write:profile"));
    /// ```
    pub fn difference(&mut self, other: &Permissions) -> &mut Self {
        self.bitmap -= &other.bitmap;
        self
    }

    /// Builder method for granting a permission (immutable version).
    ///
    /// Use this when building permissions in a functional style or when you need
    /// to create permissions without mutable access. Prefer `grant()` for
    /// performance-critical code where you're modifying existing permission sets.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::{Permissions, PermissionId};
    ///
    /// let permissions = Permissions::new()
    ///     .with("read:profile")
    ///     .with(PermissionId::from("write:profile"))
    ///     .build();
    ///
    /// assert!(permissions.has("read:profile"));
    /// assert!(permissions.has("write:profile"));
    /// ```
    pub fn with<P>(mut self, permission: P) -> Self
    where
        P: Into<PermissionId>,
    {
        self.grant(permission);
        self
    }

    /// Finalizes the builder pattern.
    ///
    /// This method returns self unchanged, providing a clean conclusion to
    /// the builder pattern. Use this when you want to clearly signal the
    /// end of permission configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let permissions = Permissions::new()
    ///     .with("read:profile")
    ///     .with("write:profile")
    ///     .build();
    /// ```
    pub fn build(self) -> Self {
        self
    }

    /// Returns an iterator over the permission IDs in this collection.
    ///
    /// Use this when you need to examine all granted permissions or integrate
    /// with external systems that work with permission IDs directly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
    /// let ids: Vec<u64> = permissions.iter().collect();
    /// assert_eq!(ids.len(), 2);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.bitmap.iter()
    }

    /// Internal method for testing access.
    #[cfg(feature = "server")]
    pub(crate) fn bitmap_mut(&mut self) -> &mut roaring::RoaringTreemap {
        &mut self.bitmap
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Permissions({})", self.len())
    }
}

impl From<roaring::RoaringTreemap> for Permissions {
    fn from(bitmap: roaring::RoaringTreemap) -> Self {
        Self { bitmap }
    }
}

impl From<Permissions> for roaring::RoaringTreemap {
    fn from(permissions: Permissions) -> Self {
        permissions.bitmap
    }
}

impl AsRef<roaring::RoaringTreemap> for Permissions {
    fn as_ref(&self) -> &roaring::RoaringTreemap {
        &self.bitmap
    }
}

impl<P> std::iter::FromIterator<P> for Permissions
where
    P: Into<PermissionId>,
{
    /// Creates a permission set from an iterator of permission names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum_gate::permissions::Permissions;
    ///
    /// let permissions: Permissions = ["read:profile", "write:profile", "read:posts"]
    ///     .into_iter()
    ///     .collect();
    ///
    /// assert!(permissions.has("read:profile"));
    /// assert!(permissions.has("write:profile"));
    /// assert!(permissions.has("read:posts"));
    /// ```
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut perms = Self::new();
        for permission in iter {
            perms.grant(permission);
        }
        perms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_permissions_is_empty() {
        let permissions = Permissions::new();
        assert!(permissions.is_empty());
        assert_eq!(permissions.len(), 0);
    }

    #[test]
    fn grant_and_has_permission() {
        let mut permissions = Permissions::new();
        permissions.grant("read:profile");

        assert!(permissions.has("read:profile"));
        assert!(!permissions.has("write:profile"));
        assert_eq!(permissions.len(), 1);
        assert!(!permissions.is_empty());
    }

    #[test]
    fn grant_chaining() {
        let mut permissions = Permissions::new();
        permissions
            .grant("read:profile")
            .grant("write:profile")
            .grant("delete:profile");

        assert!(permissions.has("read:profile"));
        assert!(permissions.has("write:profile"));
        assert!(permissions.has("delete:profile"));
        assert_eq!(permissions.len(), 3);
    }

    #[test]
    fn revoke_permission() {
        let mut permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
        permissions.revoke("write:profile");

        assert!(permissions.has("read:profile"));
        assert!(!permissions.has("write:profile"));
        assert_eq!(permissions.len(), 1);
    }

    #[test]
    fn has_all_permissions() {
        let permissions: Permissions = ["read:profile", "write:profile", "read:posts"]
            .into_iter()
            .collect();

        assert!(permissions.has_all(["read:profile", "write:profile"]));
        assert!(permissions.has_all(["read:profile"]));
        assert!(!permissions.has_all(["read:profile", "admin:users"]));
        assert!(permissions.has_all(Vec::<&str>::new())); // empty set
    }

    #[test]
    fn has_any_permission() {
        let permissions: Permissions = ["read:profile"].into_iter().collect();

        assert!(permissions.has_any(["read:profile", "write:profile"]));
        assert!(permissions.has_any(["write:profile", "read:profile"]));
        assert!(!permissions.has_any(["write:profile", "admin:users"]));
        assert!(!permissions.has_any(Vec::<&str>::new())); // empty set
    }

    #[test]
    fn clear_permissions() {
        let mut permissions: Permissions = ["read:profile", "write:profile"].into_iter().collect();
        assert!(!permissions.is_empty());

        permissions.clear();
        assert!(permissions.is_empty());
        assert_eq!(permissions.len(), 0);
    }

    #[test]
    fn union_permissions() {
        let mut permissions1: Permissions = ["read:profile"].into_iter().collect();
        let permissions2: Permissions = ["write:profile", "read:posts"].into_iter().collect();

        permissions1.union(&permissions2);

        assert!(permissions1.has("read:profile"));
        assert!(permissions1.has("write:profile"));
        assert!(permissions1.has("read:posts"));
        assert_eq!(permissions1.len(), 3);
    }

    #[test]
    fn intersection_permissions() {
        let mut permissions1: Permissions = ["read:profile", "write:profile"].into_iter().collect();
        let permissions2: Permissions = ["read:profile", "admin:users"].into_iter().collect();

        permissions1.intersection(&permissions2);

        assert!(permissions1.has("read:profile"));
        assert!(!permissions1.has("write:profile"));
        assert!(!permissions1.has("admin:users"));
        assert_eq!(permissions1.len(), 1);
    }

    #[test]
    fn difference_permissions() {
        let mut permissions1 = Permissions::from_iter(["read:profile", "write:profile"]);
        let permissions2 = Permissions::from_iter(["write:profile"]);

        permissions1.difference(&permissions2);

        assert!(permissions1.has("read:profile"));
        assert!(!permissions1.has("write:profile"));
        assert_eq!(permissions1.len(), 1);
    }

    #[test]
    fn builder_pattern() {
        let permissions = Permissions::new()
            .with("read:profile")
            .with("write:profile")
            .build();

        assert!(permissions.has("read:profile"));
        assert!(permissions.has("write:profile"));
        assert_eq!(permissions.len(), 2);
    }

    #[test]
    fn from_iter() {
        let permissions: Permissions = ["read:profile", "write:profile", "read:posts"]
            .into_iter()
            .collect();

        assert!(permissions.has("read:profile"));
        assert!(permissions.has("write:profile"));
        assert!(permissions.has("read:posts"));
        assert_eq!(permissions.len(), 3);
    }

    #[test]
    fn permissions_are_deterministic() {
        let permissions1 = Permissions::from_iter(["read:profile"]);
        let permissions2 = Permissions::from_iter(["read:profile"]);

        assert_eq!(permissions1, permissions2);
        assert!(permissions1.has("read:profile"));
        assert!(permissions2.has("read:profile"));
    }

    #[test]
    fn display_implementation() {
        let permissions = Permissions::from_iter(["read:profile", "write:profile"]);
        let display = format!("{}", permissions);
        assert_eq!(display, "Permissions(2)");
    }
}
