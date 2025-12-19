use super::SeaOrmRepository;
use crate::credentials::Credentials;
use crate::credentials::CredentialsVerifier;
use crate::errors::{Error, Result};
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;
use crate::repositories::TableName;
use crate::repositories::sea_orm::models::credentials as seaorm_credentials;
use crate::repositories::{DatabaseError, DatabaseOperation};
use crate::secrets::Secret;
use crate::secrets::SecretRepository;
use crate::verification_result::VerificationResult;

use sea_orm::{
    ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    entity::{ActiveModelTrait, ActiveValue},
};
use uuid::Uuid;

impl SecretRepository for SeaOrmRepository {
    async fn store_secret(&self, secret: Secret) -> Result<bool> {
        let account_id = secret.account_id;
        let model = seaorm_credentials::ActiveModel::from(secret);
        let _ = model.insert(&self.db).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Insert,
                format!("Failed to store secret: {}", e),
                Some(TableName::AxumGateCredentials.to_string()),
                Some(account_id.to_string()),
            ))
        })?;
        Ok(true)
    }

    /// Removes and returns the secret for the given account id.
    async fn delete_secret(&self, account_id: &Uuid) -> Result<Option<Secret>> {
        let Some(model) = seaorm_credentials::Entity::find()
            .filter(seaorm_credentials::Column::AccountId.eq(*account_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query secret for deletion: {}", e),
                    Some(TableName::AxumGateCredentials.to_string()),
                    Some(account_id.to_string()),
                ))
            })?
        else {
            return Ok(None);
        };

        seaorm_credentials::Entity::delete_by_id(model.id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete secret: {}", e),
                    Some(TableName::AxumGateCredentials.to_string()),
                    Some(account_id.to_string()),
                ))
            })?;

        Ok(Some(Secret {
            account_id: model.account_id,
            secret: model.secret,
        }))
    }

    async fn update_secret(&self, secret: Secret) -> Result<()> {
        let account_id = secret.account_id;
        let old_model = super::models::credentials::Entity::find()
            .filter(super::models::credentials::Column::AccountId.eq(account_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query secret for update: {}", e),
                    Some(TableName::AxumGateCredentials.to_string()),
                    Some(account_id.to_string()),
                ))
            })?
            .ok_or_else(|| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Update,
                    "Secret not found for update".to_string(),
                    Some(TableName::AxumGateCredentials.to_string()),
                    Some(account_id.to_string()),
                ))
            })?;
        let mut new_model = old_model.into_active_model();
        new_model.secret = ActiveValue::Set(secret.secret);
        let _ = new_model.update(&self.db).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Update,
                format!("Failed to update secret: {}", e),
                Some(TableName::AxumGateCredentials.to_string()),
                Some(account_id.to_string()),
            ))
        })?;
        Ok(())
    }
}

impl CredentialsVerifier<Uuid> for SeaOrmRepository {
    async fn verify_credentials(
        &self,
        credentials: Credentials<Uuid>,
    ) -> Result<VerificationResult> {
        use subtle::Choice;

        let model_result = seaorm_credentials::Entity::find()
            .filter(seaorm_credentials::Column::AccountId.eq(credentials.id))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query credentials for verification: {}", e),
                    Some(TableName::AxumGateCredentials.to_string()),
                    Some(credentials.id.to_string()),
                ))
            })?;

        // Select stored or dummy hash (always perform Argon2 verify)
        let (stored_secret_str, user_exists_choice) = match model_result {
            Some(model) => (model.secret, Choice::from(1u8)),
            None => (self.dummy_hash.clone(), Choice::from(0u8)),
        };

        // Perform Argon2 verification locally (constant work)
        let hasher = Argon2Hasher::new_recommended()?;
        let hash_verification_result =
            hasher.verify_value(&credentials.secret, &stored_secret_str)?;

        let hash_matches_choice = Choice::from(match hash_verification_result {
            VerificationResult::Ok => 1u8,
            VerificationResult::Unauthorized => 0u8,
        });

        let final_success_choice = user_exists_choice & hash_matches_choice;

        let final_result = if bool::from(final_success_choice) {
            VerificationResult::Ok
        } else {
            VerificationResult::Unauthorized
        };

        Ok(final_result)
    }
}
