use super::SeaOrmRepository;
use crate::errors::Error;
use crate::permissions::PermissionId;
use crate::permissions::mapping::{
    PermissionMapping, PermissionMappingRepository, PermissionMappingRepositoryBulk,
};
use crate::repositories::TableName;
use crate::repositories::sea_orm::models::permission_mapping as seaorm_permission_mapping;
use crate::repositories::{DatabaseError, DatabaseOperation};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, entity::ActiveModelTrait};

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

        // We can return ownership of the domain objects we prepared for insertion.
        // `to_insert` contains the PermissionMapping values that were not present
        // prior to the bulk insert. Because we constructed the ActiveModels from
        // clones of these domain objects and executed `ON CONFLICT DO NOTHING`,
        // the items in `to_insert` represent the intended stored mappings. Returning
        // them avoids an extra round-trip and redundant conversion from DB models.
        Ok(to_insert)
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
