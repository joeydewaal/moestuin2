use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] BoxError),
}

impl AppError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into().into())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(Box::new(e))
    }
}

impl From<std::net::AddrParseError> for AppError {
    fn from(e: std::net::AddrParseError) -> Self {
        Self::Internal(Box::new(e))
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: &'a str,
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    error: ErrorBody<'a>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "unauthorized".to_string(),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "not found".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", m.clone()),
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL",
                    "something went wrong".to_string(),
                )
            }
        };
        (
            status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code,
                    message: &message,
                },
            }),
        )
            .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
