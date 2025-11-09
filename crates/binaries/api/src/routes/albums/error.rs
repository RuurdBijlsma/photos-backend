use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::error;
use uuid;

#[derive(Debug, Error)]
pub enum AlbumsError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("internal error")]
    Internal(#[from] eyre::Report),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
}

// Renamed for more general use
fn log_error(error: &AlbumsError) {
    match error {
        AlbumsError::Database(e) => error!("Database query failed: {}", e),
        AlbumsError::Internal(e) => error!("Internal error: {:?}", e),
        AlbumsError::NotFound(id) => {
            error!("Media item not found: {}", id);
        }
        AlbumsError::InvalidUuid(e) => error!("Invalid UUID provided: {}", e),
    }
}

impl IntoResponse for AlbumsError {
    fn into_response(self) -> Response {
        log_error(&self);

        let (status, error_message) = match self {
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred.".to_string(),
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".to_string(),
            ),
            Self::NotFound(message) => (
                StatusCode::NOT_FOUND,
                format!("Album not found: {message}"),
            ),
            Self::InvalidUuid(e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid ID format: {e}"),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<tokio::task::JoinError> for AlbumsError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}