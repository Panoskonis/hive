pub mod auth;
pub mod error;
pub mod queries;
pub mod state;

use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/auth", auth::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { pool })
}
