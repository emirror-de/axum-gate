use axum_gate::cookie::CookieBuilder; 
use axum_gate::storage::memory::{MemoryAccountStorage, MemorySecretStorage};
use axum_gate::services::{AccountInsertService, AccountDeleteService, AccountStorageService, SecretStorageService, CredentialsVerifierService};
use axum_gate::hashing::{Argon2Hasher, VerificationResult};
use axum_gate::secrets::Secret;
use axum_gate::utils::AccessHierarchy;
use axum_gate::{Account, Credentials, Group, Role};

use std::sync::Arc;

use uuid::Uuid;

/// Tests for cookie security attributes and handling
mod cookie_security_tests {
    use super::*;

    #[test]
    fn test_cookie_builder_security_attributes() {
        let cookie = CookieBuilder::new("test-cookie", "test-value")
            .secure(true)
            .http_only(true)
            .same_site(cookie::SameSite::Strict)
            .build();

        assert_eq!(cookie.name(), "test-cookie");
        assert_eq!(cookie.value(), "test-value");
        assert!(cookie.secure().unwrap_or(false), "Cookie should be marked as secure");
        assert!(cookie.http_only().unwrap_or(false), "Cookie should be HTTP-only");
        assert_eq!(cookie.same_site(), Some(cookie::SameSite::Strict));
    }

    #[test]
    fn test_cookie_builder_with_path_and_domain() {
        let cookie = CookieBuilder::new("auth-cookie", "jwt-token")
            .path("/api")
            .domain("example.com")
            .secure(true)
            .build();

        assert_eq!(cookie.path(), Some("/api"));
        assert_eq!(cookie.domain(), Some("example.com"));
        assert!(cookie.secure().unwrap_or(false));
    }

    #[test]
    fn test_cookie_builder_same_site_variants() {
        let strict_cookie = CookieBuilder::new("strict", "value")
            .same_site(cookie::SameSite::Strict)
            .build();
        assert_eq!(strict_cookie.same_site(), Some(cookie::SameSite::Strict));

        let lax_cookie = CookieBuilder::new("lax", "value")
            .same_site(cookie::SameSite::Lax)
            .build();
        assert_eq!(lax_cookie.same_site(), Some(cookie::SameSite::Lax));

        let none_cookie = CookieBuilder::new("none", "value")
            .same_site(cookie::SameSite::None)
            .secure(true) // Required for SameSite=None
            .build();
        assert_eq!(none_cookie.same_site(), Some(cookie::SameSite::None));
        assert!(none_cookie.secure().unwrap_or(false));
    }

    #[test]
    fn test_cookie_with_empty_or_special_values() {
        let empty_cookie = CookieBuilder::new("empty", "")
            .secure(true)
            .build();
        assert_eq!(empty_cookie.value(), "");

        let special_chars_cookie = CookieBuilder::new("special", "value=with;special,chars")
            .secure(true)
            .build();
        assert_eq!(special_chars_cookie.value(), "value=with;special,chars");
    }
}

/// Tests for storage layer security
mod storage_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_isolation() {
        // Create two separate storage instances
        let storage1 = Arc::new(MemoryAccountStorage::<Role, Group>::default());
        let storage2 = Arc::new(MemoryAccountStorage::<Role, Group>::default());

        let account1 = Account::new("user1@example.com", &[Role::User], &[Group::new("group1")]);
        let account2 = Account::new("user2@example.com", &[Role::Admin], &[Group::new("group2")]);

        // Store accounts in different storage instances
        let _stored_account1 = storage1.store_account(account1).await.unwrap();
        let _stored_account2 = storage2.store_account(account2).await.unwrap();

        // Each storage should only have its own account
        assert!(storage1.query_account_by_user_id("user1@example.com").await.unwrap().is_some());
        assert!(storage1.query_account_by_user_id("user2@example.com").await.unwrap().is_none());

