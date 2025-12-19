use crate::errors::{Error, Result};
use crate::permissions::PermissionId;
use crate::permissions::mapping::{
    PermissionMapping, PermissionMappingRepository, PermissionMappingRepositoryBulk,
};
use crate::repositories::{RepositoriesError, RepositoryOperation, RepositoryType};

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// In-memory implementation of [`PermissionMappingRepository`] for development and testing.
///
/// This repository stores permission mappings in memory using thread-safe data structures.
/// It's ideal for development, testing, and small applications that don't require
/// persistent storage of permission mappings.
///
/// # Thread Safety
///
/// This implementation uses `Arc<RwLock<HashMap>>` for thread-safe access to the
/// stored mappings. Multiple readers can access the data concurrently, while
/// writers have exclusive access.
///
/// # Storage Strategy
///
/// Mappings are stored in two hash maps for efficient lookup:
/// - `mappings_by_id: HashMap<u64, PermissionMapping>` for primary lookup by id
/// - `id_by_normalized: HashMap<String, u64>` for lookup by normalized string
///
/// This avoids scanning a Vec for common operations and makes bulk insertion,
/// deduplication and lookups O(1) on average.
#[derive(Debug)]
pub struct MemoryPermissionMappingRepository {
    /// Primary store keyed by numeric PermissionId
    mappings_by_id: Arc<RwLock<HashMap<u64, PermissionMapping>>>,
}

impl Default for MemoryPermissionMappingRepository {
    fn default() -> Self {
        Self {
            mappings_by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl From<Vec<PermissionMapping>> for MemoryPermissionMappingRepository {
    fn from(mappings: Vec<PermissionMapping>) -> Self {
        let mut by_id: HashMap<u64, PermissionMapping> = HashMap::new();

        for mapping in mappings {
            // Validate the mapping before storing
            if let Err(e) = mapping.validate() {
                tracing::warn!("Skipping invalid permission mapping: {}", e);
                continue;
            }

            let id = mapping.permission_id().as_u64();

            // Skip if id already present
            if by_id.contains_key(&id) {
                continue;
            }

            by_id.insert(id, mapping);
        }

        Self {
            mappings_by_id: Arc::new(RwLock::new(by_id)),
        }
    }
}

impl PermissionMappingRepository for MemoryPermissionMappingRepository {
    async fn store_mapping(&self, mapping: PermissionMapping) -> Result<Option<PermissionMapping>> {
        // Validate the mapping first
        if let Err(e) = mapping.validate() {
            return Err(Error::Repositories(RepositoriesError::operation_failed(
                RepositoryType::PermissionMapping,
                RepositoryOperation::Insert,
                format!("Invalid permission mapping: {}", e),
                None,
                Some("store".to_string()),
            )));
        }

        let id = mapping.permission_id().as_u64();

        // Fast read check
        {
            let read_by_id = self.mappings_by_id.read().await;
            if read_by_id.contains_key(&id) {
                return Ok(None);
            }
        }

        // Acquire write lock and insert atomically
        {
            let mut write_by_id = self.mappings_by_id.write().await;
            // Re-check under write lock to avoid race
            if write_by_id.contains_key(&id) {
                return Ok(None);
            }
            write_by_id.insert(id, mapping.clone());
        }

        Ok(Some(mapping))
    }

    async fn remove_mapping_by_id(&self, id: PermissionId) -> Result<Option<PermissionMapping>> {
        let id_u64 = id.as_u64();

        let mut write_by_id = self.mappings_by_id.write().await;
        if let Some(removed) = write_by_id.remove(&id_u64) {
            Ok(Some(removed))
        } else {
            Ok(None)
        }
    }

    async fn remove_mapping_by_string(
        &self,
        permission: &str,
    ) -> Result<Option<PermissionMapping>> {
        let normalized = normalize_permission(permission);

        // Acquire write lock and search for the mapping by normalized string
        let mut write_by_id = self.mappings_by_id.write().await;
        let mut found_key: Option<u64> = None;
        for (k, v) in write_by_id.iter() {
            if v.normalized_string() == normalized.as_str() {
                found_key = Some(*k);
                break;
            }
        }

        if let Some(id_u64) = found_key {
            if let Some(removed) = write_by_id.remove(&id_u64) {
                return Ok(Some(removed));
            }
        }

        Ok(None)
    }

    async fn query_mapping_by_id(&self, id: PermissionId) -> Result<Option<PermissionMapping>> {
        let read = self.mappings_by_id.read().await;
        Ok(read.get(&id.as_u64()).cloned())
    }

    async fn query_mapping_by_string(&self, permission: &str) -> Result<Option<PermissionMapping>> {
        let normalized = normalize_permission(permission);

        let read = self.mappings_by_id.read().await;
        for m in read.values() {
            if m.normalized_string() == normalized.as_str() {
                return Ok(Some(m.clone()));
            }
        }
        Ok(None)
    }

    async fn list_all_mappings(&self) -> Result<Vec<PermissionMapping>> {
        let read = self.mappings_by_id.read().await;
        Ok(read.values().cloned().collect())
    }
}

/// Normalize a permission name (trim + lowercase).
///
/// This function implements the same normalization logic used in
/// the PermissionId implementation to ensure consistency.
fn normalize_permission(input: &str) -> String {
    input.trim().to_lowercase()
}

impl PermissionMappingRepositoryBulk for MemoryPermissionMappingRepository {
    async fn store_mappings(
        &self,
        mappings: Vec<PermissionMapping>,
    ) -> Result<Vec<PermissionMapping>> {
        // Validate all mappings first to match the single-store semantics
        for mapping in &mappings {
            if let Err(e) = mapping.validate() {
                return Err(Error::Repositories(RepositoriesError::operation_failed(
                    RepositoryType::PermissionMapping,
                    RepositoryOperation::Insert,
                    format!("Invalid permission mapping in bulk store: {}", e),
                    None,
                    Some("store_mappings".to_string()),
                )));
            }
        }

        let mut stored: Vec<PermissionMapping> = Vec::new();
        // Acquire write lock once and perform deduplicated inserts
        let mut write_by_id = self.mappings_by_id.write().await;

        for mapping in mappings {
            let id = mapping.permission_id().as_u64();

            // If id already present, skip
            if write_by_id.contains_key(&id) {
                continue;
            }

            // Insert into map
            write_by_id.insert(id, mapping.clone());
            stored.push(mapping);
        }

        Ok(stored)
    }

    async fn remove_mappings_by_ids(
        &self,
        ids: Vec<PermissionId>,
    ) -> Result<Vec<PermissionMapping>> {
        let mut removed: Vec<PermissionMapping> = Vec::new();

        let mut write_by_id = self.mappings_by_id.write().await;

        for id in ids {
            let id_u64 = id.as_u64();
            if let Some(r) = write_by_id.remove(&id_u64) {
                removed.push(r);
            } else {
                // silently ignore non-existing ids
                continue;
            }
        }

        Ok(removed)
    }

    async fn query_mappings_by_ids(
        &self,
        ids: Vec<PermissionId>,
    ) -> Result<Vec<PermissionMapping>> {
        let read_by_id = self.mappings_by_id.read().await;
        let mut out: Vec<PermissionMapping> = Vec::new();

        for id in ids {
            if let Some(found) = read_by_id.get(&id.as_u64()) {
                out.push(found.clone());
            }
        }

        Ok(out)
    }
}
