//! Permission set implementation that can change during runtime.

use crate::services::DynamicPermissionService;

use anyhow::Result;
use tokio::sync::RwLock;
use tracing::warn;

/// Provides a permission set that is able to extend during runtime.
///
/// The permissions are stored as `String` and its index is used for the permssion on the
/// [Gate](crate::Gate) side.
///
/// See [DynamicPermissionService] for more information about resource access that changes over
/// time.
pub struct PermissionSet {
    /// The permissions can be any string you need for your application.
    permissions: RwLock<Vec<String>>,
}

impl PermissionSet {
    /// Creates a new permission set from the given values.
    pub fn new(permissions: Vec<String>) -> Self {
        Self {
            permissions: RwLock::new(permissions),
        }
    }

    /// Checks whether the permission is already inserted.
    async fn permission_available(&self, permission: &str) -> bool {
        let read = self.permissions.read().await;
        read.contains(&permission.to_string())
    }
}

impl DynamicPermissionService for PermissionSet {
    async fn append_permission(&self, permission: &str) -> Result<()> {
        if self.permission_available(permission).await {
            warn!("Permission not appended because it is already available.");
            return Ok(());
        }
        self.permissions.write().await.push(permission.to_string());
        Ok(())
    }

    async fn extend_permission_set(&self, permissions: Vec<String>) -> Result<()> {
        for perm in permissions {
            self.append_permission(&perm).await?;
        }
        Ok(())
    }

    async fn permission_index(&self, permission: &str) -> Result<Option<u32>> {
        Ok(self
            .permissions
            .read()
            .await
            .iter()
            .position(|p| p == permission)
            .map(|p| p as u32))
    }

    async fn permission_name(&self, permission: u32) -> Result<Option<String>> {
        Ok(self
            .permissions
            .read()
            .await
            .get(permission as usize)
            .map(|p| p.to_owned()))
    }
}

#[tokio::test]
async fn dynamic_permission_set() {
    let permissions = vec!["read:dir:path", "write:file:path"]
        .into_iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>();

    let set = PermissionSet::new(permissions.clone());
    assert_eq!(
        set.permission_name(0).await.unwrap(),
        Some("read:dir:path".to_string())
    );
    assert_eq!(
        set.permission_index("write:file:path")
            .await
            .unwrap(),
        Some(1)
    );
    set.append_permission("write:file:path")
        .await
        .unwrap();
    assert_eq!(
        set.permission_index("write:file:path")
            .await
            .unwrap(),
        Some(1)
    );
}
