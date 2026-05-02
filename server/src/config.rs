use std::{env, path::PathBuf};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
#[allow(dead_code)] // fields wired up across later milestones (M2+)
pub struct Config {
    pub bind: String,
    pub database_url: String,
    pub mock_hardware: bool,
    pub mock_auth: bool,
    pub allowed_emails: Vec<String>,
    pub cookie_secret: Vec<u8>,
    pub oidc: OidcConfig,
    pub webcam_root: PathBuf,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub post_logout_redirect_url: String,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let mock_auth = env::var("MOESTUIN_MOCK_AUTH").ok().as_deref() == Some("1");

        let cookie_secret = match env::var("MOESTUIN_COOKIE_SECRET") {
            Ok(s) => s.into_bytes(),
            Err(_) if mock_auth || cfg!(debug_assertions) => {
                b"dev-cookie-secret-do-not-use-in-prod-32".to_vec()
            }
            Err(e) => {
                return Err(AppError::internal(format!(
                    "MOESTUIN_COOKIE_SECRET missing: {e}"
                )));
            }
        };

        Ok(Self {
            bind: env::var("MOESTUIN_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: env::var("MOESTUIN_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:moestuin.db".into()),
            mock_hardware: env::var("MOESTUIN_MOCK_HW").ok().as_deref() == Some("1"),
            mock_auth,
            allowed_emails: env::var("MOESTUIN_ALLOWED_EMAILS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            cookie_secret,
            oidc: OidcConfig {
                client_id: env::var("MOESTUIN_OIDC_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("MOESTUIN_OIDC_CLIENT_SECRET").unwrap_or_default(),
                redirect_url: env::var("MOESTUIN_OIDC_REDIRECT")
                    .unwrap_or_else(|_| "http://localhost:8080/auth/callback".into()),
                post_logout_redirect_url: env::var("MOESTUIN_POST_LOGOUT_REDIRECT")
                    .unwrap_or_else(|_| "/".into()),
            },
            webcam_root: env::var("MOESTUIN_WEBCAM_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/data/webcam")),
        })
    }
}
