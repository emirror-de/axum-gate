use axum_gate::jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use axum_gate::jwt::{JsonWebToken, JsonWebTokenOptions, JwtClaims, RegisteredClaims};
use axum_gate::services::AccountInsertService;
use axum_gate::storage::memory::{MemoryAccountStorage, MemorySecretStorage};
use axum_gate::{Account, Credentials, Gate, Group, Role, cookie};
use http::header;

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Extension, Json};
use axum::http::{self, Request, StatusCode};
use axum::routing::{Router, get, post};
use chrono::{TimeDelta, Utc};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tower::Service;

/// Additional permissions for testing
#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
#[non_exhaustive]
pub enum TestPermission {
    ReadData,
    WriteData,
    DeleteData,
    AdminAccess,
}

const ISSUER: &str = "test-auth-node";
const COOKIE_NAME: &str = "test-axum-gate";

async fn protected_endpoint(Extension(user): Extension<Account<Role, Group>>) -> Result<String, ()> {
    Ok(format!("Protected access granted to {}", user.user_id))
}

async fn admin_only_endpoint(Extension(user): Extension<Account<Role, Group>>) -> Result<String, ()> {
    Ok(format!("Admin access granted to {}", user.user_id))
}

async fn permission_based_endpoint(Extension(user): Extension<Account<Role, Group>>) -> Result<String, ()> {
    Ok(format!("Permission-based access granted to {}", user.user_id))
}

async fn setup_test_app() -> Router {
    let shared_secret = "TEST_SECRET_KEY_FOR_SECURITY_TESTING";
    let jwt_codec = Arc::new(
        JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(JsonWebTokenOptions {
            enc_key: EncodingKey::from_secret(shared_secret.as_bytes()),
            dec_key: DecodingKey::from_secret(shared_secret.as_bytes()),
            header: Some(Header::default()),
            validation: Some(Validation::default()),
        }),
    );

    let account_storage = Arc::new(MemoryAccountStorage::default());
    let secrets_storage = Arc::new(MemorySecretStorage::default());

    // Create test users with different roles and permissions
    AccountInsertService::insert("admin@test.com", "admin_password")
        .with_roles(vec![Role::Admin])
        .with_groups(vec![Group::new("admin")])
        .with_permissions(vec![TestPermission::AdminAccess])
        .into_storages(Arc::clone(&account_storage), Arc::clone(&secrets_storage))
        .await
        .unwrap();

    AccountInsertService::insert("user@test.com", "user_password")
        .with_roles(vec![Role::User])
        .with_groups(vec![Group::new("users")])
        .with_permissions(vec![TestPermission::ReadData])
        .into_storages(Arc::clone(&account_storage), Arc::clone(&secrets_storage))
        .await
        .unwrap();

    AccountInsertService::insert("reporter@test.com", "reporter_password")
        .with_roles(vec![Role::Reporter])
        .with_groups(vec![Group::new("reporters")])
        .into_storages(Arc::clone(&account_storage), Arc::clone(&secrets_storage))
        .await
        .unwrap();

    let cookie_template = cookie::CookieBuilder::new(COOKIE_NAME, "").secure(true);

    Router::new()
        .route(
            "/protected",
            get(protected_endpoint).layer(
                Gate::new_cookie(ISSUER, Arc::clone(&jwt_codec))
                    .with_cookie_template(cookie_template.clone())
                    .grant_role(Role::User)
                    .grant_role(Role::Admin)
                    .grant_role(Role::Reporter),
            ),
        )
        .route(
            "/admin-only",
            get(admin_only_endpoint).layer(
                Gate::new_cookie(ISSUER, Arc::clone(&jwt_codec))
                    .with_cookie_template(cookie_template.clone())
                    .grant_role(Role::Admin),
            ),
        )
        .route(
            "/permission-based",
            get(permission_based_endpoint).layer(
                Gate::new_cookie(ISSUER, Arc::clone(&jwt_codec))
                    .with_cookie_template(cookie_template.clone())
                    .grant_permission(TestPermission::ReadData),
            ),
        )
        .route(
            "/login",
            post({
                let registered_claims = RegisteredClaims::new(
                    ISSUER,
                    (Utc::now() + TimeDelta::hours(1)).timestamp() as u64,
                );
                let secrets_storage = Arc::clone(&secrets_storage);
                let account_storage = Arc::clone(&account_storage);
                let jwt_codec = Arc::clone(&jwt_codec);
                let cookie_template = cookie_template.clone();
                move |cookie_jar, request_credentials: Json<Credentials<String>>| {
                    axum_gate::route_handlers::login(
                        cookie_jar,
                        request_credentials,
                        registered_claims,
                        secrets_storage,
                        account_storage,
                        jwt_codec,
                        cookie_template,
                    )
                }
            }),
        )
}

