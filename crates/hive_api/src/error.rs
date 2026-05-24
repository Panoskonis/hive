use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("authentication required")]
    Unauthorized,
    #[error("username or email already exists")]
    UserAlreadyExists,
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("password hashing error")]
    PasswordHash,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid credentials"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "authentication required"),
            Self::UserAlreadyExists => (StatusCode::CONFLICT, "username or email already exists"),
            Self::Database(_) | Self::PasswordHash => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }
        };

        (status, Json(ErrorResponse { error })).into_response()
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::PasswordHash
    }
}
