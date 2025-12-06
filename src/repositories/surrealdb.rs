//! SurrealDB-backed repositories for accounts and secrets with constant-time credential verification.

use super::TableName;
use crate::accounts::Account;
use crate::accounts::AccountRepository;
use crate::authz::AccessHierarchy;
use crate::credentials::Credentials;
use crate::credentials::CredentialsVerifier;
use crate::errors::{Error, Result};
use crate::groups::{GroupEntity, GroupRepository};
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;
use crate::permissions::PermissionId;
use crate::permissions::mapping::PermissionMapping;
use crate::permissions::mapping::PermissionMappingRepository;
use crate::repositories::{DatabaseError, DatabaseOperation};
use crate::secrets::{Secret, SecretRepository};
use crate::verification_result::VerificationResult;

use std::default::Default;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use surrealdb::{Connection, RecordId, RecordIdKey, Surreal};
use uuid::Uuid;

/// Scope configuration (namespace, database, table names) used by `SurrealDbRepository`.
///
/// Most users can rely on `DatabaseScope::default()`. Override fields only if you
/// need custom namespace / database names or different table naming.
#[derive(Clone, Debug)]
pub struct DatabaseScope {
    /// Accounts table (stores user id, groups, roles).
    pub accounts: String,
    /// Credentials table (stores hashed secrets).
    pub credentials: String,
    /// Permission mappings table (stores normalized string <-> id mapping).
    pub permission_mappings: String,
    /// Groups table (stores persisted group entities).
    pub groups: String,
    /// Namespace where data is stored.
    pub namespace: String,
    /// Database name where data is stored.
    pub database: String,
}

impl Default for DatabaseScope {
    fn default() -> Self {
        Self {
            accounts: TableName::AxumGateAccounts.to_string(),
            credentials: TableName::AxumGateCredentials.to_string(),
            permission_mappings: TableName::AxumGatePermissionMappings.to_string(),
            groups: TableName::AxumGateGroups.to_string(),
            namespace: "axumGate".to_string(),
            database: "axumGate".to_string(),
        }
    }
}

/// SurrealDB-backed repository offering CRUD for accounts & secrets plus constant-time
/// credential verification (uses a precomputed dummy Argon2 hash when a secret is absent).
///
/// Use `SurrealDbRepository::new(db, DatabaseScope::default())` for standard setups.
#[derive(Clone)]
pub struct SurrealDbRepository<S>
where
    S: Connection,
{
    db: Surreal<S>,
    scope_settings: DatabaseScope,
    /// Precomputed dummy Argon2 hash used when a user's secret does not exist.
    /// Ensures the Argon2 verification path is always exercised.
    dummy_hash: String,
}

impl<S> SurrealDbRepository<S>
where
    S: Connection,
{
    /// Creates a new repository that uses the given database connection limited by the given scope.
    pub fn new(db: Surreal<S>, scope_settings: DatabaseScope) -> Result<Self> {
        let hasher = Argon2Hasher::new_recommended()?;
        // Panic on failure here is acceptable: construction failure indicates a
        // fundamental issue (e.g. RNG) and mirrors the in‑memory repo strategy.
        let dummy_hash = hasher.hash_value("dummy_password")?;
        Ok(Self {
            db,
            scope_settings,
            dummy_hash,
        })
    }

    /// Sets the correct namespace and database to use.
    async fn use_ns_db(&self) -> Result<()> {
        self.db
            .use_ns(&self.scope_settings.namespace)
            .use_db(&self.scope_settings.database)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Connect,
                    format!("Failed to set namespace/database: {}", e),
                    None,
                    None,
                ))
            })
    }
}

