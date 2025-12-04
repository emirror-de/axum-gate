use super::GroupEntity;
use crate::errors::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;

/// Repository abstraction for persisting and retrieving group entities.
///
/// This trait is generic over `T` so storage backends can persist domain-specific
/// group types as long as they implement `Serialize`, `DeserializeOwned` and
/// `GroupEntity`.
///
/// Semantics follow the repository patterns used elsewhere in the crate:
/// - Use `Ok(None)` (or `Ok(false)` for insert semantics) to report expected
///   absence/duplicate outcomes.
/// - Use `Err(..)` for exceptional backend failures (I/O, serialization, constraint).
pub trait GroupRepository<T>
where
    Self: Send + Sync,
    T: Serialize + DeserializeOwned + GroupEntity + Eq + Clone + Send + Sync,
{
    /// Persist a new group.
    ///
    /// Returns:
    /// - `Ok(true)` if the group was inserted
    /// - `Ok(false)` if a group with the same id already exists (no change)
    /// - `Err(e)` on backend failure
    fn store_group(&self, group: T) -> impl Future<Output = Result<bool>> + Send;

    /// Delete a group by its id (as returned by `GroupEntity::group_id`).
    ///
    /// Returns:
    /// - `Ok(Some(group))` if the group existed and was removed
    /// - `Ok(None)` if no group matched the provided id
    /// - `Err(e)` on backend failure
    fn delete_group(&self, id: &str) -> impl Future<Output = Result<Option<T>>> + Send;

    /// Update an existing group.
    ///
    /// Implementations may perform full replacement or partial updates depending
    /// on backend semantics. Returns:
    /// - `Ok(Some(updated_group))` on success
    /// - `Ok(None)` if the group does not exist
    /// - `Err(e)` on failure
    fn update_group(&self, group: T) -> impl Future<Output = Result<Option<T>>> + Send;

    /// Fetch a group by id.
    ///
    /// Returns:
    /// - `Ok(Some(group))` if found
    /// - `Ok(None)` if not found
    /// - `Err(e)` on backend failure
    fn query_group_by_id(&self, id: &str) -> impl Future<Output = Result<Option<T>>> + Send;

    /// Query all groups.
    ///
    /// Returns a vector with zero or more groups on success, or `Err` on failure.
    /// Implementations SHOULD document ordering semantics if any. For large
    /// datasets consider providing a separate paginated trait.
    fn query_all_groups(&self) -> impl Future<Output = Result<Vec<T>>> + Send;
}
