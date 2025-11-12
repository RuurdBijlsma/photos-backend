use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum S2SError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Token is invalid or expired")]
    TokenInvalid,

    #[error("Permission denied: The requested media is not part of this album share")]
    PermissionDenied,

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for S2SError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(e) => {
                error!("S2S Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred.".to_string(),
                )
            }
            Self::TokenInvalid => (
                StatusCode::UNAUTHORIZED,
                "Token is invalid or expired.".to_string(),
            ),
            Self::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied.".to_string()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
