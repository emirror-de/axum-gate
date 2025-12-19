use super::SeaOrmRepository;
use crate::accounts::Account;
use crate::accounts::AccountRepository;
use crate::authz::AccessHierarchy;
use crate::comma_separated_value::CommaSeparatedValue;
use crate::errors::{Error, Result};
use crate::repositories::TableName;
use crate::repositories::sea_orm::models::account as seaorm_account;
use crate::repositories::{DatabaseError, DatabaseOperation};

use sea_orm::{
    ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    entity::{ActiveModelTrait, ActiveValue},
};
use serde::{Serialize, de::DeserializeOwned};

impl<R, G> AccountRepository<R, G> for SeaOrmRepository
where
    R: AccessHierarchy
        + Eq
        + Serialize
        + DeserializeOwned
        + std::fmt::Display
        + Clone
        + Send
        + Sync
        + 'static,
    G: Eq + Clone + Send + Sync + 'static,
    Vec<R>: CommaSeparatedValue,
    Vec<G>: CommaSeparatedValue,
{
    async fn query_account_by_user_id(&self, user_id: &str) -> Result<Option<Account<R, G>>> {
        let Some(model) = seaorm_account::Entity::find()
            .filter(seaorm_account::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query account by user_id: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    Some(user_id.to_string()),
                ))
            })?
        else {
            return Ok(None);
        };

        Ok(Some(Account::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to convert database model to Account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                Some(user_id.to_string()),
            ))
        })?))
    }

    async fn query_account_by_id(&self, account_id: &uuid::Uuid) -> Result<Option<Account<R, G>>> {
        let Some(model) = seaorm_account::Entity::find()
            .filter(seaorm_account::Column::AccountId.eq(*account_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query account by account_id: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    Some(account_id.to_string()),
                ))
            })?
        else {
            return Ok(None);
        };

        Ok(Some(Account::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to convert database model to Account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                Some(account_id.to_string()),
            ))
        })?))
    }

    async fn store_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        let mut model = seaorm_account::ActiveModel::from(account);
        model.id = ActiveValue::NotSet;
        let model = model.insert(&self.db).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Insert,
                format!("Failed to insert account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                None,
            ))
        })?;
        Ok(Some(Account::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Insert,
                format!("Failed to convert inserted model to Account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                None,
            ))
        })?))
    }

    async fn delete_account(&self, account_id: &uuid::Uuid) -> Result<Option<Account<R, G>>> {
        let Some(model) = seaorm_account::Entity::find()
            .filter(seaorm_account::Column::AccountId.eq(*account_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query account for deletion: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    Some(account_id.to_string()),
                ))
            })?
        else {
            return Ok(None);
        };

        seaorm_account::Entity::delete_by_id(model.id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete account: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    Some(account_id.to_string()),
                ))
            })?;

        Ok(Some(Account::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to convert deleted model to Account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                Some(account_id.to_string()),
            ))
        })?))
    }

    async fn update_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        let Some(db_account) = seaorm_account::Entity::find()
            .filter(seaorm_account::Column::AccountId.eq(account.account_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query account for update: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    Some(account.user_id.clone()),
                ))
            })?
        else {
            return Ok(None);
        };
        let mut db_account = db_account.into_active_model();
        let user_id = account.user_id.clone();
        db_account.user_id = ActiveValue::Set(account.user_id);
        db_account.groups = ActiveValue::Set(account.groups.into_csv());
        db_account.roles = ActiveValue::Set(account.roles.into_csv());

        let model = db_account.update(&self.db).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Update,
                format!("Failed to update account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                Some(user_id.clone()),
            ))
        })?;
        Ok(Some(Account::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Update,
                format!("Failed to convert updated model to Account: {}", e),
                Some(TableName::AxumGateAccounts.to_string()),
                Some(user_id),
            ))
        })?))
    }

    async fn query_all_accounts(&self) -> Result<Vec<Account<R, G>>> {
        // Fetch all account models from the database and convert into domain `Account` instances.
        let models = seaorm_account::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query all accounts: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    None,
                ))
            })?;

        let mut out = Vec::with_capacity(models.len());
        for m in models {
            let dom = Account::try_from(m).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to convert account model: {}", e),
                    Some(TableName::AxumGateAccounts.to_string()),
                    None,
                ))
            })?;
            out.push(dom);
        }

        Ok(out)
    }
}
