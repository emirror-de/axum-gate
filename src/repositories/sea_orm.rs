//! SeaORM repository integration providing account & credential persistence with constant‑time verification.
//!
//! This repository includes constant-time credential verification to
//! mitigate user enumeration via timing differences. A dummy Argon2
//! hash (built with the active build-mode preset) is precomputed at
//! construction and used whenever a secret for a given account id
//! does not exist, ensuring the Argon2 verification path is always
//! executed.

use crate::accounts::Account;
use crate::accounts::AccountRepository;
use crate::authz::AccessHierarchy;
use crate::comma_separated_value::CommaSeparatedValue;
use crate::credentials::Credentials;
use crate::credentials::CredentialsVerifier;
use crate::errors::{Error, Result};
use crate::groups::{GroupEntity, GroupRepository as GroupRepositoryTrait};
use crate::hashing::HashingService;
use crate::hashing::argon2::Argon2Hasher;
use crate::permissions::PermissionId;
use crate::permissions::mapping::{
    PermissionMapping, PermissionMappingRepository, PermissionMappingRepositoryBulk,
};
use crate::repositories::TableName;
use crate::repositories::sea_orm::models::{
    account as seaorm_account, credentials as seaorm_credentials, group as seaorm_group,
    permission_mapping as seaorm_permission_mapping,
};
use crate::repositories::{DatabaseError, DatabaseOperation};
use crate::secrets::Secret;
use crate::secrets::SecretRepository;
use crate::verification_result::VerificationResult;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[cfg(feature = "storage-seaorm")]
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    entity::{ActiveModelTrait, ActiveValue},
};

/// SeaORM persistence entities (database models) used by `SeaOrmRepository`.
///
/// These are thin schemas mapping relational rows to structures convertible
/// to and from the domain layer (`Account`, `Secret`).
pub mod models;

/// Repository implementation for [SeaORM](sea_orm).
///
/// # Responsibilities
/// * Translate between domain `Account` / `Secret` and SeaORM models
/// * Provide CRUD operations required by higher‑level services
/// * Perform constant‑time credential verification
///
/// # Timing Side‑Channel Mitigation
/// A precomputed dummy Argon2 hash (same parameters as production hashes)
/// is always verified when an account's secret is missing. Existence and
/// hash‑match results are combined using bitwise operations on `subtle::Choice`
/// to avoid branching that could leak information.
///
/// # Concurrency
/// `SeaOrmRepository` is cheaply cloneable (internally holds a `DatabaseConnection`).
/// Clones share the same underlying pool and are `Send + Sync`.
///
/// # Error Semantics
/// Each DB interaction maps the concrete SeaORM / driver error into an
/// `DatabaseError::Operation` variant enriched with: operation, table,
/// and record identifier (when available).
///
/// # Usage
/// ```rust
/// # #[cfg(feature="storage-seaorm")]
/// # {
/// use axum_gate::repositories::sea_orm::SeaOrmRepository;
/// use sea_orm::Database;
/// # #[tokio::test] async fn usage_sea_orm() -> anyhow::Result<()> {
/// let db = Database::connect("sqlite::memory:").await?;
/// let repo = SeaOrmRepository::new(&db);
/// # Ok(()) }
/// # }
/// ```
///
/// # Extensibility
/// * To add new persisted aggregates: create a new model module, implement
///   conversions, and extend the repository or introduce a new trait.
/// * For multi‑tenant separation consider separate schemas / databases at
///   the connection level; this struct does not enforce tenant isolation.
///
/// # Security Considerations
/// * Still pair with rate limiting & structured logging
/// * Keep Argon2 parameters strong and consistent
/// * Secrets are assumed already hashed (insertion path uses hashed values)
pub struct SeaOrmRepository {
    db: DatabaseConnection,
    /// Precomputed dummy Argon2 hash used for nonexistent accounts to keep
    /// verification timing consistent.
    dummy_hash: String,
}

impl SeaOrmRepository {
    /// Creates a new repository that uses the given database connection as backend.
    pub fn new(db: &DatabaseConnection) -> Result<Self> {
        let hasher = Argon2Hasher::new_recommended()?;
        let dummy_hash = hasher.hash_value("dummy_password")?;
        Ok(Self {
            db: db.clone(),
            dummy_hash,
        })
    }
}

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
        let old_model = models::credentials::Entity::find()
            .filter(models::credentials::Column::AccountId.eq(account_id))
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

