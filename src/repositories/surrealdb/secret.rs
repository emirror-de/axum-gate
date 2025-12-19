use super::SurrealDbRepository;
use crate::credentials::Credentials;
use crate::credentials::CredentialsVerifier;
use crate::errors::{Error, Result};
use crate::repositories::{DatabaseError, DatabaseOperation};
use crate::secrets::{Secret, SecretRepository};
use crate::verification_result::VerificationResult;

use surrealdb::{Connection, RecordId, RecordIdKey};
use uuid::Uuid;

impl<S> SecretRepository for SurrealDbRepository<S>
where
    S: Connection,
{
    async fn store_secret(&self, secret: Secret) -> Result<bool> {
        self.use_ns_db().await?;

        let record_id =
            RecordId::from_table_key(self.scope_settings.credentials.clone(), secret.account_id);

        let account_id = secret.account_id;
        let db_credentials: Option<Secret> = self
            .db
            .insert(&record_id)
            .content(secret)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Failed to store secret: {}", e),
                    Some(self.scope_settings.credentials.clone()),
                    Some(account_id.to_string()),
                ))
            })?;
        Ok(db_credentials.is_some())
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<Option<Secret>> {
        self.use_ns_db().await?;
        let record_id = RecordId::from_table_key(self.scope_settings.credentials.clone(), *id);
        let result: Option<Secret> = self.db.delete(record_id).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to delete and return secret: {}", e),
                Some(self.scope_settings.credentials.clone()),
                Some(id.to_string()),
            ))
        })?;
        Ok(result)
    }

    async fn update_secret(&self, secret: Secret) -> Result<()> {
        self.use_ns_db().await?;

        let record_id =
            RecordId::from_table_key(self.scope_settings.credentials.clone(), secret.account_id);
        let account_id = secret.account_id;
        let _: Option<Secret> = self
            .db
            .update(record_id)
            .content(secret)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Update,
                    format!("Failed to update secret: {}", e),
                    Some(self.scope_settings.credentials.clone()),
                    Some(account_id.to_string()),
                ))
            })?;
        Ok(())
    }
}

impl<S, Id> CredentialsVerifier<Id> for SurrealDbRepository<S>
where
    S: Connection,
    Id: Into<RecordIdKey>,
{
    async fn verify_credentials(&self, credentials: Credentials<Id>) -> Result<VerificationResult> {
        use subtle::Choice;

        self.use_ns_db().await?;
        let record_id =
            RecordId::from_table_key(self.scope_settings.credentials.clone(), credentials.id);

        // Step 1: Query stored secret (if any)
        let exists_query = "SELECT VALUE secret FROM only $record_id".to_string();
        let mut exists_response = self
            .db
            .query(exists_query)
            .bind(("record_id", record_id))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to check user existence: {}", e),
                    Some(self.scope_settings.credentials.clone()),
                    None,
                ))
            })?;

        let stored_secret: Option<String> = exists_response.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract secret: {}", e),
                Some(self.scope_settings.credentials.clone()),
                None,
            ))
        })?;

        // Step 2: Select hash to verify against (always perform verification)
        let (hash_for_verification, user_exists_choice) = match stored_secret {
            Some(secret) => (secret, Choice::from(1u8)),
            None => (self.dummy_hash.clone(), Choice::from(0u8)),
        };

        // Step 3: Perform Argon2 verification inside the database engine (SurrealDB function)
        let verify_query =
            "crypto::argon2::compare(type::string($stored_hash), type::string($request_secret))"
                .to_string();
        let mut verify_response = self
            .db
            .query(verify_query)
            .bind(("stored_hash", hash_for_verification))
            .bind(("request_secret", credentials.secret))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to verify credentials: {}", e),
                    Some(self.scope_settings.credentials.clone()),
                    None,
                ))
            })?;

        let hash_matches: Option<bool> = verify_response.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract verification result: {}", e),
                Some(self.scope_settings.credentials.clone()),
                None,
            ))
        })?;

        // Step 4: Constant-time combination: success only if user exists AND hash matches
        let hash_matches_choice = Choice::from(if hash_matches.unwrap_or(false) {
            1u8
        } else {
            0u8
        });
        let final_success_choice = user_exists_choice & hash_matches_choice;

        // Step 5: Convert to domain result
        let final_result = if bool::from(final_success_choice) {
            VerificationResult::Ok
        } else {
            VerificationResult::Unauthorized
        };

        Ok(final_result)
    }
}

#[test]
#[allow(clippy::unwrap_used)]
fn secret_repository() {
    tokio_test::block_on(async move {
        use super::DatabaseScope;
        use crate::hashing::argon2::Argon2Hasher;
        use surrealdb::Surreal;
        use surrealdb::engine::local::Mem;

        // create a repository
        let db = Surreal::new::<Mem>(()).await.unwrap();
        let scope = DatabaseScope::default();
        let repo = SurrealDbRepository::new(db, scope).unwrap();

        repo.use_ns_db().await.unwrap();

        // create a secret
        let hasher = Argon2Hasher::new_recommended().unwrap();
        let secret = Secret::new(&Uuid::now_v7(), "my_secret", hasher).unwrap();

        // store it
        assert!(repo.store_secret(secret.clone()).await.unwrap());

        // update it
        let mut secret_new = secret.clone();
        secret_new.secret = secret.secret.clone();
        repo.update_secret(secret_new.clone()).await.unwrap();

        // verify it
        let credentials = Credentials::new(&secret.account_id, "my_secret");
        assert!(matches!(
            repo.verify_credentials(credentials).await,
            Ok(VerificationResult::Ok)
        ));
    });
}
