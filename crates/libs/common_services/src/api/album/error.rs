use crate::database::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn};
use url::ParseError;

#[derive(Debug, Error)]
pub enum AlbumError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("internal error")]
    Internal(#[from] eyre::Report),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid invitation token: {0}")]
    InvalidInviteToken(String),

    #[error("Remote server error: {0}")]
    RemoteServerError(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),
}

// Renamed for more general use
fn log_error(error: &AlbumError) {
    match error {
        AlbumError::Database(e) => warn!("Database query failed: {}", e),
        AlbumError::Internal(e) => warn!("Internal error: {:?}", e),
        AlbumError::NotFound(id) => {
            warn!("Album -> Media item not found: {}", id);
        }
        AlbumError::Forbidden(id) => {
            warn!("Album -> Forbidden: {}", id);
        }
        AlbumError::InvalidInviteToken(id) => {
            warn!("Invalid invitation token: {}", id);
        }
        AlbumError::RemoteServerError(message) => {
            warn!("Album sharing -> Remote server error: {}", message);
        }
        AlbumError::BadRequest(message) => {
            warn!("Album -> Bad Request: {}", message);
        }
    }
}

impl IntoResponse for AlbumError {
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
            Self::NotFound(message) => {
                (StatusCode::NOT_FOUND, format!("Album not found: {message}"))
            }
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, format!("Forbidden: {message}")),
            Self::InvalidInviteToken(message) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid invite: {message}"),
            ),
            Self::RemoteServerError(message) => (
                StatusCode::BAD_GATEWAY,
                format!("Could not contact remote server: {message}"),
            ),
            Self::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, format!("Bad request: {message}"))
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<reqwest::Error> for AlbumError {
    fn from(err: reqwest::Error) -> Self {
        Self::RemoteServerError(err.to_string())
    }
}

impl From<tokio::task::JoinError> for AlbumError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<jsonwebtoken::errors::Error> for AlbumError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<ParseError> for AlbumError {
    fn from(err: ParseError) -> Self {
        Self::Internal(eyre::Report::new(err))
    }
}

impl From<DbError> for AlbumError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::UniqueViolation(sql_err) => Self::Database(sql_err),
            DbError::Sqlx(sql_err) => {
                if matches!(sql_err, sqlx::Error::RowNotFound) {
                    Self::NotFound("row not found".into())
                } else {
                    Self::Database(sql_err)
                }
            }
            DbError::SerdeJson(err) => Self::Internal(eyre::Report::new(err)),
        }
    }
}
