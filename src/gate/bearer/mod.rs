//! Bearer gate implementation supporting two compile-time distinct modes:
//! - JWT bearer authentication & authorization (policy-based)
//! - Static shared-secret bearer token (boolean authorization)
//!
//! Each mode exposes only the relevant builder methods at compile time:
//!
//! JWT Mode (BearerGate<_, _, _, JwtConfig<_, _>>):
//!   - with_policy(...)
//!   - require_login()            (requires R: Default; baseline role + supervisors)
//!   - allow_anonymous_with_optional_user()
//!   - with_static_token(token)   (transitions to static token mode)
//!
//! Static Token Mode (BearerGate<_, _, _, StaticTokenConfig>):
//!   - allow_anonymous_with_optional_user()
//!
//! Optional Mode Semantics:
//!   - JWT optional: always forwards; inserts:
//!       * `Option<Account<R,G>>`
//!       * `Option<RegisteredClaims>`
//!   - Static token optional: always forwards; inserts:
//!       * StaticTokenAuthorized(bool)
//!
//! Strict Mode Semantics:
//!   - JWT strict: validates Authorization: Bearer `<jwt>`, enforces AccessPolicy;
//!     inserts `Account<R,G>` and `RegisteredClaims` on success, 401 otherwise
//!   - Static token strict: requires `Authorization: Bearer <exact_token>`;
//!     inserts `StaticTokenAuthorized(true)` on success, 401 otherwise
//!
//! Example (JWT strict):
//! ```rust
//! use std::sync::Arc;
//! use axum::Router;
//! use axum_gate::prelude::*;
//! let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//!
//! let router = Router::<()>::new();
//! let gate = Gate::bearer::<JsonWebToken::<JwtClaims<Account<Role, Group>>>, Role, Group>("my-app", codec)
//!     .with_policy(AccessPolicy::require_role(Role::Admin));
//! router.layer(gate);
//! ```
//!
//! Example (JWT optional):
//! ```rust
//! use std::sync::Arc;
//! use axum_gate::prelude::*;
//! let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//!
//! let gate = Gate::bearer::<JsonWebToken::<JwtClaims<Account<Role, Group>>>, Role, Group>("my-app", codec)
//!     .allow_anonymous_with_optional_user(); // Option<Account>, Option<RegisteredClaims>
//! ```
//!
//! Transition to static token mode (compile-time change of available methods):
//! ```rust
//! use std::sync::Arc;
//! use axum_gate::prelude::*;
//! let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//!
//! let gate = Gate::bearer::<JsonWebToken::<JwtClaims<Account<Role, Group>>>, Role, Group>("svc-a", codec)
//!     .with_static_token("shared-secret"); // now static token mode (no with_policy)
//! ```
//!
//! Static token optional:
//! ```rust
//! use std::sync::Arc;
//! use axum_gate::prelude::*;
//! let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
//!
//! let gate = Gate::bearer::<JsonWebToken::<JwtClaims<Account<Role, Group>>>, Role, Group>("svc-a", codec)
//!     .with_static_token("shared-secret")
//!     .allow_anonymous_with_optional_user(); // installs StaticTokenAuthorized(bool)
//! ```
//!
//! Handler extraction (static token optional):
//! ```rust
//! use axum_gate::gate::bearer::StaticTokenAuthorized;
//!
//! async fn handler(
//!     axum::Extension(token_auth): axum::Extension<StaticTokenAuthorized>
//! ) {
//!     if token_auth.is_authorized() { /* privileged */ } else { /* public */ }
//! }
//! ```

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::{body::Body, extract::Request, http::Response};
use http::StatusCode;
use subtle::ConstantTimeEq;
use tower::{Layer, Service};
use tracing::{debug, trace, warn};

pub use self::static_token_authorized::StaticTokenAuthorized;
use crate::accounts::Account;
use crate::authz::{AccessHierarchy, AccessPolicy, AuthorizationService};
use crate::codecs::Codec;
use crate::codecs::jwt::{JwtClaims, JwtValidationResult, JwtValidationService, RegisteredClaims};

mod static_token_authorized;

/// JWT mode configuration (compile-time).
#[derive(Clone)]
pub struct JwtConfig<R, G>
where
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq,
{
    policy: AccessPolicy<R, G>,
    optional: bool,
}

