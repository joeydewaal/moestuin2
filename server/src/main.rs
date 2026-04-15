use std::net::SocketAddr;

use server::{build_app, config::Config};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cfg = Config::from_env()?;
    let app = build_app(&cfg).await?;

    let addr: SocketAddr = cfg.bind.parse()?;
    tracing::info!(%addr, "moestuin listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
