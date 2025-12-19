use crate::errors::Result;
use crate::groups::{GroupEntity, GroupRepository};

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;

/// In-memory repository for group entities.
///
/// This implementation is generic over `T` where `T: Serialize + DeserializeOwned + GroupEntity`.
/// It provides the same semantics as other in-memory repositories: thread-safe, fast,
/// and suitable for tests and development.
#[derive(Clone)]
pub struct MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    store: Arc<RwLock<HashMap<String, T>>>,
}

impl<T> Default for MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> From<Vec<T>> for MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    fn from(value: Vec<T>) -> Self {
        let mut map = HashMap::new();
        for v in value {
            map.insert(v.group_id().to_string(), v);
        }
        Self {
            store: Arc::new(RwLock::new(map)),
        }
    }
}

impl<T> GroupRepository<T> for MemoryGroupRepository<T>
where
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync + 'static,
{
    async fn store_group(&self, group: T) -> Result<bool> {
        let id = group.group_id().to_string();

        // Fast read check to avoid acquiring write lock unnecessarily
        {
            let read = self.store.read().await;
            if read.contains_key(&id) {
                return Ok(false);
            }
        }

        let mut write = self.store.write().await;
        write.insert(id, group);
        Ok(true)
    }

    async fn delete_group(&self, id: &str) -> Result<Option<T>> {
        let mut write = self.store.write().await;
        Ok(write.remove(id))
    }

    async fn update_group(&self, group: T) -> Result<Option<T>> {
        let id = group.group_id().to_string();
        let mut write = self.store.write().await;
        if write.contains_key(&id) {
            write.insert(id.clone(), group.clone());
            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    async fn query_group_by_id(&self, id: &str) -> Result<Option<T>> {
        let read = self.store.read().await;
        Ok(read.get(id).cloned())
    }

    async fn query_all_groups(&self) -> Result<Vec<T>> {
        let read = self.store.read().await;
        Ok(read.values().cloned().collect())
    }
}
