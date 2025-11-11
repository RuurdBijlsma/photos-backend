use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error)]
pub enum AlbumsError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("internal error")]
    Internal(#[from] eyre::Report),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid invitation token: {0}")]
    InvalidInviteToken(String),

    #[error("Remote server error: {0}")]
    RemoteServerError(String),
}

// Renamed for more general use
fn log_error(error: &AlbumsError) {
    match error {
        AlbumsError::Database(e) => warn!("Database query failed: {}", e),
        AlbumsError::Internal(e) => warn!("Internal error: {:?}", e),
        AlbumsError::NotFound(id) => {
            warn!("Album -> Media item not found: {}", id);
        }
        AlbumsError::Unauthorized(id) => {
            warn!("Unauthorized: {}", id);
        }
        AlbumsError::InvalidInviteToken(id) => {
            warn!("Invalid invitation token: {}", id);
        }
        AlbumsError::RemoteServerError(message) => {
            warn!("Album sharing -> Remote server error: {}", message)
        }
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
            Self::NotFound(message) => {
                (StatusCode::NOT_FOUND, format!("Album not found: {message}"))
            }
            Self::Unauthorized(message) => {
                (StatusCode::UNAUTHORIZED, format!("Unauthorized: {message}"))
            }
            Self::InvalidInviteToken(message) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid invite: {message}"),
            ),
            Self::RemoteServerError(message) => (
                StatusCode::BAD_GATEWAY,
                format!("Could not contact remote server: {message}"),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<reqwest::Error> for AlbumsError {
    fn from(err: reqwest::Error) -> Self {
        Self::RemoteServerError(err.to_string())
    }
}

impl From<tokio::task::JoinError> for AlbumsError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}
