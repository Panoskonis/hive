//! Auth endpoint integration tests against a real Postgres instance.
//!
//! Requires Postgres (e.g. `docker compose up -d`) and:
//! `DATABASE_URL=postgres://postgres:postgres@localhost:5433/postgres`

use axum::body::Body;
use axum::http::{Request, StatusCode};
use hive_api::router;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test(migrations = "../../migrations")]
async fn health_returns_ok(pool: PgPool) -> sqlx::Result<()> {
    let app = router(pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn register_returns_201_and_token(pool: PgPool) -> sqlx::Result<()> {
    let (status, body) = post_json(
        pool,
        "/auth/register",
        json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "secret-password"
        }),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["username"], "alice");
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert_eq!(body["user"]["role"], "user");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn register_duplicate_email_returns_409(pool: PgPool) -> sqlx::Result<()> {
    let payload = json!({
        "username": "bob",
        "email": "bob@example.com",
        "password": "secret-password"
    });

    let (first, _) = post_json(pool.clone(), "/auth/register", payload.clone(), None).await;
    assert_eq!(first, StatusCode::CREATED);

    let (second, body) = post_json(pool, "/auth/register", payload, None).await;
    assert_eq!(second, StatusCode::CONFLICT);
    assert_eq!(body["error"], "username or email already exists");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_returns_token_for_registered_user(pool: PgPool) -> sqlx::Result<()> {
    post_json(
        pool.clone(),
        "/auth/register",
        json!({
            "username": "carol",
            "email": "carol@example.com",
            "password": "secret-password"
        }),
        None,
    )
    .await;

    let (status, body) = post_json(
        pool,
        "/auth/login",
        json!({
            "email": "carol@example.com",
            "password": "secret-password"
        }),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "carol@example.com");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn me_requires_bearer_token(pool: PgPool) -> sqlx::Result<()> {
    let (status, body) = get_json(pool, "/auth/me", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "authentication required");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn me_returns_user_with_valid_token(pool: PgPool) -> sqlx::Result<()> {
    let (_, register) = post_json(
        pool.clone(),
        "/auth/register",
        json!({
            "username": "dave",
            "email": "dave@example.com",
            "password": "secret-password"
        }),
        None,
    )
    .await;

    let token = register["token"].as_str().unwrap();
    let (status, body) = get_json(pool, "/auth/me", Some(token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "dave");
    assert_eq!(body["email"], "dave@example.com");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn logout_revokes_session(pool: PgPool) -> sqlx::Result<()> {
    let (_, register) = post_json(
        pool.clone(),
        "/auth/register",
        json!({
            "username": "eve",
            "email": "eve@example.com",
            "password": "secret-password"
        }),
        None,
    )
    .await;

    let token = register["token"].as_str().unwrap();

    let (logout_status, _) = post_json(pool.clone(), "/auth/logout", json!({}), Some(token)).await;
    assert_eq!(logout_status, StatusCode::NO_CONTENT);

    let (me_status, me_body) = get_json(pool, "/auth/me", Some(token)).await;
    assert_eq!(me_status, StatusCode::UNAUTHORIZED);
    assert_eq!(me_body["error"], "authentication required");
    Ok(())
}

async fn post_json(
    pool: PgPool,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    request_json(pool, "POST", uri, Some(body), bearer).await
}

async fn get_json(pool: PgPool, uri: &str, bearer: Option<&str>) -> (StatusCode, Value) {
    request_json(pool, "GET", uri, None, bearer).await
}

async fn request_json(
    pool: PgPool,
    method: &str,
    uri: &str,
    body: Option<Value>,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let app = router(pool);

    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }

    let request_body = match body {
        Some(value) => Body::from(value.to_string()),
        None => Body::empty(),
    };

    let response = app
        .oneshot(builder.body(request_body).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();

    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!(
                "response is not JSON (status {status}): {error}; body={}",
                String::from_utf8_lossy(&bytes)
            );
        })
    };

    (status, json)
}
