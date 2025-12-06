//! Pre-defined hierarchical role system for access control.
//!
//! This module provides a built-in role system with four levels arranged in a hierarchy
//! where higher roles inherit access from lower roles. The hierarchy is:
//!
//! **Admin > Moderator > Reporter > User**
//!
//! # Role Hierarchy
//!
//! When using `AccessPolicy::require_role_or_supervisor()`, higher roles automatically
//! inherit access from lower roles:
//!
//! ```rust
//! use axum_gate::prelude::Role;
//! use axum_gate::authz::AccessPolicy;
//! use axum_gate::prelude::Group;
//!
//! // Allow User role and all supervisor roles (Reporter, Moderator, Admin)
//! let policy = AccessPolicy::<Role, Group>::require_role_or_supervisor(Role::User);
//!
//! // Allow only Moderator role and supervisor roles (Admin)
//! let policy = AccessPolicy::<Role, Group>::require_role_or_supervisor(Role::Moderator);
//!
//! // Allow only Admin role (no supervisors above Admin)
//! let policy = AccessPolicy::<Role, Group>::require_role_or_supervisor(Role::Admin);
//! ```
//!
//! # Using Roles with Gates
//!
//! ```rust
//! use axum_gate::prelude::*;
//! use axum_gate::authz::AccessPolicy;
//! use axum_gate::codecs::jwt::{JsonWebToken, JwtClaims};
//! use axum_gate::accounts::Account;
//! use std::sync::Arc;
//!
//! # let jwt_codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! // Exact role match (only Admin)
//! let admin_gate = Gate::cookie("my-app", Arc::clone(&jwt_codec))
//!     .with_policy(AccessPolicy::<Role, Group>::require_role(Role::Admin));
//!
//! // Multiple specific roles
//! let staff_gate = Gate::cookie("my-app", Arc::clone(&jwt_codec))
//!     .with_policy(
//!         AccessPolicy::<Role, Group>::require_role(Role::Admin)
//!             .or_require_role(Role::Moderator)
//!     );
//!
//! // Hierarchical access (User + all supervisors)
//! let user_gate = Gate::cookie("my-app", jwt_codec)
//!     .with_policy(AccessPolicy::<Role, Group>::require_role_or_supervisor(Role::User));
//! ```
//!
//! # Creating Custom Roles
//!
//! Create your own role hierarchy by implementing the required traits:
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use axum_gate::authz::AccessHierarchy;
//!
//! #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
//! enum CompanyRole {
//!     #[default]
//!     Employee,      // Lowest privilege
//!     TeamLead,
//!     Manager,
//!     Director,      // Highest privilege
//! }
//!
//! impl std::fmt::Display for CompanyRole {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             CompanyRole::Employee => write!(f, "Employee"),
//!             CompanyRole::TeamLead => write!(f, "TeamLead"),
//!             CompanyRole::Manager => write!(f, "Manager"),
//!             CompanyRole::Director => write!(f, "Director"),
//!         }
//!     }
//! }
//!
//! impl AccessHierarchy for CompanyRole {}
//! ```

use crate::authz::AccessHierarchy;
#[cfg(all(feature = "server", feature = "storage-seaorm"))]
use crate::comma_separated_value::CommaSeparatedValue;
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "server", feature = "storage-seaorm"))]
use std::str::FromStr;

/// Pre-defined roles with hierarchical access control.
///
/// These roles are arranged in a hierarchy where higher roles automatically
/// inherit access from lower roles when using `AccessPolicy::require_role_or_supervisor()`.
///
/// **Hierarchy (highest to lowest):**
/// - `Admin` - Full system access
/// - `Moderator` - Content moderation and user management
/// - `Reporter` - Read access with reporting capabilities
/// - `User` - Basic user access
///
/** Updated example imports to reflect current public modules */
/// # Example Usage
/// ```rust
/// use axum_gate::prelude::{Role, Group};
/// use axum_gate::authz::AccessPolicy;
///
/// // Grant access to Moderators and all supervisor roles (Admin)
/// let policy = AccessPolicy::<Role, Group>::require_role_or_supervisor(Role::Moderator);
///
/// // Grant access to specific roles only
/// let policy = AccessPolicy::<Role, Group>::require_role(Role::Admin)
///     .or_require_role(Role::Moderator);
/// ```
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
)]
pub enum Role {
    /// Basic user role with standard application access.
    ///
    /// Users have access to core application features but limited
    /// administrative capabilities.
    #[default]
    User,
    /// Reporter role with read access and reporting capabilities.
    ///
    /// Reporters can typically view system information, generate reports,
    /// and access analytics data.
    Reporter,
    /// Moderator role with elevated privileges for content and user management.
    ///
    /// Moderators can typically manage content, moderate discussions,
    /// and have elevated access to user-facing features.
    Moderator,
    /// Administrator role with the highest level of access.
    ///
    /// Administrators typically have full system access and can perform
    /// any operation within the application.
    Admin,
}

impl AccessHierarchy for Role {}

#[cfg(all(feature = "server", feature = "storage-seaorm"))]
impl CommaSeparatedValue for Vec<Role> {
    fn from_csv(value: &str) -> Result<Self, String> {
        let mut role_str = value.split(',').collect::<Vec<&str>>();
        let mut roles = Vec::with_capacity(role_str.len());
        while let Some(r) = role_str.pop() {
            roles.push(Role::from_str(r).map_err(|e| e.to_string())?);
        }
        Ok(roles)
    }

    fn into_csv(self) -> String {
        self.into_iter()
            .map(|g| g.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}
