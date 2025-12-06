# axum-gate

[![Crates.io](https://img.shields.io/crates/v/axum-gate.svg)](https://crates.io/crates/axum-gate)
[![Documentation](https://docs.rs/axum-gate/badge.svg)](https://docs.rs/axum-gate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build Status](https://github.com/emirror-de/axum-gate/workflows/CI/badge.svg)](https://github.com/emirror-de/axum-gate/actions)

Flexible, type-safe authentication and authorization for Axum using JWTs and optional OAuth2.
- Cookie and bearer authentication
- OAuth2 Authorization Code + PKCE flow that issues first-party JWT cookies
- Hierarchical roles, groups, and string-based permissions
- Ready-to-use login/logout handlers
- Optional anonymous user context and static-token mode for internal services
- In-memory and optional database-backed repositories
- Feature-gated audit logging and Prometheus metrics

## Install

This crate re-exports jsonwebtoken, cookie, uuid, axum_extra (and optionally prometheus) because those types appear in the public API, and it provides a convenience prelude via axum_gate::prelude::*.


```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
axum-gate = { version = "1" }
```

Optional features:
- storage-surrealdb — SurrealDB repositories
- storage-seaorm — SeaORM repositories
- audit-logging — structured audit events
- prometheus — metrics for audit (implies audit-logging)
- insecure-fast-hash — faster Argon2 preset for development only

## Core concepts

- Gate layer
  - Gate::cookie("issuer", codec): JWT via HTTP-only cookies (web apps)
  - Gate::bearer("issuer", codec): JWT via Authorization: Bearer header (APIs)
  - Gate::bearer(...).with_static_token("..."): shared-secret mode (internal services)
  - Gate::oauth2::<R, G>(): OAuth2 Authorization Code + PKCE flow builder; or use Gate::oauth2_with_jwt("issuer", codec, ttl_secs) to also mint first-party JWT cookies
  - allow_anonymous_with_optional_user(): never block; inserts optional user context
  - require_login(): allow baseline role and all supervisors (hierarchy)
- Access policies
  - require_role(..), require_role_or_supervisor(..)
  - require_group(..)
  - require_permission("domain:action") — deterministic hashing to PermissionId; use validate_permissions! to catch collisions at test-time
- Login/logout
  - route_handlers::login: verifies credentials and sets the auth cookie
  - route_handlers::logout: removes the auth cookie
- Repositories
  - repositories::memory::{MemoryAccountRepository, MemorySecretRepository}
  - Optional SurrealDB / SeaORM backends via feature flags
- JWT codec
  - codecs::jwt::JsonWebToken with JsonWebTokenOptions (use persistent keys in production)

## Cryptographic Backend

This crate uses `rust_crypto` backend for JWT operations.

## Security

- Use a persistent JWT key (do not rely on the default random key in production)
- Keep the issuer consistent between Gate configuration and RegisteredClaims
- Use secure cookie settings in production (HttpOnly, Secure, appropriate SameSite)
- Rate-limit sensitive endpoints (e.g., login)
- Enable `audit-logging` and `prometheus` features for observability; never log secrets, tokens, or cookie values

## Examples and docs

Examples and complete usage are available in the crate documentation on docs.rs; the repository also includes curated examples (e.g., examples/oauth2-github for a full GitHub OAuth2 flow):
https://docs.rs/axum-gate

For common integration issues and practical debugging tips, see TROUBLESHOOTING.md (covers CookieGate, OAuth2 flows, and Bearer/Static token usage).

## MSRV and license

- MSRV: 1.88
- License: MIT
- SurrealDB (BUSL-1.1) notice:
  - Enabling the optional feature `storage-surrealdb` pulls in SurrealDB, which is licensed under the Business Source License 1.1 (not OSI-approved).
  - BUSL restricts Production Use unless allowed by the licensor or after the project's Change Date. This feature is off by default.
  - If you build or distribute binaries that enable this feature, you must comply with SurrealDB's BUSL terms or obtain a commercial license.
  - When distributing binaries that include SurrealDB, include third-party notices and the SurrealDB license text.
  - For fully open-source distributions, prefer the `memory`, `storage-seaorm` backends.
- subtle license notice:
  - The license for the subtle dependency is provided in the NOTICE file in this repository. When redistributing, retain the NOTICE contents as required by that license.
