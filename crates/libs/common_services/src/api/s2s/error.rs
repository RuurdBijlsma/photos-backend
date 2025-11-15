use crate::s2s_client::error::S2sClientError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum S2SError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Token is invalid or expired")]
    TokenInvalid,

    #[error("Permission denied: The requested media is not part of this album share")]
    PermissionDenied,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("internal error")]
    Internal(#[from] eyre::Report),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for S2SError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(e) => {
                error!("S2S Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred.".to_string(),
                )
            }
            Self::TokenInvalid => (
                StatusCode::UNAUTHORIZED,
                "Token is invalid or expired.".to_string(),
            ),
            Self::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied.".to_string()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".to_string(),
            ),
            Self::Unauthorized(message) => {
                (StatusCode::UNAUTHORIZED, format!("Unauthorized: {message}"))
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<jsonwebtoken::errors::Error> for S2SError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<S2sClientError> for S2SError {
    fn from(err: S2sClientError) -> Self {
        match err {
            S2sClientError::JwtError(_) => Self::TokenInvalid,
            S2sClientError::UrlParseError(e) => Self::Internal(eyre::Report::new(e)),
            S2sClientError::RequestError(e) => Self::Internal(eyre::Report::new(e)),
            S2sClientError::RemoteServerError(msg) => Self::Internal(eyre::eyre!(msg)),
        }
    }
}