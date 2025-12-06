//! Permission mapping persistence model (SeaORM).
//!
//! This entity stores a reverse-lookup registry from a normalized permission
//! string to its deterministic 64-bit permission identifier. It supports the
//! optional "permission registry" feature for human-readable lookups while
//! keeping the primary bitmap-based permission system unchanged.
//!
//! Notes:
//! - `normalized_string` is the trimmed + lowercased permission string.
//! - `permission_id` is stored as a string (u64 rendered as decimal) to avoid
//!   signed/unsigned integer portability issues across backends.
//!
//! See also the domain value: `crate::domain::values::PermissionMapping`.

use crate::permissions::PermissionId;
use crate::permissions::mapping::PermissionMapping;
#[cfg(feature = "storage-seaorm")]
use sea_orm::{ActiveValue, entity::prelude::*};

/// SeaORM entity for a permission mapping (normalized string <-> id).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "axum_gate_permission_mappings")]
pub struct Model {
    /// Surrogate primary key (auto-increment).
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Normalized permission string (trimmed + lowercased).
    #[sea_orm(unique)]
    pub normalized_string: String,
    /// Deterministic permission id (u64 rendered as base-10 string).
    #[sea_orm(unique)]
    pub permission_id: String,
}

/// No declared relations for this entity.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<PermissionMapping> for ActiveModel {
    fn from(value: PermissionMapping) -> Self {
        Self {
            id: ActiveValue::NotSet,
            normalized_string: ActiveValue::Set(value.normalized_string().to_string()),
            permission_id: ActiveValue::Set(value.permission_id().as_u64().to_string()),
        }
    }
}

impl TryFrom<Model> for PermissionMapping {
    type Error = String;

    fn try_from(value: Model) -> Result<Self, Self::Error> {
        // Parse the stored permission_id string back to a u64
        let id_u64 = value
            .permission_id
            .parse::<u64>()
            .map_err(|e| format!("invalid permission_id '{}': {}", value.permission_id, e))?;

        let id = PermissionId::from_u64(id_u64);

        // Reconstruct and validate the domain mapping; this guarantees
        // consistency between the normalized string and the id.
        PermissionMapping::new(value.normalized_string, id)
            .map_err(|e| format!("failed to construct PermissionMapping: {}", e))
    }
}
