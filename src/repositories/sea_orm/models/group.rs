//! SeaORM model for persisted groups.
//!
//! This model stores a JSON-serialized payload for a domain group entity along
//! with a stable `group_id` used as the unique identifier. The payload field
//! allows storing arbitrary domain group types as long as they implement
//! `Serialize`/`Deserialize` and the `GroupEntity` trait (which provides
//! `group_id()`).

use crate::groups::GroupEntity;
#[cfg(feature = "storage-seaorm")]
use sea_orm::{ActiveValue, entity::prelude::*};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// SeaORM entity for a persisted group.
///
/// - `group_id` is a stable, unique identifier used as a lookup key.
/// - `payload` contains the JSON serialized representation of the domain group
///   type T (T must implement `Serialize` and `GroupEntity`).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "axum_gate_groups")]
pub struct Model {
    /// Surrogate primary key (auto‑increment).
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Logical, unique group identifier (provided by domain type via `group_id()`).
    #[sea_orm(unique)]
    pub group_id: String,
    /// JSON-serialized payload of the domain group type.
    pub payload: String,
}

/// No declared relations for this entity.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl<T> From<T> for ActiveModel
where
    T: Serialize + GroupEntity,
{
    fn from(value: T) -> Self {
        // Serialize payload to JSON. Infallible fallback to empty string keeps
        // the conversion ergonomic; callers/DB should validate payload semantics.
        let payload = serde_json::to_string(&value).unwrap_or_default();
        Self {
            id: ActiveValue::NotSet,
            group_id: ActiveValue::Set(value.group_id().to_string()),
            payload: ActiveValue::Set(payload),
        }
    }
}

impl Model {
    /// Deserialize the stored JSON payload into a domain type `T`.
    ///
    /// Consumes the model and returns a typed domain value or an error string.
    pub fn into_payload<T>(self) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(&self.payload)
            .map_err(|e| format!("failed to deserialize group payload: {}", e))
    }

    /// Deserialize the stored JSON payload by reference into a domain type `T`.
    ///
    /// Does not consume the model.
    pub fn payload_as<T>(&self) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(&self.payload)
            .map_err(|e| format!("failed to deserialize group payload: {}", e))
    }
}
