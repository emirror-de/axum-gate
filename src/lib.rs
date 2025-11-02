#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! # axum-gate
//!
//! Flexible, type-safe authentication and authorization for axum using JWTs and optional OAuth2.
//! Supports cookie and bearer authentication, plus an OAuth2 Authorization Code + PKCE login flow
//! that mints a first-party JWT cookie for browser sessions. Designed for single nodes and
//! distributed systems with multiple storage backends.
//!
//! ## Key Features
//!
//! - **Cookie and bearer JWT authentication** - Choose HTTP-only cookies or Authorization: Bearer
//! - **OAuth2 login flow builder** - Authorization Code + PKCE; mints first-party JWT cookies
//! - **Role-based access control** - Hierarchical roles with supervisor inheritance
//! - **Group-based access control** - Organize users by teams, departments, or projects
//! - **Permission system** - Fine-grained permissions with deterministic hashing
//! - **Multiple storage backends** - In-memory, SurrealDB, SeaORM support
//! - **Distributed system ready** - Zero-synchronization permission system
//! - **Pre-built handlers** - Login/logout endpoints with timing attack protection
//! - **Optional anonymous context** - Install `Option<Account>` and `Option<RegisteredClaims>`
//! - **Static token mode** - Simple shared-secret bearer auth for internal services
//! - **Audit and metrics (feature-gated)** - Structured audit logs and Prometheus metrics
//!
//! ### Re-exports
//! This crate re-exports selected external crates (e.g., `jsonwebtoken`, `cookie`, `uuid`, `axum_extra`, and, behind a feature flag, `prometheus`) because types from these crates are part of this crate’s public API. Keeping these re-exports is intentional so users can import the exposed types from a single namespace.
//!
//! ### Prelude
//! A convenience prelude is available via `axum_gate::prelude::*` that re-exports the most commonly used types.
//!
//! ### Feature Flags
//! - `storage-surrealdb` — SurrealDB repositories (see [BUSL-1.1 license note](https://github.com/emirror-de/axum-gate?tab=readme-ov-file#msrv-and-license))
//! - `storage-seaorm` — SeaORM repositories
//! - `audit-logging` — emit structured audit events
//! - `prometheus` — export metrics for audit logging (implies `audit-logging`)
//! - `insecure-fast-hash` — faster Argon2 preset for development only (opt-in for release, not recommended)
//! - `aws_lc_rs`: Uses AWS Libcrypto for JWT cryptographic operations
//!
//!
//! For common integration issues and debugging tips, [see the Troubleshooting guide](https://github.com/emirror-de/axum-gate/blob/nightly/TROUBLESHOOTING.md).
//!
//! ## Quick Start
//!
//! ```rust
//! use axum::{routing::get, Router};
//! use axum_gate::prelude::*;
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Set up storage (dev-friendly in-memory backends)
//!     let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
//!     let secret_repo = Arc::new(MemorySecretRepository::new_with_argon2_hasher().unwrap());
//!
//!     // Create a JWT codec. Use a persistent key in production (e.g., env/secret manager).
//!     let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-key".to_string());
//!     let options = JsonWebTokenOptions {
//!         enc_key: jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
//!         dec_key: jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
//!         header: None,
//!         validation: None,
//!     };
//!     let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(options));
//!
//!     // Protect routes with role-based access (cookie auth)
//!     let app = Router::<()>::new()
//!         .route("/admin", get(admin_handler))
//!         .layer(
//!             Gate::cookie::<_, Role, Group>("my-app", jwt)
//!                 .with_policy(AccessPolicy::require_role(Role::Admin))
//!                 .configure_cookie_template(|tpl| tpl.name("auth-token"))
//!                 .unwrap(),
//!         );
//! }
//!
//! async fn admin_handler() -> &'static str { "Admin access granted!" }
//! ```
//!
//! ## Access Control
//!
//! ### Role-Based Access
//! ```rust
//! use axum_gate::prelude::{Role, Group, AccessPolicy};
//!
//! // Single role requirement
//! let policy = AccessPolicy::<Role, Group>::require_role(Role::Admin);
//!
//! // Multiple role options
//! let policy = AccessPolicy::<Role, Group>::require_role(Role::Admin)
//!     .or_require_role(Role::Moderator);
//!
//! // Hierarchical access (role + all supervisor roles)
//! let policy = AccessPolicy::<Role, Group>::require_role_or_supervisor(Role::User);
//! ```
//!
//! ### Group-Based Access
//! ```rust
//! use axum_gate::prelude::{Role, Group, AccessPolicy};
//!
//! let policy = AccessPolicy::<Role, Group>::require_group(Group::new("engineering"))
//!     .or_require_group(Group::new("management"));
//! ```
//!
//! ### Permission-Based Access
//! ```rust
//! use axum_gate::prelude::{Role, Group, AccessPolicy, PermissionId};
//!
//! // Validate permissions at compile-time (checks for hash collisions)
//! axum_gate::validate_permissions!["read:api", "write:api", "admin:system"];
//!
//! // Use in access policies
//! let policy = AccessPolicy::<Role, Group>::require_permission(PermissionId::from("read:api"));
//! ```
//!
//! ### Convenient Login Check
//! ```rust
//! use axum_gate::prelude::*;
//! use std::sync::Arc;
//!
//! # let jwt_codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! // Allow any authenticated user (all roles: User, Reporter, Moderator, Admin)
//! let gate = Gate::cookie::<_, Role, Group>("my-app", jwt_codec)
//!     .require_login()  // Convenience method for any logged-in user
//!     .configure_cookie_template(|tpl| tpl.name("auth-token"))
//!     .unwrap();
//! ```
//!
//! ## Authentication Modes
//!
//! ### Cookie (Optional User Context)
//! For routes that should never be blocked but may use authenticated context when present:
//!
//! ```rust
//! use axum::{routing::get, Router, extract::Extension};
//! use axum_gate::prelude::*;
//! use axum_gate::codecs::jwt::RegisteredClaims;
//! use std::sync::Arc;
//!
//! async fn homepage(
//!     Extension(user_opt): Extension<Option<Account<Role, Group>>>,
//!     Extension(claims_opt): Extension<Option<RegisteredClaims>>,
//! ) -> String {
//!     if let (Some(user), Some(claims)) = (user_opt, claims_opt) {
//!         format!("Welcome back {} (token expires at {})", user.user_id, claims.expiration_time)
//!     } else {
//!         "Welcome anonymous visitor".into()
//!     }
//! }
//!
//! # let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! let app = Router::<()>::new()
//!     .route("/", get(homepage))
//!     .layer(
//!         Gate::cookie::<_, Role, Group>("my-app", jwt)
//!             .allow_anonymous_with_optional_user()
//!     );
//! ```
//!
//! ### Bearer (Strict, Optional, Static Token)
//! Strict bearer (JWT) example:
//! ```rust
//! # use axum::{routing::get, Router};
//! # use axum_gate::prelude::*;
//! # use std::sync::Arc;
//! # async fn handler() {}
//! let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! let app = Router::<()>::new()
//!     .route("/admin", get(handler))
//!     .layer(
//!         Gate::bearer("my-app", Arc::clone(&jwt))
//!             .with_policy(AccessPolicy::<Role, Group>::require_role(Role::Admin))
//!     );
//! ```
//!
//! Optional mode (never blocks; installs `Option<Account>` and `Option<RegisteredClaims>`):
//! ```rust
//! # use axum_gate::prelude::*;
//! # use std::sync::Arc;
//! let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! let gate = Gate::bearer::<_, Role, Group>("my-app", jwt).allow_anonymous_with_optional_user();
//! ```
//!
//! Static token mode (shared secret; useful for internal services):
//! ```rust
//! # use axum_gate::prelude::*;
//! # use std::sync::Arc;
//! # async fn handler() {}
//! let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//! let app = axum::Router::<()>::new()
//!     .route("/internal", axum::routing::get(handler))
//!     .layer(Gate::bearer::<_, Role, Group>("my-app", jwt).with_static_token("very-secret-token"));
//! ```
//!
//! ### OAuth2 (Authorization Code + PKCE → first-party JWT)
//! Minimal setup for mounting "/auth/login" and "/auth/callback":
//! ```rust
//! use axum::{Router, routing::get};
//! use axum_gate::prelude::*;
//! use std::sync::Arc;
//!
//! // Provide a JWT codec to mint the session cookie after successful callback.
//! let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//!
//! let oauth_routes = Gate::oauth2_with_jwt("my-app", jwt, 3600)
//!     .auth_url("https://provider.example.com/oauth2/authorize")
//!     .token_url("https://provider.example.com/oauth2/token")
//!     .client_id("CLIENT_ID")
//!     .client_secret("CLIENT_SECRET")
//!     .redirect_url("https://your.app/auth/callback")
//!     .add_scope("openid")
//!     .add_scope("email")
//!     // Map provider token response to your Account<Role, Group>:

