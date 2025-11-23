use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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

    #[error("Forbidden: {0}")]
    Forbidden(String),
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
                StatusCode::FORBIDDEN,
                "Token is invalid or expired.".to_string(),
            ),
            Self::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied.".to_string()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".to_string(),
            ),
            Self::Forbidden(message) => {
                (StatusCode::FORBIDDEN, format!("Forbidden: {message}"))
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

impl From<DbError> for S2SError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Sqlx(sql_err) => Self::Database(sql_err),
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
