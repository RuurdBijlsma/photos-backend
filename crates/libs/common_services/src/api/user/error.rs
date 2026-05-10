use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("Media item not found or not accessible")]
    InvalidAvatar,

    #[error("Database error: {0}")]
    Db(#[from] crate::database::DbError),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::InvalidAvatar => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Db(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}

impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err.into())
    }
}