impl PermissionMappingRepository for SeaOrmRepository {
    async fn store_mapping(
        &self,
        mapping: PermissionMapping,
    ) -> crate::errors::Result<Option<PermissionMapping>> {
        // Validate mapping consistency first
        if let Err(e) = mapping.validate() {
            return Err(Error::Database(DatabaseError::with_context(
                DatabaseOperation::Insert,
                format!("Invalid permission mapping: {}", e),
                Some(TableName::AxumGatePermissionMappings.to_string()),
                None,
            )));
        }

        // Insert mapping; rely on DB unique constraints
        let stored = match seaorm_permission_mapping::ActiveModel::from(mapping.clone())
            .insert(&self.db)
            .await
        {
            Ok(model) => PermissionMapping::try_from(model).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Failed to convert stored permission mapping: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?,
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                if msg.contains("unique") && msg.contains("constraint") {
                    // Treat unique constraint violation as "already exists"
                    return Ok(None);
                }
                return Err(Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Failed to store permission mapping: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                )));
            }
        };
        Ok(Some(stored))
    }

    async fn remove_mapping_by_id(
        &self,
        id: PermissionId,
    ) -> crate::errors::Result<Option<PermissionMapping>> {
        let id_str = id.as_u64().to_string();

        // Fetch existing to return it
        let Some(model) = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::PermissionId.eq(id_str.clone()))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by id: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    Some(id_str.clone()),
                ))
            })?
        else {
            return Ok(None);
        };

        // Delete it
        seaorm_permission_mapping::Entity::delete_by_id(model.id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mapping by id: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    Some(id_str),
                ))
            })?;

        let domain = PermissionMapping::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to convert deleted permission mapping: {}", e),
                Some(TableName::AxumGatePermissionMappings.to_string()),
                None,
            ))
        })?;
        Ok(Some(domain))
    }

    async fn remove_mapping_by_string(
        &self,
        permission: &str,
    ) -> crate::errors::Result<Option<PermissionMapping>> {
        let normalized = PermissionMapping::from(permission)
            .normalized_string()
            .to_string();

        // Fetch existing to return it
        let Some(model) = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::NormalizedString.eq(normalized.clone()))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by string: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?
        else {
            return Ok(None);
        };

        // Delete it
        seaorm_permission_mapping::Entity::delete_by_id(model.id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mapping by string: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        let domain = PermissionMapping::try_from(model).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to convert deleted permission mapping: {}", e),
                Some(TableName::AxumGatePermissionMappings.to_string()),
                None,
            ))
        })?;
        Ok(Some(domain))
    }

    async fn query_mapping_by_id(
        &self,
        id: PermissionId,
    ) -> crate::errors::Result<Option<PermissionMapping>> {
        let id_str = id.as_u64().to_string();
        let model_opt = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::PermissionId.eq(id_str.clone()))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by id: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    Some(id_str),
                ))
            })?;

        model_opt
            .map(|m| {
                PermissionMapping::try_from(m).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Query,
                        format!("Failed to convert permission mapping: {}", e),
                        Some(TableName::AxumGatePermissionMappings.to_string()),
                        None,
                    ))
                })
            })
            .transpose()
    }

    async fn query_mapping_by_string(
        &self,
        permission: &str,
    ) -> crate::errors::Result<Option<PermissionMapping>> {
        let normalized = PermissionMapping::from(permission)
            .normalized_string()
            .to_string();

        let model_opt = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::NormalizedString.eq(normalized))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by string: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        model_opt
            .map(|m| {
                PermissionMapping::try_from(m).map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Query,
                        format!("Failed to convert permission mapping: {}", e),
                        Some(TableName::AxumGatePermissionMappings.to_string()),
                        None,
                    ))
                })
            })
            .transpose()
    }

    async fn list_all_mappings(&self) -> crate::errors::Result<Vec<PermissionMapping>> {
        let models = seaorm_permission_mapping::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to list permission mappings: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        let mut out = Vec::with_capacity(models.len());
        for m in models {
            let dom = PermissionMapping::try_from(m).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to convert permission mapping: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;
            out.push(dom);
        }

        Ok(out)
    }
}