impl<R, G, S> AccountRepository<R, G> for SurrealDbRepository<S>
where
    R: AccessHierarchy + Eq + DeserializeOwned + Serialize + Send + Sync + 'static,
    G: Serialize + DeserializeOwned + Eq + Clone + Send + Sync + 'static,
    S: Connection,
{
    async fn query_account_by_user_id(&self, user_id: &str) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;

        // Use a SELECT query to find the account by `user_id` rather than assuming
        // the storage record key equals the login identifier. This avoids coupling
        // the persisted record key to a mutable/login value.
        let query = "SELECT * FROM type::table($table) WHERE user_id = $uid LIMIT 1";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.accounts.clone()))
            .bind(("uid", user_id.to_string()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query account by user_id: {}", e),
                    Some(self.scope_settings.accounts.clone()),
                    Some(user_id.to_string()),
                ))
            })?;

        // Extract the first result row as the domain Account if present.
        res.take::<Option<Account<R, G>>>(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract account by user_id: {}", e),
                Some(self.scope_settings.accounts.clone()),
                Some(user_id.to_string()),
            ))
        })
    }

    async fn query_account_by_id(&self, account_id: &Uuid) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;

        let db_account: Option<Account<R, G>> = self
            .db
            .select(RecordId::from_table_key(
                &self.scope_settings.accounts,
                *account_id,
            ))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query account by account_id: {}", e),
                    Some(self.scope_settings.accounts.clone()),
                    Some(account_id.to_string()),
                ))
            })?;
        Ok(db_account)
    }

    async fn store_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;

        let record_id = RecordId::from_table_key(
            self.scope_settings.accounts.clone(),
            account.account_id.clone(),
        );
        let user_id = account.user_id.clone();
        let db_account: Option<Account<R, G>> = self
            .db
            .insert(record_id)
            .content(account)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Could not insert account: {}", e),
                    Some(self.scope_settings.accounts.clone()),
                    Some(user_id),
                ))
            })?;
        Ok(db_account)
    }

    async fn delete_account(&self, account_id: &Uuid) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;

        let db_account: Option<Account<R, G>> = self
            .db
            .delete(RecordId::from_table_key(
                self.scope_settings.accounts.clone(),
                *account_id,
            ))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete account: {}", e),
                    Some(self.scope_settings.accounts.clone()),
                    Some(account_id.to_string()),
                ))
            })?;
        Ok(db_account)
    }

    async fn update_account(&self, account: Account<R, G>) -> Result<Option<Account<R, G>>> {
        self.use_ns_db().await?;

        let record_id =
            RecordId::from_table_key(self.scope_settings.accounts.clone(), *account.account_id);
        let db_account: Option<Account<R, G>> = self.db.update(&record_id).content(account).await?;
        Ok(db_account)
    }

    async fn query_all_accounts(&self) -> Result<Vec<Account<R, G>>> {
        self.use_ns_db().await?;
        let db_accounts: Vec<Account<R, G>> = self
            .db
            .select(self.scope_settings.accounts.clone())
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query all accounts: {}", e),
                    Some(self.scope_settings.accounts.clone()),
                    None,
                ))
            })?;
        Ok(db_accounts)
    }
}

impl<S, T> GroupRepository<T> for SurrealDbRepository<S>
where
    S: Connection,
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    async fn store_group(&self, group: T) -> Result<bool> {
        self.use_ns_db().await?;
        let id = group.group_id().to_string();

        // Check existence first to emulate insert-if-not-exists semantics
        let recid = RecordId::from_table_key(self.scope_settings.groups.clone(), &id);
        let existing: Option<T> = self.db.select(recid.clone()).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to query group existence: {}", e),
                Some(self.scope_settings.groups.clone()),
                Some(id.clone()),
            ))
        })?;
        if existing.is_some() {
            return Ok(false);
        }

        let inserted: Option<T> = self.db.insert(recid).content(group).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Insert,
                format!("Failed to insert group: {}", e),
                Some(self.scope_settings.groups.clone()),
                Some(id.clone()),
            ))
        })?;
        Ok(inserted.is_some())
    }

    async fn delete_group(&self, id: &str) -> Result<Option<T>> {
        self.use_ns_db().await?;
        let recid = RecordId::from_table_key(self.scope_settings.groups.clone(), id);
        let deleted: Option<T> = self.db.delete(recid).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to delete group: {}", e),
                Some(self.scope_settings.groups.clone()),
                Some(id.to_string()),
            ))
        })?;
        Ok(deleted)
    }

    async fn update_group(&self, group: T) -> Result<Option<T>> {
        self.use_ns_db().await?;
        let id = group.group_id().to_string();
        let recid = RecordId::from_table_key(self.scope_settings.groups.clone(), &id);
        let updated: Option<T> = self.db.update(&recid).content(group).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Update,
                format!("Failed to update group: {}", e),
                Some(self.scope_settings.groups.clone()),
                Some(id.clone()),
            ))
        })?;
        Ok(updated)
    }

    async fn query_group_by_id(&self, id: &str) -> Result<Option<T>> {
        self.use_ns_db().await?;
        let recid = RecordId::from_table_key(self.scope_settings.groups.clone(), id);
        let res: Option<T> = self.db.select(recid).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to query group by id: {}", e),
                Some(self.scope_settings.groups.clone()),
                Some(id.to_string()),
            ))
        })?;
        Ok(res)
    }

    async fn query_all_groups(&self) -> Result<Vec<T>> {
        self.use_ns_db().await?;
        let db_groups: Vec<T> = self
            .db
            .select(self.scope_settings.groups.clone())
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query all groups: {}", e),
                    Some(self.scope_settings.groups.clone()),
                    None,
                ))
            })?;
        Ok(db_groups)
    }
}

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

