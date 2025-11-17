use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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

    #[error("Invalid month format. Expected YYYY-MM-DD, but got '{0}'")]
    InvalidMonthFormat(String),

    #[error("Media item not found: {0}")]
    MediaNotFound(String),
}

// Renamed for more general use
fn log_error(error: &PhotosError) {
    match error {
        PhotosError::Database(e) => error!("Database query failed: {}", e),
        PhotosError::Internal(e) => error!("Internal error: {}", e),
        PhotosError::InvalidMonthFormat(month) => {
            error!("Invalid month format provided: {}", month);
        }
        PhotosError::MediaNotFound(id) => {
            error!("Media item not found: {}", id);
        }
    }
}

impl IntoResponse for PhotosError {
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
            Self::InvalidMonthFormat(invalid_month) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid month format. Expected YYYY-MM-DD, but got '{invalid_month}'"),
            ),
            Self::MediaNotFound(media_id) => (
                StatusCode::NOT_FOUND,
                format!("Media item not found: {media_id}"),
            ),
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
