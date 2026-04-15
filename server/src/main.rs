use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use axum_security::{cookie::CookieSession, oidc::OidcExt};
use serde_json::json;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, fmt};

mod auth;
mod config;
mod error;

use auth::User;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cfg = config::Config::from_env()?;

    let sessions = auth::cookie_service(&cfg);

    let mut app = Router::new()
        .route("/health", get(health))
        .route("/api/ping", get(ping))
        .route("/auth/me", get(auth::me));

    if cfg.mock_auth {
        tracing::warn!("MOESTUIN_MOCK_AUTH enabled — /auth/dev-login is live, DO NOT use in prod");
        app = app.merge(auth::dev_routes(&cfg, sessions.clone()));
    } else {
        let oidc = auth::build_oidc(&cfg, sessions.clone()).await?;
        app = app.with_oidc(oidc);
    }

    let app = app
        .layer(sessions)
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

async fn ping(session: CookieSession<User>) -> Json<serde_json::Value> {
    Json(json!({ "pong": true, "email": session.state.email }))
}