/// Tests for authorization bypass attempts and edge cases
mod authorization_bypass_tests {
    use super::*;

    #[tokio::test]
    async fn test_access_without_authentication() {
        let mut app = setup_test_app().await;

        let protected_routes = vec!["/protected", "/admin-only", "/permission-based"];

        for route in protected_routes {
            let request = Request::builder()
                .method(http::Method::GET)
                .uri(route)
                .body(Body::empty())
                .unwrap();

            let response = app.call(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Route {} should require authentication", route
            );
        }
    }

    #[tokio::test]
    async fn test_access_with_invalid_cookie() {
        let mut app = setup_test_app().await;

        let invalid_cookies = vec![
            "invalid_jwt_token",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
            "",
            "null",
            "undefined",
        ];

        for invalid_cookie in invalid_cookies {
            let request = Request::builder()
                .method(http::Method::GET)
                .uri("/protected")
                .header(header::COOKIE, format!("{}={}", COOKIE_NAME, invalid_cookie))
                .body(Body::empty())
                .unwrap();

            let response = app.call(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Invalid cookie '{}' should not grant access", invalid_cookie
            );
        }
    }

    #[tokio::test]
    async fn test_role_escalation_attempt() {
        let mut app = setup_test_app().await;

        // Login as regular user
        let login_response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&Credentials::new(
                            &"user@test.com".to_string(),
                            "user_password",
                        ))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(login_response.status(), StatusCode::OK);
        let cookie_value = login_response.headers().get("set-cookie").unwrap();

        // Try to access admin-only endpoint with user credentials
        let request = Request::builder()
            .method(http::Method::GET)
            .uri("/admin-only")
            .header(header::COOKIE, cookie_value)
            .body(Body::empty())
            .unwrap();

        let response = app.call(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "User should not be able to access admin-only endpoint"
        );
    }

    #[tokio::test]
    async fn test_permission_bypass_attempt() {
        let mut app = setup_test_app().await;

        // Login as reporter (who doesn't have ReadData permission)
        let login_response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&Credentials::new(
                            &"reporter@test.com".to_string(),
                            "reporter_password",
                        ))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(login_response.status(), StatusCode::OK);
        let cookie_value = login_response.headers().get("set-cookie").unwrap();

        // Try to access permission-based endpoint without required permission
        let request = Request::builder()
            .method(http::Method::GET)
            .uri("/permission-based")
            .header(header::COOKIE, cookie_value)
            .body(Body::empty())
            .unwrap();

        let response = app.call(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Reporter without ReadData permission should not access permission-based endpoint"
        );
    }

    #[tokio::test]
    async fn test_cookie_manipulation_attempts() {
        let mut app = setup_test_app().await;

        // Login as user to get a valid cookie
        let login_response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&Credentials::new(
                            &"user@test.com".to_string(),
                            "user_password",
                        ))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookie_header = login_response.headers().get("set-cookie").unwrap().to_str().unwrap();
        
        // Extract the JWT value from the Set-Cookie header
        let jwt_start = cookie_header.find('=').unwrap() + 1;
        let jwt_end = cookie_header.find(';').unwrap_or(cookie_header.len());
        let jwt_value = &cookie_header[jwt_start..jwt_end];

        // Test various cookie manipulation attempts
        let manipulated_cookies = vec![
            format!("{}extra", jwt_value), // Append to JWT
            format!("tampered{}", jwt_value), // Prepend to JWT
            jwt_value.replace('.', "X"), // Replace JWT separator
            format!("{}=modified", COOKIE_NAME), // Different cookie value
        ];

        for manipulated_cookie in manipulated_cookies {
            let request = Request::builder()
                .method(http::Method::GET)
                .uri("/protected")
                .header(header::COOKIE, format!("{}={}", COOKIE_NAME, manipulated_cookie))
                .body(Body::empty())
                .unwrap();

            let response = app.call(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "Manipulated cookie should not grant access"
            );
        }
    }
}

/// Tests for input validation and edge cases
mod input_validation_tests {
    use super::*;

