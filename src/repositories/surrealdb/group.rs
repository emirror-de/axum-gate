use super::SurrealDbRepository;
use crate::errors::{Error, Result};
use crate::groups::{GroupEntity, GroupRepository};
use crate::repositories::{DatabaseError, DatabaseOperation};

use serde::Serialize;
use serde::de::DeserializeOwned;
use surrealdb::{Connection, RecordId};

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
