use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, TimeDelta, Utc};
use rand::{RngCore, rngs::OsRng as TokenRng};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgDatabaseError};
use uuid::Uuid;

use crate::{error::ApiError, queries::users, state::AppState};

const SESSION_TTL_DAYS: i64 = 30;
const TOKEN_BYTES: usize = 32;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: SecretString,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: i32,
    username: String,
    email: String,
    password: String,
    role: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub session_id: Uuid,
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    let password_hash = hash_password(&payload.password)?;

    let user = sqlx::query_as::<_, UserRow>(users::INSERT_USER)
        .bind(payload.username.trim())
        .bind(payload.email.trim().to_lowercase())
        .bind(password_hash)
        .fetch_one(&state.pool)
        .await
        .map_err(map_insert_user_error)?;

    let auth = create_session(&state.pool, user.into_response()).await?;

    Ok((StatusCode::CREATED, Json(auth)))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = sqlx::query_as::<_, UserRow>(users::FIND_USER_BY_EMAIL)
        .bind(payload.email.trim().to_lowercase())
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    verify_password(&payload.password, &user.password)?;

    let user = user.into_response();
    sqlx::query(users::UPDATE_USER_LAST_LOGIN)
        .bind(user.id)
        .execute(&state.pool)
        .await?;

    Ok(Json(create_session(&state.pool, user).await?))
}

async fn logout(State(state): State<AppState>, user: AuthUser) -> Result<StatusCode, ApiError> {
    sqlx::query(users::REVOKE_SESSION)
        .bind(user.session_id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn me(user: AuthUser) -> Json<UserResponse> {
    Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    })
}

async fn create_session(pool: &PgPool, user: UserResponse) -> Result<AuthResponse, ApiError> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + TimeDelta::days(SESSION_TTL_DAYS);

    sqlx::query(users::INSERT_SESSION)
        .bind(user.id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(pool)
        .await?;

    Ok(AuthResponse {
        token,
        expires_at,
        user,
    })
}

fn hash_password(password: &SecretString) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();

    Ok(hash)
}

fn verify_password(password: &SecretString, password_hash: &str) -> Result<(), ApiError> {
    let parsed_hash = PasswordHash::new(password_hash)?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::InvalidCredentials)
}

fn generate_token() -> String {
    let mut bytes = [0_u8; TOKEN_BYTES];
    TokenRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn bearer_token(parts: &Parts) -> Result<&str, ApiError> {
    let header = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    header
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .ok_or(ApiError::Unauthorized)
}

fn map_insert_user_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error
            .try_downcast_ref::<PgDatabaseError>()
            .is_some_and(|postgres_error| postgres_error.code() == "23505")
        {
            return ApiError::UserAlreadyExists;
        }
    }

    ApiError::Database(error)
}

impl UserRow {
    fn into_response(self) -> UserResponse {
        UserResponse {
            id: self.id,
            username: self.username,
            email: self.email,
            role: self.role,
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token_hash = hash_token(bearer_token(parts)?);

        sqlx::query_as::<_, AuthUser>(users::AUTHENTICATE_SESSION)
            .bind(token_hash)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(ApiError::Unauthorized)
    }
}
