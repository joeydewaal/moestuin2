use axum::{Json, Router, routing::get};
use axum_security::{cookie::CookieSession, oidc::OidcExt};
use serde_json::json;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub mod auth;
pub mod config;
pub mod error;

use auth::User;
use config::Config;

pub async fn build_app(cfg: &Config) -> anyhow::Result<Router> {
    let sessions = auth::cookie_service(cfg);

    let mut app = Router::new()
        .route("/health", get(health))
        .route("/api/ping", get(ping))
        .route("/auth/me", get(auth::me));

    if cfg.mock_auth {
        tracing::warn!("MOESTUIN_MOCK_AUTH enabled — /auth/dev-login is live, DO NOT use in prod");
        app = app.merge(auth::dev_routes(cfg, sessions.clone()));
    } else {
        let oidc = auth::build_oidc(cfg, sessions.clone()).await?;
        app = app.with_oidc(oidc);
    }

    Ok(app
        .layer(sessions)
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(TraceLayer::new_for_http()))
}

async fn health() -> &'static str {
    "ok"
}

async fn ping(session: CookieSession<User>) -> Json<serde_json::Value> {
    Json(json!({ "pong": true, "email": session.state.email }))
}