        assert!(storage2.query_account_by_user_id("user2@example.com").await.unwrap().is_some());
        assert!(storage2.query_account_by_user_id("user1@example.com").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_memory_storage_concurrent_access() {
        let storage = Arc::new(MemoryAccountStorage::<Role, Group>::default());

        // Create and store accounts concurrently
        let account1 = Account::new("user1@example.com", &[Role::User], &[Group::new("group1")]);
        let account2 = Account::new("user2@example.com", &[Role::Admin], &[Group::new("group2")]);
        
        let storage1 = Arc::clone(&storage);
        let storage2 = Arc::clone(&storage);
        
        let handle1 = tokio::spawn(async move {
            storage1.store_account(account1).await.unwrap()
        });
        
        let handle2 = tokio::spawn(async move {
            storage2.store_account(account2).await.unwrap()
        });

        // Wait for both tasks to complete
        let (_result1, _result2) = tokio::try_join!(handle1, handle2).unwrap();
        
        // Verify both accounts were stored
        assert!(storage.query_account_by_user_id("user1@example.com").await.unwrap().is_some());
        assert!(storage.query_account_by_user_id("user2@example.com").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_secret_storage_security() {
        let storage = Arc::new(MemorySecretStorage::default());
        let account_id = Uuid::now_v7();
        let password = "sensitive_password";

        let secret = Secret::new(&account_id, password, Argon2Hasher).unwrap();
        storage.store_secret(secret).await.unwrap();

        // Verify correct password works
        let correct_creds = Credentials::new(&account_id, password);
        assert_eq!(
            storage.verify_credentials(correct_creds).await.unwrap(),
            VerificationResult::Ok
        );

        // Verify wrong password fails
        let wrong_creds = Credentials::new(&account_id, "wrong_password");
        assert_eq!(
            storage.verify_credentials(wrong_creds).await.unwrap(),
            VerificationResult::Unauthorized
        );
    }

    #[tokio::test]
    async fn test_storage_with_duplicate_user_ids() {
        let storage = Arc::new(MemoryAccountStorage::<Role, Group>::default());
        let user_id = "duplicate@example.com";

        let account1 = Account::new(user_id, &[Role::User], &[Group::new("group1")]);
        let account2 = Account::new(user_id, &[Role::Admin], &[Group::new("group2")]);

        // Store first account
        let _stored1 = storage.store_account(account1).await.unwrap();
        
        // Store second account with same user_id
        let _stored2 = storage.store_account(account2).await.unwrap();

        // Query should return one of them (implementation dependent)
        let queried = storage.query_account_by_user_id(user_id).await.unwrap();
        assert!(queried.is_some());
    }

    #[tokio::test]
    async fn test_account_deletion_security() {
        let account_storage = Arc::new(MemoryAccountStorage::<Role, Group>::default());
        let secret_storage = Arc::new(MemorySecretStorage::default());

        // Create an account with secret
        let account = AccountInsertService::insert("user@example.com", "password")
            .with_roles(vec![Role::User])
            .into_storages(Arc::clone(&account_storage), Arc::clone(&secret_storage))
            .await
            .unwrap()
            .unwrap();

        let user_id = account.user_id.clone();

        // Verify account exists
        assert!(account_storage.query_account_by_user_id(&user_id).await.unwrap().is_some());

        // Delete the account using AccountDeleteService
        AccountDeleteService::delete(account)
            .from_storages(Arc::clone(&account_storage), Arc::clone(&secret_storage))
            .await
            .unwrap();

        // Verify account is deleted
        assert!(account_storage.query_account_by_user_id(&user_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_storage_with_malformed_data() {
        let storage = Arc::new(MemorySecretStorage::default());

        // Test with invalid UUID (this should be handled by the type system)
        // but we can test error handling in credential verification
        let non_existent_id = Uuid::now_v7();
        let creds = Credentials::new(&non_existent_id, "password");

        let result = storage.verify_credentials(creds).await.unwrap();
        assert_eq!(result, VerificationResult::Unauthorized);
    }
}

/// Tests for role and permission security
mod access_control_security_tests {
    use super::*;

    // Helper function to check if one role supervises another
    fn supervises<T: AccessHierarchy + PartialEq>(supervisor: &T, subordinate: &T) -> bool {
        let mut current = supervisor.subordinate();
        while let Some(role) = current {
            if role == *subordinate {
                return true;
            }
            current = role.subordinate();
        }
        false
    }

    #[test]
    fn test_role_hierarchy_security() {
        // Test that role hierarchy is correctly implemented
        assert!(supervises(&Role::Admin, &Role::Reporter));
        assert!(supervises(&Role::Admin, &Role::User));
        assert!(supervises(&Role::Reporter, &Role::User));
        
        // Test that roles don't supervise themselves
        assert!(!supervises(&Role::User, &Role::User));
        assert!(!supervises(&Role::Admin, &Role::Admin));
        
        // Test inverse relationships
        assert!(!supervises(&Role::User, &Role::Admin));
        assert!(!supervises(&Role::User, &Role::Reporter));
        assert!(!supervises(&Role::Reporter, &Role::Admin));
    }

    #[tokio::test]
    async fn test_account_with_multiple_roles() {
        let account = Account::new(
            "multiRole@example.com",
            &[Role::User, Role::Reporter],
            &[Group::new("users"), Group::new("reporters")],
        );

        assert_eq!(account.roles.len(), 2);
        assert!(account.roles.contains(&Role::User));
        assert!(account.roles.contains(&Role::Reporter));
        
        assert_eq!(account.groups.len(), 2);
        assert!(account.groups.contains(&Group::new("users")));
        assert!(account.groups.contains(&Group::new("reporters")));
    }

    #[test]
    fn test_group_security() {
        let group1 = Group::new("admin");
        let group2 = Group::new("admin");
        let group3 = Group::new("user");

        // Groups with same name should be equal
        assert_eq!(group1, group2);
        assert_ne!(group1, group3);

        // Test group name validation
        let empty_group = Group::new("");
        assert_eq!(empty_group.name(), "");

        let special_chars_group = Group::new("admin-group_123");
        assert_eq!(special_chars_group.name(), "admin-group_123");
    }

    #[tokio::test]
    async fn test_permission_boundary_enforcement() {
        use num_enum::{IntoPrimitive, TryFromPrimitive};

        #[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
        #[repr(u32)]
        enum TestPermission {
            Read = 0,
            Write = 1,
            Delete = 2,
        }

        // Test permission conversion
        let read_perm: u32 = TestPermission::Read.into();
        let write_perm: u32 = TestPermission::Write.into();
        
        assert_ne!(read_perm, write_perm);
        
        // Test reverse conversion
        let read_from_u32: TestPermission = TestPermission::try_from(0).unwrap();
        assert_eq!(read_from_u32, TestPermission::Read);
        
        // Test invalid permission
        let invalid_perm = TestPermission::try_from(999);
        assert!(invalid_perm.is_err());
    }
}

/// Tests for edge cases and error conditions
mod edge_case_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_account_with_empty_collections() {
        let account: Account<Role, Group> = Account::new("user@example.com", &[], &[]);
        
        assert!(account.roles.is_empty());
        assert!(account.groups.is_empty());
        assert!(account.permissions.is_empty());
    }

    #[tokio::test]
    async fn test_credentials_with_uuid_and_string_id() {
        let uuid_id = Uuid::now_v7();
        let string_id = "user@example.com".to_string();

        let uuid_creds = Credentials::new(&uuid_id, "password");
        let string_creds = Credentials::new(&string_id, "password");

        assert_eq!(uuid_creds.id, uuid_id);
        assert_eq!(string_creds.id, string_id);
    }

    #[tokio::test]
    async fn test_very_long_user_identifiers() {
        let long_user_id = "a".repeat(1000);
        let account = Account::new(&long_user_id, &[Role::User], &[Group::new("test")]);
        
        assert_eq!(account.user_id, long_user_id);
    }

    #[tokio::test]
    async fn test_unicode_user_identifiers() {
        let unicode_user_id = "用户@例え.com";
        let account = Account::new(unicode_user_id, &[Role::User], &[Group::new("unicode")]);
        
        assert_eq!(account.user_id, unicode_user_id);
    }

    #[tokio::test]
    async fn test_account_serialization_security() {
        use serde_json;
        
        let account = Account::new(
            "user@example.com",
            &[Role::Admin],
            &[Group::new("admin")]
        );

        // Test that account can be serialized and deserialized
        let serialized = serde_json::to_string(&account).unwrap();
        assert!(serialized.contains("user@example.com"));
        assert!(serialized.contains("Admin"));
        
        let deserialized: Account<Role, Group> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.user_id, account.user_id);
        assert_eq!(deserialized.roles, account.roles);
        assert_eq!(deserialized.groups, account.groups);
    }

    #[tokio::test]
    async fn test_secret_serialization_security() {
        use serde_json;
        
        let account_id = Uuid::now_v7();
        let secret = Secret::new(&account_id, "password", Argon2Hasher).unwrap();

        // Test that secret can be serialized (for storage)
        let serialized = serde_json::to_string(&secret).unwrap();
        
        // Ensure the serialized data doesn't contain the plain password
        assert!(!serialized.contains("password"));
        assert!(serialized.contains("$argon2")); // Should contain hash prefix
        
        let deserialized: Secret = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.account_id, secret.account_id);
        assert_eq!(deserialized.secret, secret.secret);
    }
}