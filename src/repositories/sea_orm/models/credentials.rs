//! Credentials persistence model (SeaORM).
//!
//! Internal table storing the Argon2 hashed secret for an account.
//! Most users interact through `SeaOrmRepository`; import this only
//! for custom migrations or direct queries.
use crate::secrets::Secret;

use sea_orm::{ActiveValue, entity::prelude::*};

/// Credentials persistence entity (stores Argon2 hash).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "axum_gate_credentials")]
pub struct Model {
    /// Internal surrogate primary key.
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Owning account UUID.
    #[sea_orm(unique)]
    pub account_id: Uuid,
    /// Argon2 hashed secret (never plaintext).
    pub secret: String,
}

/// No declared relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Secret> for ActiveModel {
    fn from(value: Secret) -> Self {
        Self {
            id: ActiveValue::NotSet,
            account_id: ActiveValue::Set(value.account_id),
            secret: ActiveValue::Set(value.secret),
        }
    }
}
