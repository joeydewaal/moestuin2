use std::time::Duration;

use axum::{Json, Router, routing::get};
use axum_security::{cookie::CookieSession, oidc::OidcExt};
use serde_json::json;
use toasty::Db;
use tokio::sync::broadcast;
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};

pub mod auth;
pub mod config;
pub mod error;
pub mod readings;
pub mod sensors;
pub mod session_store;
pub mod webcam;

use auth::User;
use config::Config;
use error::{AppError, AppResult};
use readings::{Reading, ReadingsState};
use webcam::WebcamState;

pub const POLL_INTERVAL: Duration = Duration::from_secs(30);

pub async fn open_db(cfg: &Config) -> AppResult<Db> {
    let db = Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&cfg.database_url)
        .await
        .map_err(|e| AppError::internal(format!("open db: {e}")))?;
    db.push_schema()
        .await
        .map_err(|e| AppError::internal(format!("push schema: {e}")))?;
    Ok(db)
}

pub async fn build_app(cfg: &Config, db: Db) -> AppResult<Router> {
    let sessions = auth::cookie_service(cfg, db.clone());

    let driver = sensors::probe(cfg.mock_hardware).await;
    let (tx, _rx) = broadcast::channel::<Reading>(64);
    sensors::spawn_poller(db.clone(), driver, tx.clone(), POLL_INTERVAL);

    let readings_state = ReadingsState { db: db.clone(), tx };
    let webcam_state = WebcamState {
        root: cfg.webcam_root.clone(),
    };

    let mut app = Router::new()
        .route("/api/ping", get(ping))
        .route("/auth/me", get(auth::me))
        .merge(readings::routes(readings_state))
        .merge(webcam::routes(webcam_state))
        .nest_service("/webcam", ServeDir::new(&cfg.webcam_root));

    if cfg.mock_auth {
        tracing::warn!("MOESTUIN_MOCK_AUTH enabled — /auth/dev-login is live, DO NOT use in prod");
        app = app.merge(auth::dev_routes(cfg, db.clone(), sessions.clone()));
    } else {
        let oidc = auth::build_oidc(cfg, db, sessions.clone()).await?;
        app = app.with_oidc(oidc);
    }

    Ok(app
        .layer(sessions)
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(TraceLayer::new_for_http()))
}

async fn ping(session: CookieSession<User>) -> Json<serde_json::Value> {
    Json(json!({ "pong": true, "email": session.state.email }))
}