impl<R, G> std::fmt::Debug for JwtConfig<R, G>
where
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field("optional", &self.optional)
            .finish_non_exhaustive()
    }
}

/// Static token mode configuration (compile-time).
#[derive(Clone)]
pub struct StaticTokenConfig {
    token: String,
    optional: bool,
}

impl std::fmt::Debug for StaticTokenConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticTokenConfig")
            .field("token", &"<redacted>")
            .field("optional", &self.optional)
            .finish_non_exhaustive()
    }
}

/// Generic bearer gate with compile-time mode parameter.
#[derive(Clone)]
pub struct BearerGate<C, R, G, M>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq,
{
    issuer: String,
    codec: Arc<C>,
    mode: M,
    _phantom: std::marker::PhantomData<(R, G)>,
}

impl<C, R, G> BearerGate<C, R, G, JwtConfig<R, G>>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq + Clone,
{
    /// Internal constructor (used by `Gate::bearer`).
    pub(crate) fn new_with_codec(issuer: &str, codec: Arc<C>) -> Self {
        Self {
            issuer: issuer.to_string(),
            codec,
            mode: JwtConfig {
                policy: AccessPolicy::deny_all(),
                optional: false,
            },
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set access policy (OR semantics between requirements).
    pub fn with_policy(mut self, policy: AccessPolicy<R, G>) -> Self {
        self.mode.policy = policy;
        self
    }

    /// Turn on optional mode (install `Option<Account>`, `Option<RegisteredClaims>`).
    pub fn allow_anonymous_with_optional_user(mut self) -> Self {
        self.mode.optional = true;
        self
    }

    /// Configure the gate to allow any authenticated user: the baseline role (least
    /// privileged) from `Default::default()` and all supervisor roles as defined by
    /// your `AccessHierarchy`.
    ///
    /// Equivalent to `with_policy(AccessPolicy::require_role_or_supervisor(R::default()))`.
    /// Requires `R: Default`.
    pub fn require_login(mut self) -> Self
    where
        R: Default,
    {
        let baseline = R::default();
        self.mode.policy = AccessPolicy::require_role_or_supervisor(baseline);
        self
    }

    /// Enables Prometheus metrics for audit logging (JWT bearer mode).
    ///
    /// No-op unless both `audit-logging` and `prometheus` features are enabled.
    /// Safe to call multiple times; registration is idempotent.
    #[cfg(feature = "prometheus")]
    pub fn with_prometheus_metrics(self) -> Self {
        let _ = crate::audit::prometheus_metrics::install_prometheus_metrics();
        self
    }

    /// Installs Prometheus metrics into the provided registry (JWT bearer mode).
    ///
    /// No-op if metrics already installed. Returns `self` for builder chaining.
    #[cfg(feature = "prometheus")]
    pub fn with_prometheus_registry(self, registry: &prometheus::Registry) -> Self {
        let _ =
            crate::audit::prometheus_metrics::install_prometheus_metrics_with_registry(registry);
        self
    }

    /// Transition to static token mode: policies are discarded.
    pub fn with_static_token(
        self,
        token: impl Into<String>,
    ) -> BearerGate<C, R, G, StaticTokenConfig> {
        BearerGate {
            issuer: self.issuer,
            codec: self.codec,
            mode: StaticTokenConfig {
                token: token.into(),
                optional: false,
            },
            _phantom: std::marker::PhantomData,
        }
    }
}

// (require_login specialization for crate::auth::Role/Group temporarily removed to resolve generic issues)

impl<C, R, G> BearerGate<C, R, G, StaticTokenConfig>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq + Clone,
{
    /// Enable optional mode (install StaticTokenAuthorized(bool)).
    pub fn allow_anonymous_with_optional_user(mut self) -> Self {
        self.mode.optional = true;
        self
    }
}

// ===================== LAYER IMPLEMENTATIONS ======================

impl<S, C, R, G> Layer<S> for BearerGate<C, R, G, JwtConfig<R, G>>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display + Sync + Send + 'static,
    G: Eq + Clone + Sync + Send + 'static,
{
    type Service = JwtBearerService<C, R, G, S>;

    fn layer(&self, inner: S) -> Self::Service {
        if self.mode.optional {
            JwtBearerService::new_optional(
                inner,
                &self.issuer,
                self.mode.policy.clone(), // policy unused in optional mode but cloned for uniform struct
                Arc::clone(&self.codec),
            )
        } else {
            JwtBearerService::new(
                inner,
                &self.issuer,
                self.mode.policy.clone(),
                Arc::clone(&self.codec),
            )
        }
    }
}

impl<S, C, R, G> Layer<S> for BearerGate<C, R, G, StaticTokenConfig>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq + Clone,
{
    type Service = StaticTokenService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        if self.mode.optional {
            StaticTokenService::new_optional(inner, self.mode.token.clone())
        } else {
            StaticTokenService::new(inner, self.mode.token.clone())
        }
    }
}

// ===================== JWT SERVICE ======================

#[derive(Clone)]
/// JWT bearer token authentication service.
///
/// This service handles JWT bearer token authentication for protected routes,
/// validating tokens from the `Authorization: Bearer <token>` header.
pub struct JwtBearerService<C, R, G, S>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq + Clone,
{
    inner: S,
    authorization: AuthorizationService<R, G>,
    validator: JwtValidationService<C>,
    optional: bool,
}

impl<C, R, G, S> JwtBearerService<C, R, G, S>
where
    C: Codec,
    R: AccessHierarchy + Eq + std::fmt::Display,
    G: Eq + Clone,
{
    fn new(inner: S, issuer: &str, policy: AccessPolicy<R, G>, codec: Arc<C>) -> Self {
        Self {
            inner,
            authorization: AuthorizationService::new(policy),
            validator: JwtValidationService::new(codec, issuer),
            optional: false,
        }
    }

    fn new_optional(inner: S, issuer: &str, policy: AccessPolicy<R, G>, codec: Arc<C>) -> Self {
        // policy retained only for debugging; not used in optional path
        Self {
            inner,
            authorization: AuthorizationService::new(policy),
            validator: JwtValidationService::new(codec, issuer),
            optional: true,
        }
    }

    #[allow(clippy::expect_used)]
    fn unauthorized() -> Response<Body> {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(http::header::WWW_AUTHENTICATE, "Bearer")
            .body(Body::from("Unauthorized"))
            .expect("static unauthorized response")
    }

    fn bearer_token(req: &Request<Body>) -> Option<&str> {
        let value = req.headers().get(http::header::AUTHORIZATION)?;
        let value = value.to_str().ok()?.trim();
        let mut parts = value.split_whitespace();
        let scheme = parts.next()?;
        if !scheme.eq_ignore_ascii_case("Bearer") {
            return None;
        }
        parts.next()
    }
}

impl<C, R, G, S> Service<Request<Body>> for JwtBearerService<C, R, G, S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
    Account<R, G>: Clone,
    C: Codec<Payload = JwtClaims<Account<R, G>>>,
    R: AccessHierarchy + Eq + std::fmt::Display + Sync + Send + 'static,
    G: Eq + Clone + Sync + Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        #[cfg(feature = "audit-logging")]
        use crate::audit;

