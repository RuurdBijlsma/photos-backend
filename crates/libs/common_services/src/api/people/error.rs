use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum PeopleError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal error")]
    Internal(#[from] eyre::Report),

    #[error("Person not found: {0}")]
    NotFound(i64),
}

fn log_error(error: &PeopleError) {
    match error {
        PeopleError::Database(e) => warn!("People -> Database query failed: {}", e),
        PeopleError::Internal(e) => warn!("People -> Internal error: {:?}", e),
        PeopleError::NotFound(id) => warn!("People -> Person not found: {}", id),
    }
}

impl IntoResponse for PeopleError {
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

impl From<DbError> for PeopleError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Sqlx(sql_err) => Self::Database(sql_err),
            DbError::UniqueViolation(sql_err) => Self::Database(sql_err),
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
