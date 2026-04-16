use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

use axum_security::{
    cookie::{CookieContext, CookieSession, SameSite},
    oidc::{AfterLoginCookies, LogoutContext, OidcContext, OidcHandler, OidcTokenResponse},
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::Db;
use uuid::Uuid;

use crate::{config::Config, session_store::ToastySessionStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub subject: String,
    pub email: String,
    pub name: Option<String>,
    pub created_at: Timestamp,
}

pub type Sessions = CookieContext<User>;

pub fn cookie_service(cfg: &Config, db: Db) -> Sessions {
    CookieContext::builder()
        .cookie(|c| {
            c.name("moestuin_session")
                .path("/")
                .max_age(Duration::from_secs(60 * 60 * 24 * 7))
                .secure()
                .http_only()
                .same_site(SameSite::Lax)
        })
        .dev_cookie(|c| {
            c.name("moestuin_session_dev")
                .path("/")
                .max_age(Duration::from_secs(60 * 60 * 24 * 7))
                .http_only()
                .same_site(SameSite::Lax)
        })
        .use_dev_cookie(cfg.mock_auth || cfg!(debug_assertions))
        .store(ToastySessionStore::new(db))
        .expires_max_age()
        .build::<User>()
}

pub struct AllowlistHandler {
    sessions: Sessions,
    allowed: Vec<String>,
}

impl OidcHandler for AllowlistHandler {
    async fn after_login(
        &self,
        token_res: OidcTokenResponse<'_>,
        cookies: &mut AfterLoginCookies<'_>,
    ) -> impl IntoResponse {
        let email = match token_res.claims.email.as_ref() {
            Some(e) => e.to_string(),
            None => return Redirect::to("/login?error=no_email").into_response(),
        };

        if token_res.claims.email_verified != Some(true) {
            tracing::warn!(%email, "rejecting unverified email");
            return Redirect::to("/login?error=unverified").into_response();
        }

        if !self.allowed.iter().any(|a| a.eq_ignore_ascii_case(&email)) {
            tracing::warn!(%email, "rejecting email not on allowlist");
            return Redirect::to("/login?error=forbidden").into_response();
        }

        let user = User {
            id: Uuid::now_v7(),
            subject: token_res.claims.subject.to_string(),
            email,
            name: token_res.claims.name.map(ToString::to_string),
            created_at: Timestamp::now(),
        };

        match self.sessions.create_session(user).await {
            Ok(cookie) => {
                cookies.add(cookie);
                Redirect::to("/").into_response()
            }
            Err(e) => {
                tracing::error!(?e, "failed to create session");
                (StatusCode::INTERNAL_SERVER_ERROR, "session error").into_response()
            }
        }
    }

    async fn logout(&self, context: LogoutContext) -> impl IntoResponse {
        context.default_redirect()
    }
}

pub async fn build_oidc(
    cfg: &Config,
    sessions: Sessions,
) -> crate::error::AppResult<OidcContext<AllowlistHandler>> {
    let handler = AllowlistHandler {
        sessions,
        allowed: cfg.allowed_emails.clone(),
    };

    Ok(OidcContext::google()
        .await
        .map_err(|e| crate::error::AppError::internal(format!("google discovery failed: {e}")))?
        .client_id(cfg.oidc.client_id.clone())
        .client_secret(cfg.oidc.client_secret.clone())
        .redirect_url(cfg.oidc.redirect_url.clone())
        .login_path("/auth/login")
        .logout_path("/auth/logout")
        .scopes(&["openid", "email", "profile"])
        .use_dev_cookies(cfg.mock_auth || cfg!(debug_assertions))
        .build(handler))
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

pub async fn me(session: CookieSession<User>) -> Json<MeResponse> {
    Json(MeResponse {
        id: session.state.id,
        email: session.state.email.clone(),
        name: session.state.name.clone(),
    })
}

#[derive(Deserialize)]
pub struct DevLoginQuery {
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Dev-only login endpoint, only mounted when `mock_auth` is on.
/// Creates a session for any email that passes the allowlist — no OIDC hop.
async fn dev_login(State(state): State<DevLoginState>, Query(q): Query<DevLoginQuery>) -> Response {
    if !state
        .allowed
        .iter()
        .any(|a| a.eq_ignore_ascii_case(&q.email))
    {
        return (StatusCode::FORBIDDEN, "email not on allowlist").into_response();
    }
    let user = User {
        id: Uuid::now_v7(),
        subject: format!("dev:{}", q.email),
        email: q.email,
        name: q.name,
        created_at: Timestamp::now(),
    };
    match state.sessions.create_session(user).await {
        Ok(cookie) => (cookie, Redirect::to("/")).into_response(),
        Err(e) => {
            tracing::error!(?e, "dev session error");
            (StatusCode::INTERNAL_SERVER_ERROR, "session error").into_response()
        }
    }
}

#[derive(Clone)]
struct DevLoginState {
    sessions: Sessions,
    allowed: Arc<Vec<String>>,
}

pub fn dev_routes(cfg: &Config, sessions: Sessions) -> Router {
    Router::new()
        .route("/auth/dev-login", get(dev_login))
        .with_state(DevLoginState {
            sessions,
            allowed: Arc::new(cfg.allowed_emails.clone()),
        })
}