        let unauthorized_future = Box::pin(async move { Ok(Self::unauthorized()) });

        #[cfg(feature = "audit-logging")]
        let _span = audit::request_span(req.method().as_str(), req.uri().path(), None);

        if self.optional {
            let mut opt_account: Option<Account<R, G>> = None;
            let mut opt_claims: Option<RegisteredClaims> = None;

            if let Some(token) = Self::bearer_token(&req) {
                trace!("JWT optional bearer header present");
                if let JwtValidationResult::Valid(jwt) = self.validator.validate_token(token) {
                    // Valid JWT present; optional mode inserts only Option<...> extensions
                    opt_account = Some(jwt.custom_claims.clone());
                    opt_claims = Some(jwt.registered_claims.clone());
                } else {
                    debug!("Optional JWT: invalid token; inserting None extensions");
                }
            }

            req.extensions_mut().insert(opt_account);
            req.extensions_mut().insert(opt_claims);

            let fut = self.inner.call(req);
            return Box::pin(fut);
        }

        if self.authorization.policy_denies_all_access() {
            debug!("Bearer JWT gate denying access (deny-all policy)");
            #[cfg(feature = "audit-logging")]
            audit::denied(None, "policy_denies_all");
            return unauthorized_future;
        }

        let Some(token) = Self::bearer_token(&req) else {
            #[cfg(feature = "audit-logging")]
            audit::denied(None, "missing_authorization_header");
            return unauthorized_future;
        };