    #[tokio::test]
    async fn test_login_with_malformed_json() {
        let mut app = setup_test_app().await;

        let malformed_payloads = vec![
            "not_json",
            "{invalid_json}",
            "{\"id\":}",
            "{\"secret\":\"password\"}",
            "{\"id\":\"user\"}",
            "",
            "null",
            "[]",
        ];

        for payload in malformed_payloads {
            let response = app
                .call(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri("/login")
                        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                        .body(Body::from(payload))
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should return 400 Bad Request or 422 Unprocessable Entity
            assert!(
                response.status() == StatusCode::BAD_REQUEST || 
                response.status() == StatusCode::UNPROCESSABLE_ENTITY,
                "Malformed JSON '{}' should be rejected", payload
            );
        }
    }

    #[tokio::test]
    async fn test_login_with_sql_injection_attempts() {
        let mut app = setup_test_app().await;

        let sql_injection_attempts = vec![
            "' OR '1'='1",
            "'; DROP TABLE users; --",
            "' UNION SELECT * FROM secrets --",
            "admin'--",
            "' OR 1=1 --",
        ];

        for injection_attempt in sql_injection_attempts {
            let response = app
                .call(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri("/login")
                        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                        .body(Body::from(
                            serde_json::to_string(&Credentials::new(
                                &injection_attempt.to_string(),
                                "password",
                            ))
                            .unwrap(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should return NOT_FOUND (user doesn't exist) or UNAUTHORIZED
            assert!(
                response.status() == StatusCode::NOT_FOUND || 
                response.status() == StatusCode::UNAUTHORIZED,
                "SQL injection attempt '{}' should be safely handled", injection_attempt
            );
        }
    }

    #[tokio::test]
    async fn test_login_with_extremely_long_credentials() {
        let mut app = setup_test_app().await;

        // Test with very long user ID and password
        let long_user_id = "a".repeat(10_000);
        let long_password = "b".repeat(10_000);

        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&Credentials::new(&long_user_id, &long_password))
                            .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should handle gracefully without crashing
        assert!(
            response.status() == StatusCode::NOT_FOUND || 
            response.status() == StatusCode::UNAUTHORIZED ||
            response.status() == StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn test_login_with_unicode_and_special_characters() {
        let mut app = setup_test_app().await;

        let special_credentials = vec![
            ("user@example.com", "пароль🔒"),
            ("用户@example.com", "password"),
            ("user@例え.com", "password"),
            ("user+tag@example.com", "password!@#$%"),
        ];

        for (user_id, password) in special_credentials {
            let response = app
                .call(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri("/login")
                        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                        .body(Body::from(
                            serde_json::to_string(&Credentials::new(&user_id.to_string(), password))
                                .unwrap(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should handle Unicode gracefully
            assert!(
                response.status() == StatusCode::NOT_FOUND || 
                response.status() == StatusCode::UNAUTHORIZED,
                "Unicode credentials should be handled safely"
            );
        }
    }
}

/// Tests for timing attack protection (informational - may be flaky in CI environments)
mod timing_attack_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    #[ignore] // Timing tests can be flaky in CI environments
    async fn test_login_timing_consistency() {
        let mut app = setup_test_app().await;

        // Test login timing for existing vs non-existing users
        // to ensure no timing information leakage
        
        let existing_user_creds = Credentials::new(&"user@test.com".to_string(), "wrong_password");
        let nonexistent_user_creds = Credentials::new(&"nonexistent@test.com".to_string(), "wrong_password");

        // Measure timing for existing user with wrong password
        let start = Instant::now();
        let _response1 = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&existing_user_creds).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let existing_user_time = start.elapsed();

        // Measure timing for non-existent user
        let start = Instant::now();
        let _response2 = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&nonexistent_user_creds).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let nonexistent_user_time = start.elapsed();

        // Allow for some variation but not excessive timing differences
        // In a real implementation, you might want to add artificial delays to normalize timing
        let timing_ratio = if existing_user_time > nonexistent_user_time {
            existing_user_time.as_millis() as f64 / (nonexistent_user_time.as_millis().max(1)) as f64
        } else {
            nonexistent_user_time.as_millis() as f64 / (existing_user_time.as_millis().max(1)) as f64
        };

        // This is more of an informational test - significant timing differences
        // could indicate potential timing attack vulnerabilities
        println!(
            "Timing analysis: existing user: {}ms, non-existing user: {}ms, ratio: {}", 
            existing_user_time.as_millis(), 
            nonexistent_user_time.as_millis(),
            timing_ratio
        );

        // For this test to be meaningful in production, you would want to
        // implement constant-time comparison or add artificial delays
    }
}