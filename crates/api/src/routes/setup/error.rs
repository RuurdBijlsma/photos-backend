use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use color_eyre::eyre;
use serde_json::json;
use std::path::StripPrefixError;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("path is not within the media directory")]
    PathNotInMediaDir(#[from] StripPrefixError),

    #[error("i/o error")]
    Io(#[from] std::io::Error),

    #[error("failed to create directory: {0}")]
    DirectoryCreation(String),

    // This single attribute correctly and safely handles the conversion from
    // `color_eyre::Result`, resolving the conflict.
    #[error(transparent)]
    Internal(#[from] eyre::Report),
}

#[allow(clippy::cognitive_complexity)]
fn log_setup_failure(error: &SetupError) {
    match error {
        SetupError::InvalidPath(path) => warn!("Invalid path provided: {}", path),
        SetupError::PathNotInMediaDir(e) => error!("Path hierarchy error: {}", e),
        SetupError::Io(e) => error!("I/O error: {}", e),
        SetupError::DirectoryCreation(path) => error!("Failed to create directory: {}", path),
        SetupError::Internal(e) => error!("Internal server error in setup: {:?}", e),
    }
}

impl IntoResponse for SetupError {
    fn into_response(self) -> Response {
        log_setup_failure(&self);

        let (status, error_message) = match &self {
            Self::InvalidPath(path) => (
                StatusCode::BAD_REQUEST,
                format!("The provided path is invalid: {path}"),
            ),
            Self::PathNotInMediaDir(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred while processing a file path.".into(),
            ),
            Self::Io(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A filesystem error occurred.".into(),
            ),
            Self::DirectoryCreation(name) => (
                StatusCode::BAD_REQUEST,
                format!(
                    "The directory '{name}' contains invalid characters and cannot be created."
                ),
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".into(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<tokio::task::JoinError> for SetupError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<sqlx::Error> for SetupError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}