        #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
        let jwt_validation_start = std::time::Instant::now();

        let jwt = match self.validator.validate_token(token) {
            JwtValidationResult::Valid(jwt) => {
                #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
                crate::audit::prometheus_metrics::observe_jwt_validation_latency(
                    jwt_validation_start,
                    crate::audit::prometheus_metrics::JwtValidationOutcome::Valid,
                );
                jwt
            }
            JwtValidationResult::InvalidToken => {
                #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
                crate::audit::prometheus_metrics::observe_jwt_validation_latency(
                    jwt_validation_start,
                    crate::audit::prometheus_metrics::JwtValidationOutcome::InvalidToken,
                );
                debug!("JWT token validation failed");
                #[cfg(feature = "audit-logging")]
                audit::jwt_invalid_token("validation_failed");
                return unauthorized_future;
            }
            JwtValidationResult::InvalidIssuer { expected, actual } => {
                #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
                crate::audit::prometheus_metrics::observe_jwt_validation_latency(
                    jwt_validation_start,
                    crate::audit::prometheus_metrics::JwtValidationOutcome::InvalidIssuer,
                );
                warn!("JWT issuer mismatch. Expected='{expected}', Actual='{actual}'");
                #[cfg(feature = "audit-logging")]
                audit::jwt_invalid_issuer(&expected, &actual);
                return unauthorized_future;
            }
        };

        #[cfg(feature = "audit-logging")]
        let _authz_span = audit::authorization_span(Some(&jwt.custom_claims.account_id), None);
        #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
        let authz_start = std::time::Instant::now();

        if !self.authorization.is_authorized(&jwt.custom_claims) {
            #[cfg(feature = "audit-logging")]
            audit::denied(Some(&jwt.custom_claims.account_id), "policy_denied");
            #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
            crate::audit::observe_authz_latency(authz_start, crate::audit::AuthzOutcome::Denied);
            return unauthorized_future;
        }

        #[cfg(feature = "audit-logging")]
        audit::authorized(&jwt.custom_claims.account_id, None);
        #[cfg(all(feature = "audit-logging", feature = "prometheus"))]
        crate::audit::observe_authz_latency(authz_start, crate::audit::AuthzOutcome::Authorized);

        req.extensions_mut().insert(jwt.custom_claims.clone());
        req.extensions_mut().insert(jwt.registered_claims.clone());

        let fut = self.inner.call(req);
        Box::pin(fut)
    }
}

// ===================== STATIC TOKEN SERVICE ======================

#[derive(Clone)]
/// Static bearer token authentication service.
///
/// This service handles authentication using pre-configured static tokens
/// from the `Authorization: Bearer <token>` header.
pub struct StaticTokenService<S> {
    inner: S,
    token: String,
    optional: bool,
}

impl<S> StaticTokenService<S> {
    fn new(inner: S, token: String) -> Self {
        Self {
            inner,
            token,
            optional: false,
        }
    }

    fn new_optional(inner: S, token: String) -> Self {
        Self {
            inner,
            token,
            optional: true,
        }
    }

    #[allow(clippy::expect_used)]
    fn unauthorized() -> Response<Body> {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(http::header::WWW_AUTHENTICATE, "Bearer")
            .body(Body::from("Unauthorized"))
            .expect("static unauthorized response")
    }

    fn bearer_token(req: &Request<Body>) -> Option<&str> {
        let value = req.headers().get(http::header::AUTHORIZATION)?;
        let value = value.to_str().ok()?.trim();
        let mut parts = value.split_whitespace();
        let scheme = parts.next()?;
        if !scheme.eq_ignore_ascii_case("Bearer") {
            return None;
        }
        parts.next()
    }
}

