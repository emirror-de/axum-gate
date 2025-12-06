use axum::{
    Router,
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
};

use axum_gate::accounts::{Account, AccountRepository};
use axum_gate::codecs::jwt::{JsonWebToken, JwtClaims};
use axum_gate::cookie;
use axum_gate::prelude::{AccessPolicy, Gate, Group, Role};
use axum_gate::repositories::memory::MemoryAccountRepository;
use dotenvy::dotenv;
use oauth2::TokenResponse;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(serde::Deserialize)]
struct GithubUser {
    login: String,
}

/// Logging wrapper around an AccountRepository that emits info! logs on query/insert.
/// This lets the example show whether the account was inserted (first login) or queried.
struct LoggingAccountRepository<R, G, Inner> {
    inner: Arc<Inner>,
    _phantom: std::marker::PhantomData<(R, G)>,
}

impl<R, G, Inner> LoggingAccountRepository<R, G, Inner> {
    fn new(inner: Arc<Inner>) -> Self {
        Self {
            inner,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, G, Inner> AccountRepository<R, G> for LoggingAccountRepository<R, G, Inner>
where
    R: axum_gate::authz::AccessHierarchy + Eq + std::fmt::Display + Send + Sync + 'static,
    G: Eq + Clone + Send + Sync + 'static,
    Inner: AccountRepository<R, G> + Send + Sync + 'static,
{
    async fn store_account(
        &self,
        account: Account<R, G>,
    ) -> axum_gate::errors::Result<Option<Account<R, G>>> {
        let res = self.inner.store_account(account).await?;
        if let Some(ref acc) = res {
            info!(user_id = %acc.user_id, account_id = %acc.account_id, "OAuth2: new account inserted");
        }
        Ok(res)
    }

    async fn delete_account(
        &self,
        user_id: &str,
    ) -> axum_gate::errors::Result<Option<Account<R, G>>> {
        self.inner.delete_account(user_id).await
    }

    async fn update_account(
        &self,
        account: Account<R, G>,
    ) -> axum_gate::errors::Result<Option<Account<R, G>>> {
        self.inner.update_account(account).await
    }

    async fn query_account_by_user_id(
        &self,
        user_id: &str,
    ) -> axum_gate::errors::Result<Option<Account<R, G>>> {
        let res = self.inner.query_account_by_user_id(user_id).await?;
        if let Some(ref acc) = res {
            info!(user_id = %acc.user_id, account_id = %acc.account_id, "OAuth2: existing account queried");
        }
        Ok(res)
    }

    async fn query_account_by_id(
        &self,
        account_id: &str,
    ) -> axum_gate::errors::Result<Option<Account<R, G>>> {
        let res = self.inner.query_account_by_id(account_id).await?;
        if let Some(ref acc) = res {
            info!(user_id = %acc.user_id, account_id = %acc.account_id, "OAuth2: existing account queried");
        }
        Ok(res)
    }

    async fn query_all_accounts(&self) -> axum_gate::errors::Result<Vec<Account<R, G>>> {
        let res = self.inner.query_all_accounts().await?;
        info!(count = res.len(), "OAuth2: queried all accounts");
        Ok(res)
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    // Load .env if present
    let _ = dotenv();

    // Server configuration
    let addr: SocketAddr = env::var("APP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()
        .expect("APP_ADDR must be host:port");

    // JWT config (first-party cookie)
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "local-dev-secret-change-me".into());
    let jwt_issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "my-app".into());
    let auth_cookie_name = env::var("AUTH_COOKIE_NAME").unwrap_or_else(|_| "auth-token".into());
    let post_login_redirect = env::var("POST_LOGIN_REDIRECT").unwrap_or_else(|_| "/".into());
    let jwt_ttl_secs: u64 = env::var("JWT_TTL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60 * 60 * 24); // 24h

    // Build a JWT codec with a persistent symmetric key (from env for demo)
    let jwt_codec = Arc::new(
        JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(
            axum_gate::codecs::jwt::JsonWebTokenOptions {
                enc_key: axum_gate::jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
                dec_key: axum_gate::jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
                header: None,
                validation: None,
            },
        ),
    );

    let account_repo = Arc::new(MemoryAccountRepository::<Role, Group>::default());
    let logging_repo = Arc::new(LoggingAccountRepository::<Role, Group, _>::new(Arc::clone(
        &account_repo,
    )));

    // GitHub OAuth2 configuration via env
    let github_client_id =
        env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set (GitHub OAuth app)");
    let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
        .expect("GITHUB_CLIENT_SECRET must be set (GitHub OAuth app)");
    let github_redirect = env::var("GITHUB_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:3000/auth/callback".into());

    // Construct OAuth2 gate for GitHub
    // The account mapper below fetches info from GitHub's user APIs:
    //  - https://api.github.com/user
    //  - https://api.github.com/user/emails (optional)
    // and uses the GitHub username as the Account user_id.
    let oauth2_gate = axum_gate::gate::oauth2::OAuth2Gate::<Role, Group>::new()
        .auth_url("https://github.com/login/oauth/authorize")
        .token_url("https://github.com/login/oauth/access_token")
        .client_id(github_client_id)
        .client_secret(github_client_secret)
        .redirect_url(github_redirect)
        .add_scope("read:user")
        .add_scope("user:email")
        .configure_cookie_template(|tpl| tpl.name(auth_cookie_name.clone()))
        .expect("valid oauth2 cookie template")
        .with_post_login_redirect(post_login_redirect.clone())
        .with_jwt_codec(&jwt_issuer, Arc::clone(&jwt_codec), jwt_ttl_secs)
        .with_account_repository(Arc::clone(&logging_repo))
        .with_account_mapper(|token_resp| {
            Box::pin(async move {
                // Fetch the actual GitHub username using the access token.
                let access_token = token_resp.access_token().secret().to_string();

                let client = reqwest::Client::new();
                let login = match client
                    .get("https://api.github.com/user")
                    .header("Authorization", format!("Bearer {}", access_token))
                    .header("Accept", "application/vnd.github+json")
                    .header("User-Agent", "axum-gate-oauth2-github-example")
                    .send()
                    .await
                {
                    Ok(r) => match r.json::<GithubUser>().await {
                        Ok(user) => user.login,
                        Err(_) => "github-user".to_string(),
                    },
                    Err(_) => "github-user".to_string(),
                };

                Ok(Account::<Role, Group>::new(&login, &[Role::User], &[]))
            })
        });

    // Mount OAuth2 routes at /auth/login and /auth/callback
    let auth_router = oauth2_gate
        .routes("/auth")
        .expect("valid GitHub OAuth2 configuration");

    // A protected route that requires any authenticated user (baseline + supervisors)
    let protected = Router::new()
        .route("/protected", get(protected_handler))
        .layer(
            Gate::cookie::<_, Role, Group>(&jwt_issuer, Arc::clone(&jwt_codec))
                .require_login()
                .configure_cookie_template(|tpl| {
                    tpl.name(auth_cookie_name.clone())
                        .persistent(cookie::time::Duration::hours(24))
                })
                .expect("valid cookie template"),
        );

    // Public routes
    let public = Router::new()
        .route(
            "/",
            get(homepage).layer(
                Gate::cookie::<_, Role, Group>(&jwt_issuer, Arc::clone(&jwt_codec))
                    .allow_anonymous_with_optional_user()
                    .configure_cookie_template(|tpl| {
                        tpl.name(auth_cookie_name.clone())
                            .persistent(cookie::time::Duration::hours(24))
                    })
                    .expect("valid cookie template"),
            ),
        )
        .route(
            "/logout",
            get({
                let name = auth_cookie_name.clone();
                move |cookie_jar| async move {
                    let cookie_template =
                        axum_gate::cookie_template::CookieTemplate::recommended().name(name);
                    let jar = axum_gate::route_handlers::logout(cookie_jar, cookie_template).await;
                    (jar, axum::response::Redirect::to("/"))
                }
            }),
        );

    // Compose the app
    let app = Router::new()
        .merge(public)
        .merge(auth_router)
        .merge(protected);

    info!("Starting server on http://{}", addr);
    info!("Homepage: http://{}/", addr);
    info!("Login with GitHub: http://{}/auth/login", addr);
    info!("Protected route: http://{}/protected", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn homepage(Extension(opt_user): Extension<Option<Account<Role, Group>>>) -> Html<String> {
    let status_html = match opt_user {
        Some(account) => format!(
            r#"<p class="note">Status: Logged in as <code>{}</code> — <a href="/logout">Log out</a></p>"#,
            account.user_id
        ),
        None => r#"<p class="note">Status: Not logged in</p>"#.to_string(),
    };

    let prefix = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>axum-gate: GitHub OAuth2 Example</title>
  <style>
    body { font-family: system-ui, Arial, sans-serif; margin: 2rem; }
    .btn { display: inline-block; padding: 0.6rem 1rem; background: #24292f; color: #fff; border-radius: 6px; text-decoration: none; }
    .btn:hover { background: #000; }
    .note { color: #444; margin-top: 1rem; }
    code { background: #f5f5f5; padding: 0.2rem 0.4rem; border-radius: 4px; }
  </style>
</head>
<body>
  <h1>axum-gate: GitHub OAuth2 Example</h1>
  <p>This example demonstrates using GitHub as an OAuth2 provider to mint a first‑party JWT cookie.</p>
  <p><a class="btn" href="/auth/login">Login with GitHub</a></p>
"#;

    let suffix = r#"
  <h2>Protected route</h2>
  <p>After login, visit <a href="/protected"><code>/protected</code></a> to see authenticated content.</p>

  <div class="note">
    <p>Required environment variables: <code>GITHUB_CLIENT_ID</code>, <code>GITHUB_CLIENT_SECRET</code>.</p>
    <p>Optional: <code>GITHUB_REDIRECT_URL</code> (default <code>http://localhost:3000/auth/callback</code>), <code>JWT_SECRET</code>, <code>JWT_ISSUER</code>, <code>AUTH_COOKIE_NAME</code>, <code>POST_LOGIN_REDIRECT</code>.</p>
  </div>
</body>
</html>
"#;

    let mut html = String::with_capacity(prefix.len() + status_html.len() + suffix.len());
    html.push_str(prefix);
    html.push_str(&status_html);
    html.push_str(suffix);
    Html(html)
}

async fn protected_handler(
    Extension(account): Extension<Account<Role, Group>>,
) -> impl IntoResponse {
    let roles = account
        .roles
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let groups = account
        .groups
        .iter()
        .map(|g| format!("{:?}", g))
        .collect::<Vec<_>>()
        .join(", ");

    let body = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Protected</title>
  <style> body {{ font-family: system-ui, Arial, sans-serif; margin: 2rem; }} code {{ background: #f5f5f5; padding: .2rem .4rem; border-radius: 4px; }}</style>
</head>
<body>
  <h1>Protected</h1>
  <p>You are authenticated as <code>{}</code></p>
  <p>Account ID: <code>{}</code></p>
  <p>Roles: <code>{}</code></p>
  <p>Groups: <code>{}</code></p>
  <p><a href="/">← Home</a></p>
</body>
</html>"#,
        account.user_id, account.account_id, roles, groups
    );

    Html(body)
}

fn init_tracing() {
    let env_filter = env::var("RUST_LOG").unwrap_or_else(|_| "info,axum=info,hyper=info".into());
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(env_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Helper: demonstrate composing a policy (not used directly in this example)
#[allow(dead_code)]
fn _policy_example() -> AccessPolicy<Role, Group> {
    AccessPolicy::<Role, Group>::require_role(Role::User)
        .or_require_role(Role::Moderator)
        .or_require_role(Role::Admin)
}
