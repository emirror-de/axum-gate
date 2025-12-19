#![cfg(feature = "storage-surrealdb")]

use axum_gate::permissions::{
    PermissionId,
    mapping::{PermissionMapping, PermissionMappingRepository},
};
use axum_gate::repositories::surrealdb::{DatabaseScope, SurrealDbRepository};

use surrealdb::Surreal;
use surrealdb::engine::local::Mem;

#[tokio::test]
async fn surrealdb_permission_mapping_crud_and_queries() {
    // Prepare SurrealDB in-memory instance and repository
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create SurrealDB Mem engine");

    let scope = DatabaseScope::default();
    let repo = SurrealDbRepository::new(db, scope.clone()).unwrap();

    // 1) Store a mapping
    let mapping = PermissionMapping::from("Read:API");
    let id = mapping.permission_id();

    let stored = repo
        .store_mapping(mapping.clone())
        .await
        .expect("store_mapping failed");
    assert!(
        matches!(stored, Some(m) if m.permission_id() == id && m.normalized_string() == "read:api"),
        "Expected stored mapping to match input"
    );

    // 2) Uniqueness (storing again should be a no-op)
    let stored_again = repo
        .store_mapping(mapping.clone())
        .await
        .expect("store_mapping duplicate failed");
    assert!(stored_again.is_none(), "Duplicate store should return None");

    // 3) Query by ID
    let fetched_by_id = repo
        .query_mapping_by_id(id)
        .await
        .expect("query_mapping_by_id failed");
    assert!(
        matches!(fetched_by_id, Some(m) if m.permission_id() == id && m.normalized_string() == "read:api"),
        "Query by id should return the mapping"
    );

    // 4) Query by string (with different case/whitespace)
    let fetched_by_str = repo
        .query_mapping_by_string("  READ:api ")
        .await
        .expect("query_mapping_by_string failed");
    assert!(
        matches!(fetched_by_str, Some(m) if m.permission_id() == id && m.normalized_string() == "read:api"),
        "Query by string should normalize and return the mapping"
    );

    // 5) List all mappings (expect one)
    let all = repo
        .list_all_mappings()
        .await
        .expect("list_all_mappings failed");
    assert_eq!(all.len(), 1, "Expected exactly one mapping");
    assert_eq!(all[0].permission_id(), id);
    assert_eq!(all[0].normalized_string(), "read:api");

    // 6) Remove by string (with different case) and verify removal
    let removed_by_str = repo
        .remove_mapping_by_string("READ:API")
        .await
        .expect("remove_mapping_by_string failed");
    assert!(
        matches!(removed_by_str, Some(m) if m.permission_id() == id),
        "Expected remove by string to return the mapping"
    );

    // Further removal should be a no-op
    let removed_again = repo
        .remove_mapping_by_string("read:api")
        .await
        .expect("remove_mapping_by_string second call failed");
    assert!(removed_again.is_none(), "Second remove should return None");

    // After removal, queries should return None
    assert!(
        repo.query_mapping_by_id(id)
            .await
            .expect("query by id after removal failed")
            .is_none(),
        "Query by id should return None after removal"
    );
    assert!(
        repo.query_mapping_by_string("read:api")
            .await
            .expect("query by string after removal failed")
            .is_none(),
        "Query by string should return None after removal"
    );

    // 7) Store another mapping and remove by id
    let mapping2 = PermissionMapping::from("write:file");
    let id2 = mapping2.permission_id();

    let stored2 = repo
        .store_mapping(mapping2.clone())
        .await
        .expect("store_mapping mapping2 failed");
    assert!(stored2.is_some(), "Expected second mapping to be stored");

    // Remove by ID
    let removed_by_id = repo
        .remove_mapping_by_id(id2)
        .await
        .expect("remove_mapping_by_id failed");
    assert!(
        matches!(removed_by_id, Some(m) if m.permission_id() == id2),
        "Expected remove by id to return the mapping"
    );

    // Nothing should remain
    let all_after = repo
        .list_all_mappings()
        .await
        .expect("list_all_mappings after removals failed");
    assert!(all_after.is_empty(), "Expected no mappings after removals");
}

#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn surrealdb_permission_mapping_uniqueness() {
    // Prepare SurrealDB in-memory instance and repository
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create SurrealDB Mem engine");
    let scope = DatabaseScope::default();
    let repo = SurrealDbRepository::new(db, scope).unwrap();

    // Store a mapping
    let m1 = PermissionMapping::from("Read:Api");
    let id1 = m1.permission_id();
    assert_eq!(m1.normalized_string(), "read:api");

    let stored1 = repo
        .store_mapping(m1.clone())
        .await
        .expect("store m1 failed");
    assert!(stored1.is_some(), "First store should succeed");

    // Attempt to store equivalent mapping with different case/whitespace
    let m1_equiv = PermissionMapping::from("  read:API  ");
    assert_eq!(m1_equiv.permission_id(), id1);
    assert_eq!(m1_equiv.normalized_string(), "read:api");

    let stored_equiv = repo
        .store_mapping(m1_equiv)
        .await
        .expect("store m1_equiv failed");
    assert!(
        stored_equiv.is_none(),
        "Equivalent mapping should not be stored (uniqueness by id/string)"
    );

    // Ensure only one mapping exists
    let all = repo
        .list_all_mappings()
        .await
        .expect("list_all_mappings failed");
    assert_eq!(
        all.len(),
        1,
        "Expected exactly one mapping after uniqueness checks"
    );
    assert_eq!(all[0].permission_id(), id1);

    // Removing a non-existent mapping by id should return None
    let nonexistent_id = PermissionId::from("does:not:exist");
    assert!(
        repo.remove_mapping_by_id(nonexistent_id)
            .await
            .expect("remove non-existent by id failed")
            .is_none(),
        "Removing non-existent id should return None"
    );

    // Removing a non-existent mapping by string should return None
    assert!(
        repo.remove_mapping_by_string("does:not:exist")
            .await
            .expect("remove non-existent by string failed")
            .is_none(),
        "Removing non-existent string should return None"
    );
}
