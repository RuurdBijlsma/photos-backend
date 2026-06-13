use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum DailyCardsError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal error")]
    Internal(#[from] eyre::Report),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

fn log_error(error: &DailyCardsError) {
    match error {
        DailyCardsError::Database(e) => warn!("DailyCards -> Database query failed: {}", e),
        DailyCardsError::Internal(e) => warn!("DailyCards -> Internal error: {:?}", e),
        DailyCardsError::BadRequest(msg) => warn!("DailyCards -> Bad request: {}", msg),
    }
}

impl IntoResponse for DailyCardsError {
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
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<DbError> for DailyCardsError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Sqlx(sql_err) | DbError::UniqueViolation(sql_err) => Self::Database(sql_err),
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
