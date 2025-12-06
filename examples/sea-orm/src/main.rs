use axum_gate::accounts::AccountInsertService;
use axum_gate::codecs::jwt::RegisteredClaims;
use axum_gate::prelude::*;
use axum_gate::repositories::sea_orm::SeaOrmRepository;

use std::sync::Arc;

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{Router, get, post};
use axum_gate::accounts::AccountRepository;
use axum_gate::hashing::argon2::Argon2Hasher;
use axum_gate::secrets::{Secret, SecretRepository};
use chrono::{TimeDelta, Utc};
use sea_orm::{ConnectionTrait, DbBackend, DbConn, Schema};
use sea_query::table::TableCreateStatement;
use tracing::debug;

const DATABASE_URL: &str = "sqlite::memory:";
// Use the following if you want to see what is stored
//const DATABASE_URL: &str = "sqlite:auth-node.sqlite3?mode=rwc";

async fn setup_database_schema(db: &DbConn) {
    let schema = Schema::new(DbBackend::Sqlite);
    let stmt: TableCreateStatement = schema
        .create_table_from_entity(axum_gate::repositories::sea_orm::models::credentials::Entity);
    db.execute(&stmt)
        .await
        .expect("Could not create credentials table");
    let stmt: TableCreateStatement =
        schema.create_table_from_entity(axum_gate::repositories::sea_orm::models::account::Entity);
    db.execute(&stmt)
        .await
        .expect("Could not create account table");
}

#[derive(serde::Deserialize)]
struct PasswordUpdate {
    user_id: String,
    new_password: String,
}

#[derive(serde::Deserialize)]
struct AccountUpdate {
    user_id: String,
    roles: Vec<Role>,
    groups: Vec<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().expect("Could not read .env file.");
    let shared_secret =
        dotenvy::var("AXUM_GATE_SHARED_SECRET").expect("AXUM_GATE_SHARED_SECRET env var not set.");
    let jwt_options = JsonWebTokenOptions {
        enc_key: axum_gate::jsonwebtoken::EncodingKey::from_secret(shared_secret.as_bytes()),
        dec_key: axum_gate::jsonwebtoken::DecodingKey::from_secret(shared_secret.as_bytes()),
        header: Some(axum_gate::jsonwebtoken::Header::default()),
        validation: Some(axum_gate::jsonwebtoken::Validation::default()),
    };
    let jwt_codec =
        Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(jwt_options));

    // SQLite memory database connection
    let db = sea_orm::Database::connect(DATABASE_URL)
        .await
        .unwrap_or_else(|_| panic!("Could not connect to {DATABASE_URL} database."));

    setup_database_schema(&db).await;

    let account_repository = Arc::new(SeaOrmRepository::new(&db).unwrap());
    debug!("Account repository initialized.");
    let secrets_repository = Arc::clone(&account_repository);
    debug!("Secrets repository initialized.");

    AccountInsertService::insert("admin@example.com", "admin_password")
        .with_roles(vec![Role::Admin])
        .with_groups(vec![Group::new("admin")])
        .into_repositories(
            Arc::clone(&account_repository),
            Arc::clone(&secrets_repository),
        )
        .await
        .unwrap();
    debug!("Inserted Admin.");

    AccountInsertService::insert("reporter@example.com", "reporter_password")
        .with_roles(vec![Role::Reporter])
        .with_groups(vec![Group::new("reporter")])
        .into_repositories(
            Arc::clone(&account_repository),
            Arc::clone(&secrets_repository),
        )
        .await
        .unwrap();
    debug!("Inserted Reporter.");

    AccountInsertService::insert("user@example.com", "user_password")
        .with_roles(vec![Role::User])
        .with_groups(vec![Group::new("user")])
        .into_repositories(
            Arc::clone(&account_repository),
            Arc::clone(&secrets_repository),
        )
        .await
        .unwrap();
    debug!("Inserted User.");

    let cookie_template = axum_gate::cookie_template::CookieTemplate::recommended();

    let app = Router::new()
        .route(
            "/login",
            post({
                let registered_claims = RegisteredClaims::new(
                    // same as in distributed example, so you can re-use the consumer_node
                    "auth-node",
                    (Utc::now() + TimeDelta::weeks(1)).timestamp() as u64,
                );
                let secrets_repository = Arc::clone(&secrets_repository);
                let account_repository = Arc::clone(&account_repository);
                let jwt_codec = Arc::clone(&jwt_codec);
                let cookie_template = cookie_template.clone();
                move |cookie_jar, Json(credentials): Json<Credentials<String>>| {
                    axum_gate::route_handlers::login(
                        cookie_jar,
                        credentials,
                        registered_claims,
                        secrets_repository,
                        account_repository,
                        jwt_codec,
                        cookie_template,
                    )
                }
            }),
        )
        .route(
            "/logout",
            get(move |cookie_jar| axum_gate::route_handlers::logout(cookie_jar, cookie_template)),
        )
        .route(
            "/password",
            post({
                let account_repository = Arc::clone(&account_repository);
                let secrets_repository = Arc::clone(&secrets_repository);
                move |Json(body): Json<PasswordUpdate>| async move {
                    let Some(account): std::option::Option<
                        axum_gate::accounts::Account<Role, Group>,
                    > = account_repository
                        .query_account_by_user_id(&body.user_id)
                        .await
                        .unwrap()
                    else {
                        return (StatusCode::NOT_FOUND, "account not found").into_response();
                    };
                    let hasher = Argon2Hasher::new_recommended().unwrap();
                    let secret =
                        Secret::new(&account.account_id, &body.new_password, hasher).unwrap();
                    secrets_repository.update_secret(secret).await.unwrap();
                    (StatusCode::OK, "password updated").into_response()
                }
            }),
        )
        .route(
            "/account",
            post({
                let account_repository = Arc::clone(&account_repository);
                move |Json(body): Json<AccountUpdate>| async move {
                    let Some(mut account) = account_repository
                        .query_account_by_user_id(&body.user_id)
                        .await
                        .unwrap()
                    else {
                        return (StatusCode::NOT_FOUND, "account not found").into_response();
                    };
                    account.roles = body.roles;
                    account.groups = body.groups.into_iter().map(|g| Group::new(&g)).collect();
                    let _ = account_repository.update_account(account).await.unwrap();
                    (StatusCode::OK, "account updated").into_response()
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