impl PermissionMappingRepositoryBulk for SeaOrmRepository {
    async fn store_mappings(
        &self,
        mappings: Vec<PermissionMapping>,
    ) -> crate::errors::Result<Vec<PermissionMapping>> {
        // Validate all mappings first to preserve single-insert semantics
        for mapping in &mappings {
            if let Err(e) = mapping.validate() {
                return Err(Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Invalid permission mapping in bulk store: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                )));
            }
        }

        // Collect ids for a compact existence check (each id uniquely maps to a normalized string)
        let mut id_strs: Vec<String> = Vec::with_capacity(mappings.len());
        for m in &mappings {
            id_strs.push(m.permission_id().as_u64().to_string());
        }

        // Fetch existing records by permission_id IN ids in a single batched query
        let mut existing_models: Vec<seaorm_permission_mapping::Model> = Vec::new();
        if !id_strs.is_empty() {
            let found = seaorm_permission_mapping::Entity::find()
                .filter(seaorm_permission_mapping::Column::PermissionId.is_in(id_strs.clone()))
                .all(&self.db)
                .await
                .map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Query,
                        format!("Failed to query existing permission mappings by id: {}", e),
                        Some(TableName::AxumGatePermissionMappings.to_string()),
                        None,
                    ))
                })?;
            existing_models.extend(found);
        }

        // Build set of existing ids for quick lookup (normalized strings are implied by ids)
        let mut existing_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for m in existing_models {
            existing_ids.insert(m.permission_id);
        }

        // Determine which mappings are new (not present by id). Since permission_id is
        // a deterministic mapping for normalized strings, checking ids is sufficient.
        let mut to_insert: Vec<PermissionMapping> = Vec::new();
        for m in mappings {
            let pid = m.permission_id().as_u64().to_string();
            if existing_ids.contains(&pid) {
                // skip existing
                continue;
            }
            to_insert.push(m);
        }

        // If nothing to insert, return early
        if to_insert.is_empty() {
            return Ok(Vec::new());
        }

        // Convert to ActiveModel list for insert_many
        let mut active_models: Vec<seaorm_permission_mapping::ActiveModel> =
            Vec::with_capacity(to_insert.len());
        let mut insert_pids: Vec<String> = Vec::with_capacity(to_insert.len());
        for pm in &to_insert {
            active_models.push(seaorm_permission_mapping::ActiveModel::from(pm.clone()));
            insert_pids.push(pm.permission_id().as_u64().to_string());
        }

        // Try insert_many with returning (preferred). If the DB or driver supports
        // returning, `exec_with_returning` yields a TryInsertResult which must be
        // matched. If the driver doesn't support returning, fall back to exec() and
        // then query the inserted rows. Use on_conflict_do_nothing to make the operation
        // idempotent in the presence of concurrent writers.
        // Execute a single bulk insert with ON CONFLICT DO NOTHING, then select the
        // rows by permission_id to return the domain objects. This avoids dealing
        // with the DB/driver-specific exec_with_returning TryInsertResult variants.
        seaorm_permission_mapping::Entity::insert_many(active_models)
            .on_conflict_do_nothing()
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Failed to execute bulk insert: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        // Query back the models for the inserted permission_ids. This will return the
        // newly inserted rows or existing rows if concurrent writers inserted them.
        let models = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::PermissionId.is_in(insert_pids.clone()))
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!(
                        "Failed to query permission mappings after bulk insert: {}",
                        e
                    ),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        let mut out: Vec<PermissionMapping> = Vec::with_capacity(models.len());
        for m in models {
            let dom = PermissionMapping::try_from(m).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to convert permission mapping: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;
            out.push(dom);
        }

        Ok(out)
    }

    async fn remove_mappings_by_ids(
        &self,
        ids: Vec<PermissionId>,
    ) -> crate::errors::Result<Vec<PermissionMapping>> {
        // Convert requested ids into strings for DB comparison
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_strs: Vec<String> = ids.iter().map(|id| id.as_u64().to_string()).collect();

        // Fetch all models that match any of the provided permission_ids in one query
        let models = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::PermissionId.is_in(id_strs.clone()))
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mappings for bulk delete: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        if models.is_empty() {
            return Ok(Vec::new());
        }

        // Collect primary keys for deletion
        let pk_ids: Vec<i32> = models.iter().map(|m| m.id).collect();

        // Delete the matching rows by primary key in a single operation if supported
        seaorm_permission_mapping::Entity::delete_many()
            .filter(seaorm_permission_mapping::Column::Id.is_in(pk_ids.clone()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mappings in bulk: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        // Convert fetched models into domain objects to return what was removed
        let mut removed: Vec<PermissionMapping> = Vec::with_capacity(models.len());
        for m in models {
            let dom = PermissionMapping::try_from(m).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to convert deleted permission mapping: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;
            removed.push(dom);
        }

        Ok(removed)
    }

    async fn query_mappings_by_ids(
        &self,
        ids: Vec<PermissionId>,
    ) -> crate::errors::Result<Vec<PermissionMapping>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let id_strs: Vec<String> = ids.iter().map(|id| id.as_u64().to_string()).collect();
        let models = seaorm_permission_mapping::Entity::find()
            .filter(seaorm_permission_mapping::Column::PermissionId.is_in(id_strs))
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mappings in bulk: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;

        let mut out = Vec::with_capacity(models.len());
        for m in models {
            let dom = PermissionMapping::try_from(m).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to convert permission mapping: {}", e),
                    Some(TableName::AxumGatePermissionMappings.to_string()),
                    None,
                ))
            })?;
            out.push(dom);
        }

        Ok(out)
    }
}

