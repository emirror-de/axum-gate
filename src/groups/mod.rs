//! Group-based access control for organizing users into logical collections.
//!
//! Groups provide a flexible way to organize users into collections for access control
//! purposes. Unlike hierarchical roles, groups are typically used for organizational
//! structures like departments, teams, projects, or any logical grouping of users.
//!
//! # Key Features
//!
//! - **Organizational structure** - Model departments, teams, or project memberships
//! - **Flexible membership** - Users can belong to multiple groups simultaneously
//! - **Simple equality** - Groups are compared by name (no hierarchy)
//! - **Access policies** - Use groups in access control policies alongside roles
//!
//! # Creating and Using Groups
//!
//! ```rust
//! use axum_gate::prelude::Group;
//! use axum_gate::authz::AccessPolicy;
//! use axum_gate::prelude::Role;
//!
//! // Create groups for different organizational units
//! let engineering = Group::new("engineering");
//! let marketing = Group::new("marketing");
//! let qa_team = Group::new("qa-team");
//! let project_alpha = Group::new("project-alpha");
//!
//! // Use groups in access policies
//! let policy = AccessPolicy::<Role, Group>::require_group(engineering)
//!     .or_require_group(marketing);
//! ```
//!
//! # Group-Based Access Control
//!
//! ```rust
//! use axum_gate::accounts::Account;
//! use axum_gate::authz::AccessPolicy;
//! use axum_gate::prelude::{Gate, Role, Group};
//! use axum_gate::codecs::jwt::{JsonWebToken, JwtClaims};
//! use std::sync::Arc;
//!
//! // Create an account with multiple group memberships
//! let account = Account::new(
//!     "developer@example.com",
//!     &[Role::User],
//!     &[Group::new("engineering"), Group::new("backend-team"), Group::new("project-alpha")]
//! );
//!
//! // Create access policies for different areas
//! # let jwt_codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! let engineering_gate = Gate::cookie("my-app", Arc::clone(&jwt_codec))
//!     .with_policy(AccessPolicy::<Role, Group>::require_group(Group::new("engineering")));
//!
//! let project_gate = Gate::cookie("my-app", Arc::clone(&jwt_codec))
//!     .with_policy(
//!         AccessPolicy::<Role, Group>::require_group(Group::new("project-alpha"))
//!             .or_require_group(Group::new("project-beta"))
//!     );
//!
//! // Combine with role requirements
//! let admin_or_engineering = Gate::cookie("my-app", jwt_codec)
//!     .with_policy(
//!         AccessPolicy::<Role, Group>::require_role(Role::Admin)
//!             .or_require_group(Group::new("engineering"))
//!     );
//! ```
//!
//! # Common Group Patterns
//!
//! ```rust
//! use axum_gate::prelude::Group;
//!
//! // Department-based groups
//! let groups = vec![
//!     Group::new("engineering"),
//!     Group::new("marketing"),
//!     Group::new("sales"),
//!     Group::new("support"),
//! ];
//!
//! // Project-based groups
//! let project_groups = vec![
//!     Group::new("project-alpha"),
//!     Group::new("project-beta"),
//!     Group::new("maintenance"),
//! ];
//!
//! // Team-based groups
//! let team_groups = vec![
//!     Group::new("frontend-team"),
//!     Group::new("backend-team"),
//!     Group::new("devops-team"),
//!     Group::new("qa-team"),
//! ];
//! ```

#[cfg(all(
    feature = "server",
    any(feature = "storage-seaorm", feature = "storage-seaorm-v2")
))]
use crate::comma_separated_value::CommaSeparatedValue;
#[cfg(feature = "server")]
pub use group_repository::GroupRepository;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
mod group_repository;

/// A group represents a collection of users for access control purposes.
///
/// Groups are typically used to represent organizational units like departments,
/// teams, projects, or any other logical grouping of users. They provide an
/// additional dimension of access control beyond roles.
///
/// # Example Usage
/// ```rust
/// use axum_gate::prelude::Group;
///
/// let engineering = Group::new("engineering");
/// let backend_team = Group::new("backend-team");
/// let project_alpha = Group::new("project-alpha");
///
/// println!("Group name: {}", engineering.name());
/// ```
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Group(String);

impl Group {
    /// Creates a new group with the specified name.
    ///
    /// # Arguments
    /// * `group` - The name of the group (e.g., "engineering", "marketing", "admin")
    ///
    /// # Example
    /// ```rust
    /// use axum_gate::prelude::Group;
    ///
    /// let engineering = Group::new("engineering");
    /// let marketing = Group::new("marketing");
    /// let project_team = Group::new("project-alpha-team");
    /// ```
    pub fn new(group: &str) -> Self {
        Self(group.to_string())
    }

    /// Returns the name of this group.
    ///
    /// # Example
    /// ```rust
    /// use axum_gate::prelude::Group;
    ///
    /// let group = Group::new("engineering");
    /// assert_eq!(group.name(), "engineering");
    /// ```
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Trait implemented by types that can act as a group entity inside repositories.
///
/// Implement this for your domain group type so the repository can use the
/// `group_id()` as the unique identifier. The method returns a string slice
/// identifying the group (for example a name or a stable id).
pub trait GroupEntity {
    /// Return the unique identifier for this group as `&str`.
    fn group_id(&self) -> &str;
}

/// Implement the `GroupEntity` trait for the crate's built-in `Group` type so
/// the concrete `Group` can be used directly with `GroupRepository<T>`.
impl GroupEntity for Group {
    fn group_id(&self) -> &str {
        self.name()
    }
}

#[cfg(all(
    feature = "server",
    any(feature = "storage-seaorm", feature = "storage-seaorm-v2")
))]
impl CommaSeparatedValue for Vec<Group> {
    fn from_csv(value: &str) -> Result<Self, String> {
        Ok(value
            .split(',')
            .collect::<Vec<&str>>()
            .iter()
            .map(|g| Group::new(g))
            .collect())
    }

    fn into_csv(self) -> String {
        self.into_iter()
            .map(|g| g.name().to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}
