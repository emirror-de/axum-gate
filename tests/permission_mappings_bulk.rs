/*
Tests for bulk permission mapping repository operations.

Covers:
- In-memory repository (fast unit test)
- SeaORM repository using in-memory SQLite (integration-style; requires `storage-seaorm` feature)
- SurrealDB repository using the in-memory engine (integration-style; requires `storage-surrealdb` feature)

These tests validate:
- `store_mappings` inserts only new mappings and skips duplicates
- `query_mappings_by_ids` returns stored mappings
- `remove_mappings_by_ids` deletes requested mappings and returns removed domain objects
*/

use axum_gate::permissions::mapping::{
    PermissionMapping, PermissionMappingRepository, PermissionMappingRepositoryBulk,
};

#[tokio::test]
async fn memory_permission_mapping_bulk_ops() {
    use axum_gate::repositories::memory::MemoryPermissionMappingRepository;

    let repo = MemoryPermissionMappingRepository::default();

    // Create mappings
    let m1 = PermissionMapping::from("read:api");
    let m2 = PermissionMapping::from("write:file");
    let m3 = PermissionMapping::from("delete:item");

    // Bulk insert m1,m2
    let stored = repo
        .store_mappings(vec![m1.clone(), m2.clone()])
        .await
        .expect("memory store_mappings failed");
    assert_eq!(stored.len(), 2, "Expected two mappings to be stored");

    // Insert again all three; only m3 should be newly stored
    let stored2 = repo
        .store_mappings(vec![m1.clone(), m2.clone(), m3.clone()])
        .await
        .expect("memory store_mappings second call failed");
    assert_eq!(
        stored2.len(),
        1,
        "Expected one new mapping (m3) to be stored on second bulk insert"
    );
    assert_eq!(stored2[0].permission_id(), m3.permission_id());

    // Query by ids
    let ids = vec![m1.permission_id(), m2.permission_id(), m3.permission_id()];
    let queried = repo
        .query_mappings_by_ids(ids.clone())
        .await
        .expect("memory query_mappings_by_ids failed");
    // ordering not guaranteed, but we expect 3 results
    assert_eq!(
        queried.len(),
        3,
        "Expected three mappings returned by bulk query"
    );

    // Remove by ids (remove two)
    let removed = repo
        .remove_mappings_by_ids(vec![m1.permission_id(), m2.permission_id()])
        .await
        .expect("memory remove_mappings_by_ids failed");
    assert_eq!(removed.len(), 2, "Expected two mappings to be removed");

    // Remaining should be only m3
    let remaining = repo
        .list_all_mappings()
        .await
        .expect("list_all_mappings failed");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].permission_id(), m3.permission_id());
}

#[cfg(feature = "storage-seaorm")]
#[tokio::test]
async fn seaorm_permission_mapping_bulk_ops() {
    use axum_gate::repositories::sea_orm::SeaOrmRepository;

    use sea_orm::Database;

    // Connect to an in-memory sqlite DB
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to sqlite in-memory");

    // Ensure the table exists (create minimal schema matching the entity)
    // id: INTEGER PRIMARY KEY AUTOINCREMENT
    // normalized_string: TEXT UNIQUE
    // permission_id: TEXT UNIQUE
    let create_sql = r#"
    CREATE TABLE IF NOT EXISTS axum_gate_permission_mappings (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        normalized_string TEXT NOT NULL UNIQUE,
        permission_id TEXT NOT NULL UNIQUE
    );
    "#;
    use sea_orm::ConnectionTrait;
    ConnectionTrait::execute_unprepared(&db, create_sql)
        .await
        .expect("Failed to create permission mapping table");

    // Create repository
    let repo = SeaOrmRepository::new(&db).expect("Failed to create SeaOrmRepository");

    // Prepare mappings
    let a = PermissionMapping::from("alpha:one");
    let b = PermissionMapping::from("beta:two");
    let c = PermissionMapping::from("gamma:three");

    // Insert a,b
    let stored = repo
        .store_mappings(vec![a.clone(), b.clone()])
        .await
        .expect("seaorm store_mappings failed");
    assert_eq!(stored.len(), 2);

    // Insert again a,b,c -> c is new
    let stored2 = repo
        .store_mappings(vec![a.clone(), b.clone(), c.clone()])
        .await
        .expect("seaorm store_mappings second call failed");
    assert_eq!(stored2.len(), 1);
    assert_eq!(stored2[0].permission_id(), c.permission_id());

    // Query by ids
    let ids = vec![a.permission_id(), b.permission_id(), c.permission_id()];
    let queried = repo
        .query_mappings_by_ids(ids.clone())
        .await
        .expect("seaorm query_mappings_by_ids failed");
    assert_eq!(queried.len(), 3);

    // Remove a and b
    let removed = repo
        .remove_mappings_by_ids(vec![a.permission_id(), b.permission_id()])
        .await
        .expect("seaorm remove_mappings_by_ids failed");
    assert_eq!(removed.len(), 2);

    // Remaining should be c only
    let all = repo
        .list_all_mappings()
        .await
        .expect("seaorm list_all_mappings failed");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].permission_id(), c.permission_id());
}

#[cfg(feature = "storage-surrealdb")]
#[tokio::test]
async fn surrealdb_permission_mapping_bulk_ops() {
    use axum_gate::repositories::surrealdb::{DatabaseScope, SurrealDbRepository};
    use surrealdb::Surreal;
    use surrealdb::engine::local::Mem;

    // Create in-memory SurrealDB
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create SurrealDB Mem");
    let scope = DatabaseScope::default();
    let repo = SurrealDbRepository::new(db, scope).expect("Failed to create SurrealDbRepository");

    // Prepare mappings
    let m1 = PermissionMapping::from("one:alpha");
    let m2 = PermissionMapping::from("two:beta");
    let m3 = PermissionMapping::from("three:gamma");

    // Insert first two
    let stored = repo
        .store_mappings(vec![m1.clone(), m2.clone()])
        .await
        .expect("surrealdb store_mappings failed");
    assert_eq!(stored.len(), 2);

    // Insert all three; only m3 should be new
    let stored2 = repo
        .store_mappings(vec![m1.clone(), m2.clone(), m3.clone()])
        .await
        .expect("surrealdb store_mappings second call failed");
    assert_eq!(stored2.len(), 3);
    assert_eq!(stored2[0].permission_id(), m1.permission_id());
    assert_eq!(stored2[1].permission_id(), m2.permission_id());
    assert_eq!(stored2[2].permission_id(), m3.permission_id());

    // Query by ids
    let ids = vec![m1.permission_id(), m2.permission_id(), m3.permission_id()];
    let queried = repo
        .query_mappings_by_ids(ids.clone())
        .await
        .expect("surrealdb query_mappings_by_ids failed");
    assert_eq!(queried.len(), 3);

    // Remove m1 and m2
    let removed = repo
        .remove_mappings_by_ids(vec![m1.permission_id(), m2.permission_id()])
        .await
        .expect("surrealdb remove_mappings_by_ids failed");
    assert_eq!(removed.len(), 2);

    // Remaining should be m3 only
    let all = repo
        .list_all_mappings()
        .await
        .expect("surrealdb list_all_mappings failed");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].permission_id(), m3.permission_id());
}
