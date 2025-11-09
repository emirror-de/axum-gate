//! Account persistence model (SeaORM).
//!
//! Internal database representation of a domain `Account<R, G>` storing user id,
//! groups and roles as comma separated strings. Most users only need the repository
//! API; this model is exposed for migrations or direct queries.
//! See also: [`credentials`](super::credentials) for secret storage.

use crate::accounts::Account;
use crate::authz::AccessHierarchy;
use crate::comma_separated_value::CommaSeparatedValue;

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

/// SeaORM entity for an account.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "axum_gate_accounts")]
pub struct Model {
    /// Surrogate primary key (auto‑increment). Not exposed at domain level.
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Stable business identifier for the account (UUID v7 recommended).
    #[sea_orm(unique)]
    pub account_id: Uuid,
    /// External user identifier (e.g. email or username). Should be unique + indexed.
    pub user_id: String,
    /// Comma‑separated list of group identifiers (serialization of `Vec<G>`).
    pub groups: String,
    /// Comma‑separated list of role identifiers (serialization of `Vec<R>`).
    pub roles: String,
}

/// Relation definition placeholder (no outbound relations defined for this model).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl<R, G> From<Account<R, G>> for ActiveModel
where
    R: AccessHierarchy + Eq,
    Vec<R>: CommaSeparatedValue,
    G: Eq + Clone,
    Vec<G>: CommaSeparatedValue,
{
    fn from(value: Account<R, G>) -> Self {
        Self {
            id: ActiveValue::NotSet,
            account_id: ActiveValue::Set(value.account_id),
            user_id: ActiveValue::Set(value.user_id),
            groups: ActiveValue::Set(value.groups.into_csv()),
            roles: ActiveValue::Set(value.roles.into_csv()),
        }
    }
}