/// Adapter for persisting `PermissionMapping` in SurrealDB.
///
/// SurrealDB can deserialize numeric fields as signed 64-bit integers (i64),
/// while our permission IDs are computed 64-bit values that may exceed the
/// positive i63 range. Persisting `permission_id` as a `String` avoids
/// signedness/width pitfalls across different SurrealDB backends and ensures
/// stable round‑trips regardless of how numbers are represented internally.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SurrealPermissionMapping {
    normalized_string: String,
    permission_id: String,
}

impl From<PermissionMapping> for SurrealPermissionMapping {
    fn from(m: PermissionMapping) -> Self {
        Self {
            normalized_string: m.normalized_string().to_string(),
            permission_id: m.permission_id().as_u64().to_string(),
        }
    }
}

impl std::convert::TryFrom<SurrealPermissionMapping> for PermissionMapping {
    type Error = String;

    fn try_from(value: SurrealPermissionMapping) -> std::result::Result<Self, Self::Error> {
        let id_u64 = value.permission_id.parse::<u64>().map_err(|e| {
            format!(
                "invalid permission_id string '{}': {}",
                value.permission_id, e
            )
        })?;
        let id = PermissionId::from_u64(id_u64);
        PermissionMapping::new(value.normalized_string.clone(), id)
            .map_err(|e| format!("failed to construct PermissionMapping: {}", e))
    }
}

