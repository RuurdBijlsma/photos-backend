use crate::database::DbError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Database error: {0}")]
    Db(#[from] DbError),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for SystemError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Db(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}

impl From<sqlx::Error> for SystemError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err.into())
    }
}
