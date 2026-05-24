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
    #[error("access forbidden")]
    Forbidden,
    #[error("username or email already exists")]
    UserAlreadyExists,
    #[error("game not found")]
    GameNotFound,
    #[error("invite not found")]
    InviteNotFound,
    #[error("game already started")]
    GameAlreadyStarted,
    #[error("game not started")]
    GameNotStarted,
    #[error("game already finished")]
    GameAlreadyFinished,
    #[error("wrong turn")]
    WrongTurn,
    #[error("invalid action")]
    InvalidAction,
    #[error("rule violation: {0}")]
    RuleViolation(String),
    #[error("cannot join your own game")]
    CannotJoinOwnGame,
    #[error("invalid game request")]
    InvalidGameRequest,
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("password hashing error")]
    PasswordHash,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid credentials"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "authentication required"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "access forbidden"),
            Self::UserAlreadyExists => (StatusCode::CONFLICT, "username or email already exists"),
            Self::GameNotFound => (StatusCode::NOT_FOUND, "game not found"),
            Self::InviteNotFound => (StatusCode::NOT_FOUND, "invite not found"),
            Self::GameAlreadyStarted => (StatusCode::CONFLICT, "game already started"),
            Self::GameNotStarted => (StatusCode::CONFLICT, "game not started"),
            Self::GameAlreadyFinished => (StatusCode::CONFLICT, "game already finished"),
            Self::WrongTurn => (StatusCode::CONFLICT, "wrong turn"),
            Self::InvalidAction => (StatusCode::BAD_REQUEST, "invalid action"),
            Self::CannotJoinOwnGame => (StatusCode::CONFLICT, "cannot join your own game"),
            Self::InvalidGameRequest => (StatusCode::BAD_REQUEST, "invalid game request"),
            Self::RuleViolation(message) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(ErrorResponse { error: message }),
                )
                    .into_response();
            }
            Self::Database(_) | Self::PasswordHash => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }
        };

        (
            status,
            Json(ErrorResponse {
                error: error.to_owned(),
            }),
        )
            .into_response()
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::PasswordHash
    }
}
