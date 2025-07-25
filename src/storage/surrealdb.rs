//! Storage implementations that use surrealdb as backend.

use super::TableNames;
use crate::hashing::VerificationResult;
use crate::secrets::Secret;
use crate::services::{AccountStorageService, CredentialsVerifierService, SecretStorageService};
use crate::utils::AccessHierarchy;
use crate::{Account, Credentials, Error};

use std::default::Default;

use anyhow::{Result, anyhow};
use serde::Serialize;
use serde::de::DeserializeOwned;
use surrealdb::{Connection, RecordId, RecordIdKey, Surreal};
use uuid::Uuid;

/// Configurations to use with the [surrealdb] database.
#[derive(Clone, Debug)]
pub struct DatabaseScope {
    /// The table names of the database.
    pub table_names: TableNames,
    /// Namespace where data is stored.
    pub namespace: String,
    /// Database name where data is stored.
    pub database: String,
}

impl Default for DatabaseScope {
    fn default() -> Self {
        Self {
            table_names: TableNames::default(),
            namespace: "axum-gate".to_string(),
            database: "axum-gate".to_string(),
        }
    }
}

/// A storage that uses [surrealdb] as backend.
#[derive(Clone)]
pub struct SurrealDbStorage<S>
where
    S: Connection,
{
    db: Surreal<S>,
    scope_settings: DatabaseScope,
}

impl<S> SurrealDbStorage<S>
where
    S: Connection,
{
    /// Creates a new storage that uses the given database connection limited by the given scope.
    pub fn new(db: Surreal<S>, scope_settings: DatabaseScope) -> Self {
        Self { db, scope_settings }
    }

    /// Sets the correct namespace and database to use.
    async fn use_ns_db(&self) -> Result<()> {
        self.db
            .use_ns(&self.scope_settings.namespace)
            .use_db(&self.scope_settings.database)
            .await
            .map_err(|e| anyhow!(Error::Storage(e.to_string())))
    }
}

impl<R, G, S> AccountStorageService<R, G> for SurrealDbStorage<S>
where
    R: AccessHierarchy + Eq + DeserializeOwned + Serialize + 'static,
    G: Serialize + DeserializeOwned + Eq + 'static,
    S: Connection,
{
    async fn query_account_by_user_id(&self, user_id: &str) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;
        let db_account: Option<Account<R, G>> = self
            .db
            .select(RecordId::from_table_key(
                &self.scope_settings.table_names.accounts,
                user_id,
            ))
            .await
            .map_err(|e| Error::AccountStorage(e.to_string()))?;
        Ok(db_account)
    }

    async fn store_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;
        let record_id =
            RecordId::from_table_key(&self.scope_settings.table_names.accounts, &account.user_id);
        let db_account: Option<Account<R, G>> = self
            .db
            .insert(record_id)
            .content(account)
            .await
            .map_err(|e| Error::AccountStorage(format!("Could not insert account: {e}")))?;
        Ok(db_account)
    }

    async fn delete_account(&self, user_id: &str) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;
        let db_account: Option<Account<R, G>> = self
            .db
            .delete(RecordId::from_table_key(
                &self.scope_settings.table_names.accounts,
                user_id,
            ))
            .await
            .map_err(|e| Error::AccountStorage(e.to_string()))?;
        Ok(db_account)
    }

    async fn update_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;
        let record_id =
            RecordId::from_table_key(&self.scope_settings.table_names.accounts, &account.user_id);
        let db_account: Option<Account<R, G>> = self.db.update(&record_id).content(account).await?;
        Ok(db_account)
    }
}

