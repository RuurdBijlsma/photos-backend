use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum CameraError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal error")]
    Internal(#[from] eyre::Report),

    #[error("Person not found: {0}")]
    NotFound(String),
}

fn log_error(error: &CameraError) {
    match error {
        CameraError::Database(e) => warn!("Camera -> Database query failed: {}", e),
        CameraError::Internal(e) => warn!("Camera -> Internal error: {:?}", e),
        CameraError::NotFound(id) => warn!("Camera -> Person not found: {}", id),
    }
}

impl IntoResponse for CameraError {
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
            Self::NotFound(id) => (StatusCode::NOT_FOUND, format!("Person not found: {id}")),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<DbError> for CameraError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Sqlx(sql_err) | DbError::UniqueViolation(sql_err) => Self::Database(sql_err),
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
