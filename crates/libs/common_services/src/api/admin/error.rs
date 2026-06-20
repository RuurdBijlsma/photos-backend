use crate::api::album::error::AlbumError;
use crate::api::user::error::UserError;
use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use std::path::StripPrefixError;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("path is not within the media directory")]
    PathNotInMediaDir(#[from] StripPrefixError),

    #[error("i/o error")]
    Io(#[from] std::io::Error),

    #[error("Get disk info error")]
    GetDiskInfoError(#[from] UserError),

    #[error("failed to create directory with invalid name: {0}")]
    DirectoryCreation(String),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("media folder already set")]
    MediaFolderAlreadySet,

    #[error("cannot delete your own administrator account")]
    CannotDeleteSelf,

    #[error("folder is already in use by another user")]
    FolderInUse,

    #[error("internal error")]
    Internal(#[from] color_eyre::Report),

    #[error("album error")]
    AlbumError(#[from] AlbumError),
}

fn log_failure(error: &AdminError) {
    match error {
        AdminError::InvalidPath(path) => warn!("Invalid path provided: {}", path),
        AdminError::PathNotInMediaDir(e) => error!("Path hierarchy error: {}", e),
        AdminError::Io(e) => error!("I/O error: {}", e),
        AdminError::DirectoryCreation(path) => error!("Failed to create directory: {}", path),
        AdminError::Database(e) => error!("Database query failed: {}", e),
        AdminError::Internal(e) => println!("Error in /admin: {e:?}"),
        AdminError::AlbumError(e) => error!("Album query failed: {}", e),
        AdminError::GetDiskInfoError(e) => error!("Get disk info failed: {}", e),
        AdminError::FolderInUse => {
            println!("Folder already in use by another user");
        }
        AdminError::MediaFolderAlreadySet => {
            warn!("Tried to set media folder on user that already had it. /admin");
        }
        AdminError::CannotDeleteSelf => {
            warn!("An administrator tried to delete their own account.");
        }
    }
}

impl IntoResponse for AdminError {
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
                StatusCode::FORBIDDEN,
                "Media folder is already configured for this user.".into(),
            ),
            Self::CannotDeleteSelf => (
                StatusCode::BAD_REQUEST,
                "An administrator cannot delete their own account.".into(),
            ),
            Self::FolderInUse => (
                StatusCode::BAD_REQUEST,
                "Folder already in use by another user.".into(),
            ),
            Self::GetDiskInfoError(_) => (StatusCode::BAD_REQUEST, "Couldn't get disk info".into()),
            Self::AlbumError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<tokio::task::JoinError> for AdminError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<DbError> for AdminError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::UniqueViolation(sql_err) => Self::Database(sql_err),
            DbError::Sqlx(err) => Self::Internal(eyre::Report::new(err)),
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