impl<S> SecretStorageService for SurrealDbStorage<S>
where
    S: Connection,
{
    async fn store_secret(&self, secret: Secret) -> Result<bool> {
        self.use_ns_db().await?;

        let record_id = RecordId::from_table_key(
            &self.scope_settings.table_names.credentials,
            secret.account_id,
        );

        let db_credentials: Option<Secret> = self
            .db
            .insert(&record_id)
            .content(secret)
            .await
            .map_err(|e| Error::SecretStorage(e.to_string()))?;
        Ok(db_credentials.is_some())
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<bool> {
        self.use_ns_db().await?;
        let record_id =
            RecordId::from_table_key(&self.scope_settings.table_names.credentials, *id);
        let result: Option<Secret> = self
            .db
            .delete(record_id)
            .await
            .map_err(|e| Error::SecretStorage(e.to_string()))?;
        Ok(result.is_some())
    }

    async fn update_secret(&self, secret: Secret) -> Result<()> {
        self.use_ns_db().await?;

        let record_id = RecordId::from_table_key(
            &self.scope_settings.table_names.credentials,
            secret.account_id,
        );
        let _: Option<Secret> = self
            .db
            .update(record_id)
            .content(secret)
            .await
            .map_err(|e| Error::SecretStorage(e.to_string()))?;
        Ok(())
    }
}

impl<S, Id> CredentialsVerifierService<Id> for SurrealDbStorage<S>
where
    S: Connection,
    Id: Into<RecordIdKey>,
{
    async fn verify_credentials(&self, credentials: Credentials<Id>) -> Result<VerificationResult> {
        self.use_ns_db().await?;
        let record_id =
            RecordId::from_table_key(&self.scope_settings.table_names.credentials, credentials.id);
        let query = "crypto::argon2::compare((SELECT secret from only $record_id).secret, type::string($request_secret))".to_string();

        let mut response = self
            .db
            .query(query)
            .bind(("record_id", record_id))
            .bind(("request_secret", credentials.secret))
            .await
            .map_err(|e| Error::SecretStorage(e.to_string()))?;
        let result: Option<bool> = response
            .take(0)
            .map_err(|e| Error::SecretStorage(e.to_string()))?;

        Ok(VerificationResult::from(result.unwrap_or(false)))
    }
}

#[test]
fn secret_storage() {
    tokio_test::block_on(async move {
        use crate::hashing::Argon2Hasher;
        use surrealdb::engine::local::Mem;

        // create a storage
        let db = Surreal::new::<Mem>(())
            .await
            .expect("Could not create in memory database.");
        let creds_storage = SurrealDbStorage::new(db, DatabaseScope::default());
        let id = Uuid::now_v7();

        let creds = Secret::new(&id, "admin_password", Argon2Hasher).unwrap();

        creds_storage.store_secret(creds).await.unwrap();

        let creds_to_verify = Credentials::new(&id, "admin_password");
        let wrong_creds = Credentials::new(&id, "admin_passwordwrong");
        assert_eq!(
            VerificationResult::Unauthorized,
            creds_storage.verify_credentials(wrong_creds).await.unwrap()
        );
        assert_eq!(
            VerificationResult::Ok,
            creds_storage
                .verify_credentials(creds_to_verify)
                .await
                .unwrap()
        );
    })
}

#[test]
fn account_storage() {
    tokio_test::block_on(async move {
        use crate::{Account, Group, Role};
        use surrealdb::engine::local::Mem;

        let db = Surreal::new::<Mem>(())
            .await
            .expect("Could not create in memory database.");
        let account_storage = SurrealDbStorage::new(db, DatabaseScope::default());

        let account = Account::new(
            "mymail@accountid-example.com",
            &[Role::Admin],
            &[Group::new("admin"), Group::new("audio")],
        );
        let account = account_storage
            .store_account(account)
            .await
            .unwrap()
            .unwrap();

        let Some(db_account): Option<Account<Role, Group>> = account_storage
            .query_account_by_user_id(&account.user_id)
            .await
            .unwrap()
        else {
            panic!("Account not found in database.");
        };

        assert_eq!(account.account_id, db_account.account_id);

        let Some(account): Option<Account<Role, Group>> = account_storage
            .delete_account(&account.user_id)
            .await
            .unwrap()
        else {
            panic!("Removing passport was not successful.");
        };

        let account: Option<Account<Role, Group>> = account_storage
            .query_account_by_user_id(&account.user_id)
            .await
            .unwrap();
        if account.is_some() {
            panic!("Passport is still available althoug it should not.");
        };
    })
}

/*
/// Account model as it is represented within [surrealdb].
#[derive(Serialize, Deserialize)]
struct SurrealDbAccount<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq,
{
    /// The unique record id.
    pub id: RecordId,
    /// The actual account data.
    #[serde(flatten)]
    pub account_data: Account<R, G>,
}

impl<R, G> From<Account<R, G>> for SurrealDbAccount<R, G>
where
    R: AccessHierarchy + Eq,
    G: Eq,
{
    fn from(value: Account<R, G>) -> Self {
        Self {
            id: RecordId::from_table_key(TableNames::default().accounts, value.user_id.clone()),
            account_data: value,
        }
    }
}

/// Credentials model as it is represented within [surrealdb].
#[derive(Serialize, Deserialize)]
struct SurrealDbCredentials {
    /// The unique record id.
    pub id: RecordId,
    /// The actual secret data.
    #[serde(flatten)]
    pub data: Credentials<Uuid>,
}

impl From<Credentials<Uuid>> for SurrealDbCredentials {
    fn from(value: Credentials<Uuid>) -> Self {
        Self {
            id: RecordId::from_table_key(TableNames::default().credentials, value.id.clone()),
            data: value,
        }
    }
} */
