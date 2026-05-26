use std::{env, net::SocketAddr};

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hive_api=debug,tower_http=debug".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/postgres".to_owned());
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let addr: SocketAddr = listener.local_addr()?;

    tracing::info!(%addr, "listening");
    axum::serve(listener, hive_api::router(pool)).await?;

    Ok(())
}
