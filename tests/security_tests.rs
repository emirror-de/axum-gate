use axum_gate::jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use axum_gate::jwt::{JsonWebToken, JsonWebTokenOptions, JwtClaims, RegisteredClaims};
use axum_gate::hashing::{Argon2Hasher, VerificationResult};
use axum_gate::secrets::Secret;
use axum_gate::services::{CodecService, HashingService};
use axum_gate::{Account, Credentials, Group, Role};

use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use uuid::Uuid;

/// Tests for JWT security vulnerabilities and edge cases
mod jwt_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_with_tampered_signature() {
        let shared_secret = "SECURE_SECRET_KEY_FOR_TESTING";
        let jwt_codec = Arc::new(
            JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(JsonWebTokenOptions {
                enc_key: EncodingKey::from_secret(shared_secret.as_bytes()),
                dec_key: DecodingKey::from_secret(shared_secret.as_bytes()),
                header: Some(Header::default()),
                validation: Some(Validation::default()),
            }),
        );

        let account = Account::new("user@example.com", &[Role::User], &[Group::new("test")]);
        let registered_claims = RegisteredClaims::new(
            "test-issuer",
            (Utc::now() + TimeDelta::hours(1)).timestamp() as u64,
        );
        let claims = JwtClaims::new(account, registered_claims);

        // Create a valid JWT
        let jwt_bytes = jwt_codec.encode(&claims).unwrap();
        let jwt_string = String::from_utf8(jwt_bytes).unwrap();

        // Tamper with the signature by appending characters
        let tampered_jwt = format!("{}tampered", jwt_string);

        // Attempt to decode the tampered JWT should fail
        let result = jwt_codec.decode(tampered_jwt.as_bytes());
        assert!(result.is_err(), "Tampered JWT should not be valid");
    }

    #[tokio::test]
    async fn test_jwt_with_expired_token() {
        let shared_secret = "SECURE_SECRET_KEY_FOR_TESTING";
        let jwt_codec = Arc::new(
            JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(JsonWebTokenOptions {
                enc_key: EncodingKey::from_secret(shared_secret.as_bytes()),
                dec_key: DecodingKey::from_secret(shared_secret.as_bytes()),
                header: Some(Header::default()),
                validation: Some(Validation::default()),
            }),
        );

        let account = Account::new("user@example.com", &[Role::User], &[Group::new("test")]);
        // Create an already expired token
        let registered_claims = RegisteredClaims::new(
            "test-issuer",
            (Utc::now() - TimeDelta::hours(1)).timestamp() as u64, // Expired 1 hour ago
        );
        let claims = JwtClaims::new(account, registered_claims);

        // Create the JWT
        let jwt_bytes = jwt_codec.encode(&claims).unwrap();

        // Attempt to decode should fail due to expiration
        let result = jwt_codec.decode(&jwt_bytes);
        assert!(result.is_err(), "Expired JWT should not be valid");
    }

    #[tokio::test]
    async fn test_jwt_with_wrong_issuer() {
        let shared_secret = "SECURE_SECRET_KEY_FOR_TESTING";
        let jwt_codec = Arc::new(
            JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(JsonWebTokenOptions {
                enc_key: EncodingKey::from_secret(shared_secret.as_bytes()),
                dec_key: DecodingKey::from_secret(shared_secret.as_bytes()),
                header: Some(Header::default()),
                validation: Some(Validation::default()),
            }),
        );

        let account = Account::new("user@example.com", &[Role::User], &[Group::new("test")]);
        let registered_claims = RegisteredClaims::new(
            "wrong-issuer", // Different issuer
            (Utc::now() + TimeDelta::hours(1)).timestamp() as u64,
        );
        let claims = JwtClaims::new(account, registered_claims);

        let jwt_bytes = jwt_codec.encode(&claims).unwrap();
        let decoded_claims = jwt_codec.decode(&jwt_bytes).unwrap();

        // Test issuer validation
        assert!(!decoded_claims.has_issuer("expected-issuer"));
        assert!(decoded_claims.has_issuer("wrong-issuer"));
    }

    #[tokio::test]
    async fn test_jwt_with_different_secret() {
        let secret1 = "SECRET_KEY_1";
        let secret2 = "SECRET_KEY_2";
        
        let jwt_codec1 = Arc::new(
            JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(JsonWebTokenOptions {
                enc_key: EncodingKey::from_secret(secret1.as_bytes()),
                dec_key: DecodingKey::from_secret(secret1.as_bytes()),
                header: Some(Header::default()),
                validation: Some(Validation::default()),
            }),
        );

        let jwt_codec2 = Arc::new(
            JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(JsonWebTokenOptions {
                enc_key: EncodingKey::from_secret(secret2.as_bytes()),
                dec_key: DecodingKey::from_secret(secret2.as_bytes()),
                header: Some(Header::default()),
                validation: Some(Validation::default()),
            }),
        );

        let account = Account::new("user@example.com", &[Role::User], &[Group::new("test")]);
        let registered_claims = RegisteredClaims::new(
            "test-issuer",
            (Utc::now() + TimeDelta::hours(1)).timestamp() as u64,
        );
        let claims = JwtClaims::new(account, registered_claims);

        // Create JWT with first secret
        let jwt_bytes = jwt_codec1.encode(&claims).unwrap();

        // Try to decode with second secret - should fail
        let result = jwt_codec2.decode(&jwt_bytes);
        assert!(result.is_err(), "JWT signed with different secret should not be valid");
    }
}

