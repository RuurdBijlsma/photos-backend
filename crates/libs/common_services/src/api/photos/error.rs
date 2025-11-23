use crate::database::DbError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum PhotosError {
    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("internal error")]
    Internal(#[from] eyre::Report),

    #[error("Media item not found: {0}")]
    MediaNotFound(String),

    #[error("Invalid media path provided.")]
    InvalidPath,

    #[error("Permission denied while accessing media file.")]
    AccessDenied,

    #[error("The requested file is not a supported media type.")]
    UnsupportedMediaType,
}

impl IntoResponse for PhotosError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred.".to_string(),
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".to_string(),
            ),
            Self::MediaNotFound(media_id) => (
                StatusCode::NOT_FOUND,
                format!("Media item not found: {media_id}"),
            ),
            Self::InvalidPath => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::AccessDenied => (StatusCode::FORBIDDEN, self.to_string()),
            Self::UnsupportedMediaType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<tokio::task::JoinError> for PhotosError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<DbError> for PhotosError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Sqlx(sql_err) => Self::Database(sql_err),
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
