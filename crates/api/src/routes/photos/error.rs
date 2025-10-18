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
}

fn log_setup_failure(error: &PhotosError) {
    match error {
        PhotosError::Database(e) => error!("Database query failed: {}", e),
        PhotosError::Internal(e) => println!("Error in /setup: {e:?}"),
    }
}

impl IntoResponse for PhotosError {
    fn into_response(self) -> Response {
        log_setup_failure(&self);

        let (status, error_message): (_, &str) = match self {
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred.",
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.",
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