impl<T> GroupRepositoryTrait<T> for SeaOrmRepository
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    async fn store_group(&self, group: T) -> Result<bool> {
        // Convert to ActiveModel (will serialize payload)
        let mut model = seaorm_group::ActiveModel::from(group);
        model.id = ActiveValue::NotSet;
        // Try insert and handle unique constraint as "already exists"
        match model.insert(&self.db).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                if msg.contains("unique") && msg.contains("constraint") {
                    Ok(false)
                } else {
                    Err(Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Insert,
                        format!("Failed to insert group: {}", e),
                        Some(TableName::AxumGateGroups.to_string()),
                        None,
                    )))
                }
            }
        }
    }

    async fn delete_group(&self, id: &str) -> Result<Option<T>> {
        // Find existing model by logical group_id
        let model_opt = seaorm_group::Entity::find()
            .filter(seaorm_group::Column::GroupId.eq(id.to_string()))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query group for deletion: {}", e),
                    Some(TableName::AxumGateGroups.to_string()),
                    Some(id.to_string()),
                ))
            })?;

        let model = match model_opt {
            Some(m) => m,
            None => return Ok(None),
        };

        seaorm_group::Entity::delete_by_id(model.id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete group: {}", e),
                    Some(TableName::AxumGateGroups.to_string()),
                    Some(id.to_string()),
                ))
            })?;

        // Deserialize stored model payload into domain T
        let dom = model.into_payload().map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!("Failed to deserialize deleted group: {}", e),
                Some(TableName::AxumGateGroups.to_string()),
                Some(id.to_string()),
            ))
        })?;
        Ok(Some(dom))
    }

    async fn update_group(&self, group: T) -> Result<Option<T>> {
        let gid = group.group_id().to_string();
        let model_opt = seaorm_group::Entity::find()
            .filter(seaorm_group::Column::GroupId.eq(gid.clone()))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query group for update: {}", e),
                    Some(TableName::AxumGateGroups.to_string()),
                    Some(gid.clone()),
                ))
            })?;

        let model = match model_opt {
            Some(m) => m,
            None => return Ok(None),
        };

        let mut active = model.into_active_model();
        let new_active = seaorm_group::ActiveModel::from(group);
        active.payload = new_active.payload;

        let updated = active.update(&self.db).await.map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Update,
                format!("Failed to update group: {}", e),
                Some(TableName::AxumGateGroups.to_string()),
                Some(gid.clone()),
            ))
        })?;

        let dom = updated.into_payload().map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Update,
                format!("Failed to deserialize updated group: {}", e),
                Some(TableName::AxumGateGroups.to_string()),
                Some(gid.clone()),
            ))
        })?;
        Ok(Some(dom))
    }

    async fn query_group_by_id(&self, id: &str) -> Result<Option<T>> {
        let model_opt = seaorm_group::Entity::find()
            .filter(seaorm_group::Column::GroupId.eq(id.to_string()))
            .one(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query group by id: {}", e),
                    Some(TableName::AxumGateGroups.to_string()),
                    Some(id.to_string()),
                ))
            })?;

        let model = match model_opt {
            Some(m) => m,
            None => return Ok(None),
        };

        let dom = model.into_payload().map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to deserialize group payload: {}", e),
                Some(TableName::AxumGateGroups.to_string()),
                Some(id.to_string()),
            ))
        })?;
        Ok(Some(dom))
    }

    async fn query_all_groups(&self) -> Result<Vec<T>> {
        let models = seaorm_group::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query all groups: {}", e),
                    Some(TableName::AxumGateGroups.to_string()),
                    None,
                ))
            })?;

        let mut out = Vec::with_capacity(models.len());
        for m in models {
            let dom = m.into_payload().map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to deserialize group payload: {}", e),
                    Some(TableName::AxumGateGroups.to_string()),
                    None,
                ))
            })?;
            out.push(dom);
        }
        Ok(out)
    }
}
