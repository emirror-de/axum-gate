use super::SurrealDbRepository;
use crate::errors::{Error, Result};
use crate::permissions::PermissionId;
use crate::permissions::mapping::{
    PermissionMapping, PermissionMappingRepository, PermissionMappingRepositoryBulk,
};
use crate::repositories::{DatabaseError, DatabaseOperation};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use surrealdb::{Connection, RecordId};

/// Adapter for persisting `PermissionMapping` in SurrealDB.
///
/// SurrealDB can deserialize numeric fields as signed 64-bit integers (i64),
/// while our permission IDs are computed 64-bit values that may exceed the
/// positive i63 range. Persisting `permission_id` as a `String` avoids
/// signedness/width pitfalls across different SurrealDB backends and ensures
/// stable round‑trips regardless of how numbers are represented internally.
///
/// NOTE: The record key for permission mappings is the `permission_id`
/// (stringified). The `normalized_string` is stored as a regular field and
/// can be queried when reversing from human-readable permission names to ids.
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

        // Use permission_id as record key for the insert
        let record_id = RecordId::from_table_key(
            self.scope_settings.permission_mappings.clone(),
            mapping.permission_id().as_u64().to_string(),
        );

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

        // Delete directly by record id (permission_id used as key) and return the removed record (if any)
        let record_id = RecordId::from_table_key(
            self.scope_settings.permission_mappings.clone(),
            id.as_u64().to_string(),
        );
        let removed_spm: Option<SurrealPermissionMapping> =
            self.db.delete(record_id).await.map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mapping by id: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    Some(id.as_u64().to_string()),
                ))
            })?;

        removed_spm
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
        // We keep this behavior: remove by normalized_string field.
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

        // Direct select by record key (permission_id used as the record id)
        let record_id = RecordId::from_table_key(
            self.scope_settings.permission_mappings.clone(),
            id.as_u64().to_string(),
        );

        let mapping_spm: Option<SurrealPermissionMapping> =
            self.db.select(record_id).await.map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by id: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    Some(id.as_u64().to_string()),
                ))
            })?;

        mapping_spm
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

        // Query by normalized_string field (since the record key is now permission_id)
        let query = "SELECT * FROM type::table($table) WHERE normalized_string = $ns LIMIT 1";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("ns", normalized.clone()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mapping by string: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        let found: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract permission mapping by string: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                None,
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

impl<S> PermissionMappingRepositoryBulk for SurrealDbRepository<S>
where
    S: Connection,
{
    async fn store_mappings(
        &self,
        mappings: Vec<PermissionMapping>,
    ) -> Result<Vec<PermissionMapping>> {
        // Validate all mappings first (match single-store validation behavior)
        for mapping in &mappings {
            if let Err(e) = mapping.validate() {
                return Err(Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Insert,
                    format!("Invalid permission mapping in bulk store: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                )));
            }
        }

        self.use_ns_db().await?;

        // Short-circuit for empty input
        if mappings.is_empty() {
            return Ok(Vec::new());
        }

        // Collect permission_id strings for a single existence check (each ID maps to one normalized string)
        let ids: Vec<String> = mappings
            .iter()
            .map(|m| m.permission_id().as_u64().to_string())
            .collect();

        // Query once for any existing records that match any pid.
        // Using a single IN query reduces round-trips compared to per-item checks.
        let mut existing_pids: HashSet<String> = HashSet::new();
        if !ids.is_empty() {
            let query = "SELECT * FROM type::table($table) WHERE permission_id IN $pids";
            let mut res = self
                .db
                .query(query)
                .bind(("table", self.scope_settings.permission_mappings.clone()))
                .bind(("pids", ids.clone()))
                .await
                .map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Query,
                        format!("Failed to query existing mappings for bulk insert: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        None,
                    ))
                })?;

            let existing: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to extract existing mappings for bulk insert: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

            for e in existing {
                existing_pids.insert(e.permission_id);
            }
        }

        // Filter mappings to only those that don't already exist (checking ids is sufficient)
        let mut to_insert: Vec<PermissionMapping> = Vec::new();
        for m in mappings {
            let pid = m.permission_id().as_u64().to_string();
            if existing_pids.contains(&pid) {
                // skip existing
                continue;
            }
            to_insert.push(m);
        }

        // Insert remaining mappings. We still insert per-record because the client
        // driver does not expose a safe multi-insert here, but we've eliminated
        // the N existence-check round-trips.
        let mut stored: Vec<PermissionMapping> = Vec::new();
        for mapping in to_insert {
            // Use the permission_id string as record key
            let record_id = RecordId::from_table_key(
                self.scope_settings.permission_mappings.clone(),
                mapping.permission_id().as_u64().to_string(),
            );

            // Perform the insert. We don't need to convert the returned DB model
            // back into a domain object because the domain value we just inserted
            // (`mapping`) is authoritative and deterministic. If the insert returns
            // an object, it's a representation of the same data; we therefore
            // record the original domain object on success.
            let insert_res: Option<SurrealPermissionMapping> = self
                .db
                .insert(&record_id)
                .content(SurrealPermissionMapping::from(mapping.clone()))
                .await
                .map_err(|e| {
                    Error::Database(DatabaseError::with_context(
                        DatabaseOperation::Insert,
                        format!("Failed to store permission mapping in bulk: {}", e),
                        Some(self.scope_settings.permission_mappings.clone()),
                        None,
                    ))
                })?;

            if insert_res.is_some() {
                // Move the domain object into the returned list (no extra conversion)
                stored.push(mapping);
            } else {
                // unexpected but skip
                continue;
            }
        }

        Ok(stored)
    }

    async fn remove_mappings_by_ids(
        &self,
        ids: Vec<PermissionId>,
    ) -> Result<Vec<PermissionMapping>> {
        self.use_ns_db().await?;

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Convert ids to strings and perform a single DELETE ... IN (...) RETURN BEFORE
        // We keep using WHERE permission_id IN (...) since permission_id is also stored as a field.
        let pid_strs: Vec<String> = ids.iter().map(|id| id.as_u64().to_string()).collect();

        let query = "DELETE type::table($table) WHERE permission_id IN $pids RETURN BEFORE";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("pids", pid_strs.clone()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!("Failed to delete permission mappings in bulk: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        // Extract deleted records and convert to domain objects
        let removed_vec: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Delete,
                format!(
                    "Failed to extract deleted permission mappings in bulk: {}",
                    e
                ),
                Some(self.scope_settings.permission_mappings.clone()),
                None,
            ))
        })?;

        let mut removed: Vec<PermissionMapping> = Vec::with_capacity(removed_vec.len());
        for spm in removed_vec {
            let dom = PermissionMapping::try_from(spm).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Delete,
                    format!(
                        "Failed to convert deleted permission mapping in bulk: {}",
                        e
                    ),
                    Some(self.scope_settings.permission_mappings.clone()),
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
    ) -> Result<Vec<PermissionMapping>> {
        self.use_ns_db().await?;

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Build list of permission_id strings and query once using IN (we still use the field)
        let pid_strs: Vec<String> = ids.iter().map(|id| id.as_u64().to_string()).collect();
        let query = "SELECT * FROM type::table($table) WHERE permission_id IN $pids";
        let mut res = self
            .db
            .query(query)
            .bind(("table", self.scope_settings.permission_mappings.clone()))
            .bind(("pids", pid_strs.clone()))
            .await
            .map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to query permission mappings in bulk: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;

        let found: Vec<SurrealPermissionMapping> = res.take(0).map_err(|e| {
            Error::Database(DatabaseError::with_context(
                DatabaseOperation::Query,
                format!("Failed to extract permission mappings in bulk: {}", e),
                Some(self.scope_settings.permission_mappings.clone()),
                None,
            ))
        })?;

        let mut out: Vec<PermissionMapping> = Vec::with_capacity(found.len());
        for spm in found {
            let dom = PermissionMapping::try_from(spm).map_err(|e| {
                Error::Database(DatabaseError::with_context(
                    DatabaseOperation::Query,
                    format!("Failed to convert permission mapping in bulk query: {}", e),
                    Some(self.scope_settings.permission_mappings.clone()),
                    None,
                ))
            })?;
            out.push(dom);
        }

        Ok(out)
    }
}
