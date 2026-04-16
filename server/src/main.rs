use std::{error::Error, net::SocketAddr};

use server::{build_app, config::Config, open_db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let cfg = Config::from_env()?;
    let db = open_db(&cfg).await?;
    let app = build_app(&cfg, db).await?;

    let addr: SocketAddr = cfg.bind.parse()?;
    tracing::info!(%addr, "moestuin listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
