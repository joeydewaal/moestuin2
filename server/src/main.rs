use std::net::SocketAddr;

use axum::{Router, routing::get};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cfg = config::Config::from_env()?;

    let app = Router::new()
        .route("/health", get(health))
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = cfg.bind.parse()?;
    tracing::info!(%addr, "moestuin listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