/// Tests for password hashing security and timing attack protection
mod password_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing_produces_different_hashes() {
        let password = "test_password";
        let hasher = Argon2Hasher::default();

        // Hash the same password multiple times
        let hash1 = hasher.hash_value(password).unwrap();
        let hash2 = hasher.hash_value(password).unwrap();

        // Should produce different hashes due to salt
        assert_ne!(hash1, hash2, "Same password should produce different hashes due to salt");

        // But both should verify correctly
        assert_eq!(hasher.verify_value(password, &hash1).unwrap(), VerificationResult::Ok);
        assert_eq!(hasher.verify_value(password, &hash2).unwrap(), VerificationResult::Ok);
    }

    #[tokio::test]
    async fn test_password_verification_with_wrong_password() {
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        let hasher = Argon2Hasher::default();

        let hash = hasher.hash_value(correct_password).unwrap();

        assert_eq!(hasher.verify_value(correct_password, &hash).unwrap(), VerificationResult::Ok);
        assert_eq!(hasher.verify_value(wrong_password, &hash).unwrap(), VerificationResult::Unauthorized);
    }

    #[tokio::test]
    async fn test_empty_password_handling() {
        let hasher = Argon2Hasher::default();
        
        // Empty password should still be hashable
        let hash = hasher.hash_value("").unwrap();
        
        // And should verify correctly
        assert_eq!(hasher.verify_value("", &hash).unwrap(), VerificationResult::Ok);
        assert_eq!(hasher.verify_value("non-empty", &hash).unwrap(), VerificationResult::Unauthorized);
    }

    #[tokio::test]
    async fn test_malformed_hash_handling() {
        let hasher = Argon2Hasher::default();
        
        // Test with various malformed hashes
        let malformed_hashes = vec![
            "not_a_hash",
            "",
            "$$invalid$hash$format",
            "argon2$invalid",
        ];

        for malformed_hash in malformed_hashes {
            let result = hasher.verify_value("password", malformed_hash);
            assert!(result.is_err(), "Malformed hash '{}' should cause error", malformed_hash);
        }
    }

    #[tokio::test]
    async fn test_very_long_password() {
        let hasher = Argon2Hasher::default();
        
        // Test with a very long password (10KB)
        let long_password = "a".repeat(10_000);
        
        let hash = hasher.hash_value(&long_password).unwrap();
        assert_eq!(hasher.verify_value(&long_password, &hash).unwrap(), VerificationResult::Ok);
        
        // Different long password should fail
        let different_long_password = "b".repeat(10_000);
        assert_eq!(hasher.verify_value(&different_long_password, &hash).unwrap(), VerificationResult::Unauthorized);
    }
}

/// Tests for Secret model security
mod secret_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_secret_creation_and_verification() {
        let account_id = Uuid::now_v7();
        let password = "secure_password";
        let hasher = Argon2Hasher::default();

        let secret = Secret::new(&account_id, password, hasher).unwrap();
        
        assert_eq!(secret.account_id, account_id);
        assert_eq!(secret.verify(password, Argon2Hasher).unwrap(), VerificationResult::Ok);
        assert_eq!(secret.verify("wrong_password", Argon2Hasher).unwrap(), VerificationResult::Unauthorized);
    }

    #[tokio::test]
    async fn test_secret_from_hashed() {
        let account_id = Uuid::now_v7();
        let hasher = Argon2Hasher::default();
        let password = "test_password";
        
        let hashed_value = hasher.hash_value(password).unwrap();
        let secret = Secret::from_hashed(&account_id, &hashed_value);
        
        assert_eq!(secret.account_id, account_id);
        assert_eq!(secret.secret, hashed_value);
        assert_eq!(secret.verify(password, Argon2Hasher).unwrap(), VerificationResult::Ok);
    }

    #[tokio::test]
    async fn test_secret_with_unicode_password() {
        let account_id = Uuid::now_v7();
        let unicode_password = "пароль🔒密码";
        let hasher = Argon2Hasher::default();

        let secret = Secret::new(&account_id, unicode_password, hasher).unwrap();
        
        assert_eq!(secret.verify(unicode_password, Argon2Hasher).unwrap(), VerificationResult::Ok);
        assert_eq!(secret.verify("different", Argon2Hasher).unwrap(), VerificationResult::Unauthorized);
    }
}

/// Tests for Credentials security
mod credentials_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_credentials_creation() {
        let user_id = "user@example.com".to_string();
        let password = "secure_password";
        
        let creds = Credentials::new(&user_id, password);
        
        assert_eq!(creds.id, user_id);
        assert_eq!(creds.secret, password);
    }

    #[tokio::test]
    async fn test_credentials_with_special_characters() {
        let user_id = "user+tag@example.com".to_string();
        let password = "password!@#$%^&*()_+{}|:<>?[];',./";
        
        let creds = Credentials::new(&user_id, password);
        
        assert_eq!(creds.id, user_id);
        assert_eq!(creds.secret, password);
    }

    #[tokio::test]
    async fn test_credentials_with_empty_values() {
        let empty_user = "".to_string();
        let normal_user = "user".to_string();
        
        let creds1 = Credentials::new(&empty_user, "password");
        let creds2 = Credentials::new(&normal_user, "");
        let creds3 = Credentials::new(&empty_user, "");
        
        // Should be able to create credentials with empty values
        // Validation should happen at the authentication level
        assert_eq!(creds1.id, empty_user);
        assert_eq!(creds2.secret, "");
        assert_eq!(creds3.id, empty_user);
        assert_eq!(creds3.secret, "");
    }
}