//!     .with_account_mapper(|_token_resp| {
//!         Box::pin(async {
//!             // fetch userinfo as needed, then construct Account<Role, Group>
//!             Ok(Account::<Role, Group>::new("user@example.com", &[], &[]))
//!         })
//!     })
//!     .routes("/auth")
//!     .expect("valid oauth2 config");
//!
//! let app = Router::<()>::new().nest("/auth", oauth_routes);
//! ```
//!
//! ## Account Management
//!
//! ```rust
//! use axum_gate::accounts::AccountInsertService;
//! use axum_gate::permissions::Permissions;
//! use axum_gate::prelude::{Role, Group, Account};
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository};
//! use std::sync::Arc;
//!
//! # tokio_test::block_on(async {
//! # let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
//! # let secret_repo = Arc::new(MemorySecretRepository::new_with_argon2_hasher().unwrap());
//! // Create account with roles, groups, and permissions
//! let account = AccountInsertService::insert("user@example.com", "password")
//!     .with_roles(vec![Role::User])
//!     .with_groups(vec![Group::new("staff")])
//!     .with_permissions(Permissions::from_iter(["read:profile"]))
//!     .into_repositories(account_repo, secret_repo)
//!     .await;
//! # });
//! ```
//!
//! ## Storage Backends
//!
//! ### In-Memory (Development)
//! ```rust
//! use axum_gate::repositories::memory::{MemoryAccountRepository, MemorySecretRepository};
//! use axum_gate::prelude::{Role, Group};
//! use std::sync::Arc;
//!
//! let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
//! let secret_repo = Arc::new(MemorySecretRepository::new_with_argon2_hasher().unwrap());
//! ```
//!
//! ### SurrealDB (Feature: `storage-surrealdb`)
//! ```rust
//! # #[cfg(feature="storage-surrealdb")]
//! # {
//! use axum_gate::repositories::surrealdb::{DatabaseScope, SurrealDbRepository};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let db: surrealdb::Surreal<surrealdb::engine::any::Any> = todo!();
//! # let scope = DatabaseScope::default();
//! let repo = Arc::new(SurrealDbRepository::new(db, scope));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ### SeaORM (Feature: `storage-seaorm`)
//! ```rust
//! # #[cfg(feature="storage-seaorm")]
//! # {
//! use axum_gate::repositories::sea_orm::{SeaOrmRepository, models};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let db: sea_orm::DatabaseConnection = todo!();
//! let repo = Arc::new(SeaOrmRepository::new(&db));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Authentication Handlers
//!
//! Pre-built [`route_handlers::login`] and [`route_handlers::logout`] handlers integrate with
//! your storage backends and JWT configuration. See examples in the repository for complete
//! implementation patterns with dependency injection and routing setup.
//!
//! Note: The `login` handler is for username/password flows and is not used with
//! `OAuth2Gate` (which mounts its own `/login` and `/callback` routes). For sign-out,
//! the same `logout` handler remains applicable—as long as its `CookieTemplate` matches
//! the auth cookie name/template used by your OAuth2-issued first‑party JWT.
//!
//! ## User Data in Handlers
//!
//! ```rust
//! use axum::extract::Extension;
//! use axum_gate::codecs::jwt::RegisteredClaims;
//! use axum_gate::prelude::{Account, Role, Group};
//!
//! async fn profile_handler(
//!     Extension(user): Extension<Account<Role, Group>>,
//!     Extension(claims): Extension<RegisteredClaims>,
//! ) -> String {
//!     format!(
//!         "Hello {}, roles: {:?}, issued at: {}, expires: {}",
//!         user.user_id, user.roles, claims.issued_at_time, claims.expiration_time
//!     )
//! }
//! ```
//!
//! ## Security Features
//!
//! ### Cookie Security
//! - **Secure defaults**: [`CookieTemplate::recommended`](cookie_template::CookieTemplate::recommended) provides secure defaults
//! - **HTTPS enforcement**: `secure(true)` cookies in production
//! - **XSS protection**: `http_only(true)` prevents script access
//! - **CSRF mitigation**: `SameSite::Strict` for sensitive operations
//!
//! ### JWT Security
//! - **Persistent keys**: Use stable signing keys in production (see [`JsonWebToken`](codecs::jwt::JsonWebToken) docs)
//! - **Proper expiration**: Set reasonable JWT expiration times
//! - **Key rotation**: Support for periodic key updates
//!
//! ### Timing Attack Protection
//! Built-in protection against timing attacks:
//! - Constant-time credential verification using the [`subtle`] crate
//! - Always performs password verification, even for non-existent users
//! - Unified error responses prevent user enumeration
//! - Applied to all storage backends
//!
//! ### Audit and Metrics (feature-gated)
//! - Enable `audit-logging` to emit structured audit events for authentication flows
//! - Enable `prometheus` (implies `audit-logging`) to export metrics; in bearer mode you can
//!   also call `with_prometheus_metrics()` or `with_prometheus_registry(..)` on the gate builder
//! - Never log sensitive values (secrets, tokens, cookies); only high-level event metadata
//!
//! ## Permission System
//! - **Compile-time validation**: Use [`validate_permissions!`] macro for collision detection
//! - **Runtime validation**: [`permissions::PermissionCollisionChecker`] for dynamic permissions
//! - **Deterministic hashing**: No coordination needed between distributed nodes
//! - **Efficient storage**: Bitmap-based permission storage with fast lookups

pub use axum_extra;
pub use cookie;
pub use jsonwebtoken;
#[cfg(feature = "prometheus")]
pub use prometheus;
pub use uuid;

pub mod accounts;
#[cfg(feature = "audit-logging")]
pub mod audit;
pub mod authn;
pub mod authz;
pub mod codecs;
#[cfg(feature = "storage-seaorm")]
pub mod comma_separated_value;
pub mod cookie_template;
pub mod credentials;
pub mod gate;
pub mod groups;
pub mod hashing;
pub mod permissions;
pub mod repositories;
pub mod roles;
pub mod route_handlers;
pub mod secrets;
pub mod verification_result;

pub mod errors;
pub mod prelude;
