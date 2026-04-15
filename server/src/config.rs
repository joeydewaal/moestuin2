use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: String,
    pub database_url: String,
    pub mock_hardware: bool,
    pub allowed_emails: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            bind: env::var("MOESTUIN_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: env::var("MOESTUIN_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:moestuin.db".into()),
            mock_hardware: env::var("MOESTUIN_MOCK_HW").ok().as_deref() == Some("1"),
            allowed_emails: env::var("MOESTUIN_ALLOWED_EMAILS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }
}
