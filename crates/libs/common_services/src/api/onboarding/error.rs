use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use color_eyre::eyre;
use serde_json::json;
use std::path::StripPrefixError;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error)]
pub enum OnboardingError {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("path is not within the media directory")]
    PathNotInMediaDir(#[from] StripPrefixError),

    #[error("i/o error")]
    Io(#[from] std::io::Error),

    #[error("failed to create directory with invalid name: {0}")]
    DirectoryCreation(String),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("media folder already set")]
    MediaFolderAlreadySet,

    #[error("internal error")]
    Internal(#[from] eyre::Report),
}

fn log_failure(error: &OnboardingError) {
    match error {
        OnboardingError::InvalidPath(path) => warn!("Invalid path provided: {}", path),
        OnboardingError::PathNotInMediaDir(e) => error!("Path hierarchy error: {}", e),
        OnboardingError::Io(e) => error!("I/O error: {}", e),
        OnboardingError::DirectoryCreation(path) => error!("Failed to create directory: {}", path),
        OnboardingError::Database(e) => error!("Database query failed: {}", e),
        OnboardingError::Internal(e) => println!("Error in /onboarding: {e:?}"),
        OnboardingError::MediaFolderAlreadySet => {
            println!("Tried to set media folder on user that already had it. /onboarding")
        }
    }
}

impl IntoResponse for OnboardingError {
    fn into_response(self) -> Response {
        log_failure(&self);

        let (status, error_message) = match self {
            Self::InvalidPath(path) => (
                StatusCode::BAD_REQUEST,
                format!("The provided path is invalid: {path}"),
            ),
            Self::DirectoryCreation(name) => (
                StatusCode::BAD_REQUEST,
                format!("The directory name '{name}' contains invalid characters."),
            ),
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred.".into(),
            ),
            Self::PathNotInMediaDir(_) | Self::Io(_) | Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".into(),
            ),
            Self::MediaFolderAlreadySet => (
                StatusCode::UNAUTHORIZED,
                "Media folder is already configured for this user.".into(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<tokio::task::JoinError> for OnboardingError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}