impl<S> PermissionMappingRepository for SurrealDbRepository<S>
where
    S: Connection,
{
    async fn store_mapping(&self, mapping: PermissionMapping) -> Result<Option<PermissionMapping>> {
        // Validate the mapping first
        if let Err(e) = mapping.validate() {
            // Treat validation failures as infrastructure-safe errors for this backend
            return Err(Error::Database(DatabaseError::with_context(
                DatabaseOperation::Insert,
                format!("Invalid permission mapping: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                None,
            )));
        }

        self.use_ns_db().await?;

        // Enforce uniqueness by permission ID (direct WHERE query)
        let query_id = "SELECT * FROM type::table($table) WHERE permission_id = $pid LIMIT 1";
        let mut res_id = self
            .db
            .query(query_id)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("pid", mapping.permission_id().as_u64().to_string()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to check existing mapping by id: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    Some(mapping.permission_id().as_u64().to_string()),
                ))
            })?;
        let exists_by_id: Vec<SurrealPermissionMapping> = res_id.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract existing mapping by id: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                Some(mapping.permission_id().as_u64().to_string()),
            ))
        })?;
        if !exists_by_id.is_empty() {
            return Ok(None);
        }

        // Enforce uniqueness by normalized string (record key)
        let record_id = RecordId::from_table_key(
            self.scope_settings.permission_mappings.clone(),
            mapping.normalized_string(),
        );
        let exists_by_string: Option<SurrealPermissionMapping> =
            self.db.select(&record_id).await.map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to check existing mapping by string: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;
        if exists_by_string.is_some() {
            return Ok(None);
        }

        // Insert mapping using normalized string as the record key
        let stored_spm: Option<SurrealPermissionMapping> = self
            .db
            .insert(&record_id)
            .content(SurrealPermissionMapping::from(mapping))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Failed to store permission mapping: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        let stored = match stored_spm {
            Some(spm) => {
                let dom = PermissionMapping::try_from(spm).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Insert,
                        format!("Failed to convert stored permission mapping: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        None,
                    ))
                })?;
                Some(dom)
            }
            None => None,
        };

        Ok(stored)
    }

    async fn remove_mapping_by_id(&self, id: PermissionId) -> Result<Option<PermissionMapping>> {
        self.use_ns_db().await?;

        // Delete directly by permission_id and return the removed record (if any)
        let query = "DELETE type::table($table) WHERE permission_id = $pid RETURN BEFORE";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("pid", id.as_u64().to_string()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mapping by id: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    Some(id.as_u64().to_string()),
                ))
            })?;

        let removed: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to extract deleted permission mapping: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                Some(id.as_u64().to_string()),
            ))
        })?;

        removed
            .into_iter()
            .next()
            .map(|spm| {
                PermissionMapping::try_from(spm).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Delete,
                        format!("Failed to convert deleted permission mapping: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        Some(id.as_u64().to_string()),
                    ))
                })
            })
            .transpose()
    }

    async fn remove_mapping_by_string(
        &self,
        permission: &str,
    ) -> Result<Option<PermissionMapping>> {
        self.use_ns_db().await?;

        // Normalize via domain logic
        let normalized = PermissionMapping::from(permission)
            .normalized_string()
            .to_string();

        // Delete directly by normalized string and return the removed record (if any)
        let query = "DELETE type::table($table) WHERE normalized_string = $ns RETURN BEFORE";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("ns", normalized))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mapping by string: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        let removed: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to extract deleted permission mapping: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                None,
            ))
        })?;

        removed
            .into_iter()
            .next()
            .map(|spm| {
                PermissionMapping::try_from(spm).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Delete,
                        format!("Failed to convert deleted permission mapping: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        None,
                    ))
                })
            })
            .transpose()
    }

    async fn query_mapping_by_id(&self, id: PermissionId) -> Result<Option<PermissionMapping>> {
        self.use_ns_db().await?;

        // Direct WHERE query by permission_id
        let query = "SELECT * FROM type::table($table) WHERE permission_id = $pid LIMIT 1";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("pid", id.as_u64().to_string()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by id: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        let found: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract permission mapping by id: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                Some(id.as_u64().to_string()),
            ))
        })?;

        found
            .into_iter()
            .next()
            .map(|spm| {
                PermissionMapping::try_from(spm).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Query,
                        format!("Failed to convert permission mapping: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        Some(id.as_u64().to_string()),
                    ))
                })
            })
            .transpose()
    }

    async fn query_mapping_by_string(&self, permission: &str) -> Result<Option<PermissionMapping>> {
        self.use_ns_db().await?;

        let normalized = PermissionMapping::from(permission)
            .normalized_string()
            .to_string();

        // Direct select by record key (normalized string)
        let record_id = RecordId::from_table_key(
            self.scope_settings.permission_mappings.clone(),
            normalized.clone(),
        );

        let mapping_spm: Option<SurrealPermissionMapping> =
            self.db.select(record_id).await.map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by string: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        mapping_spm
            .map(|spm| {
                PermissionMapping::try_from(spm).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Query,
                        format!("Failed to convert permission mapping: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        None,
                    ))
                })
            })
            .transpose()
    }

    async fn list_all_mappings(&self) -> Result<Vec<PermissionMapping>> {
        self.use_ns_db().await?;

        let all_spm: Vec<SurrealPermissionMapping> = self
            .db
            .select(self.scope_settings.permission_mappings.clone())
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to list permission mappings: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        let mut out = Vec::with_capacity(all_spm.len());
        for spm in all_spm {
            let dom = PermissionMapping::try_from(spm).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to convert permission mapping: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;
            out.push(dom);
        }
        Ok(out)
    }
}

#[test]
#[allow(clippy::unwrap_used)]
fn secret_repository() {
    tokio_test::block_on(async move {
        use crate::hashing::argon2::Argon2Hasher;
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