impl<S> Service<Request<Body>> for StaticTokenService<S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        #[cfg(feature = "audit-logging")]
        use crate::audit;

        #[cfg(feature = "audit-logging")]
        let _span = audit::request_span(req.method().as_str(), req.uri().path(), None);

        if self.optional {
            let provided = Self::bearer_token(&req);
            let authorized = provided
                .map(|v| v.as_bytes().ct_eq(self.token.as_bytes()).into())
                .unwrap_or(false);
            req.extensions_mut()
                .insert(StaticTokenAuthorized::new(authorized));
            let fut = self.inner.call(req);
            return Box::pin(fut);
        }

        let Some(provided) = Self::bearer_token(&req) else {
            #[cfg(feature = "audit-logging")]
            audit::denied(None, "missing_authorization_header");
            return Box::pin(async move { Ok(Self::unauthorized()) });
        };

        if !bool::from(provided.as_bytes().ct_eq(self.token.as_bytes())) {
            #[cfg(feature = "audit-logging")]
            audit::denied(None, "static_token_mismatch");
            return Box::pin(async move { Ok(Self::unauthorized()) });
        }

        // Strict static token success: insert positive indicator
        req.extensions_mut()
            .insert(StaticTokenAuthorized::new(true));

        let fut = self.inner.call(req);
        Box::pin(fut)
    }
}

// ===================== TESTS ======================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::Account;
    use crate::codecs::jwt::{JsonWebToken, JwtClaims};
    use crate::groups::Group;
    use crate::roles::Role;

    type BearerGateJsonwebtoken = BearerGate<
        JsonWebToken<JwtClaims<Account<Role, Group>>>,
        Role,
        Group,
        JwtConfig<Role, Group>,
    >;

    #[test]
    fn jwt_gate_initial_deny_all() {
        let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
        let gate: BearerGateJsonwebtoken = BearerGate::new_with_codec("issuer", codec);
        assert!(gate.mode.policy.denies_all());
        assert!(!gate.mode.optional);
    }

    #[test]
    fn jwt_gate_policy_set() {
        let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
        let gate =
            BearerGate::new_with_codec("issuer", codec)
                .with_policy(AccessPolicy::<Role, Group>::require_role(Role::Admin));
        assert!(!gate.mode.policy.denies_all());
    }

    #[test]
    fn transition_to_static_mode() {
        let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
        let static_gate: BearerGate<_, Role, Group, StaticTokenConfig> =
            BearerGate::new_with_codec("issuer", codec).with_static_token("secret");
        assert_eq!(static_gate.mode.token, "secret");
        assert!(!static_gate.mode.optional);
    }

    #[test]
    fn static_optional_mode() {
        let codec = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
        let static_gate: BearerGate<_, Role, Group, StaticTokenConfig> =
            BearerGate::new_with_codec("issuer", codec)
                .with_static_token("secret")
                .allow_anonymous_with_optional_user();
        assert!(static_gate.mode.optional);
    }

    #[test]
    fn jwt_unauthorized_has_www_authenticate() {
        tokio_test::block_on(async {
            use axum::{body::Body, extract::Request, http::Response};
            use std::convert::Infallible;
            use tower::ServiceExt;

            let codec =
                std::sync::Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
            let gate: BearerGateJsonwebtoken = BearerGate::new_with_codec("issuer", codec)
                .with_policy(AccessPolicy::<Role, Group>::require_role(Role::Admin));

            let svc = gate.layer(tower::service_fn(|_req: Request<Body>| async {
                Ok::<_, Infallible>(Response::new(Body::from("ok")))
            }));

            let req = Request::new(Body::empty());
            let resp = svc.oneshot(req).await.unwrap();

            assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
            let hdr = resp
                .headers()
                .get(http::header::WWW_AUTHENTICATE)
                .and_then(|v| v.to_str().ok());
            assert_eq!(hdr, Some("Bearer"));
        });
    }

    #[test]
    fn static_token_unauthorized_has_www_authenticate() {
        tokio_test::block_on(async {
            use axum::{body::Body, extract::Request, http::Response};
            use std::convert::Infallible;
            use tower::ServiceExt;

            let codec =
                std::sync::Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::default());
            let gate: BearerGate<_, Role, Group, StaticTokenConfig> =
                BearerGate::new_with_codec("issuer", codec).with_static_token("secret");

            let svc = gate.layer(tower::service_fn(|_req: Request<Body>| async {
                Ok::<_, Infallible>(Response::new(Body::from("ok")))
            }));

            let req = Request::new(Body::empty());
            let resp = svc.oneshot(req).await.unwrap();

            assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
            let hdr = resp
                .headers()
                .get(http::header::WWW_AUTHENTICATE)
                .and_then(|v| v.to_str().ok());
            assert_eq!(hdr, Some("Bearer"));
        });
    }
}
