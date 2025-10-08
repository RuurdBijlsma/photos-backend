use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use tracing::{info, warn};

/// Represents various authentication-related errors.
pub enum AuthError {
    MissingToken,
    InvalidToken,
    InvalidCredentials,
    RefreshTokenExpiredOrNotFound,
    UserAlreadyExists,
    UserNotFound,
    PermissionDenied { user_email: String, path: String },
    Internal(eyre::Report),
}

/// Helper function to log authentication failures with appropriate tracing levels.
#[allow(clippy::cognitive_complexity)]
fn log_auth_failure(error: &AuthError) {
    match error {
        AuthError::MissingToken => warn!("Authentication failed: Missing Authorization token."),
        AuthError::InvalidToken => warn!("Authentication failed: Invalid token provided."),
        AuthError::InvalidCredentials => {
            info!("Authentication failed: Invalid credentials provided.");
        }
        AuthError::RefreshTokenExpiredOrNotFound => info!("Refresh token not found or expired."),
        AuthError::UserAlreadyExists => info!("Registration failed: User already exists."),
        AuthError::UserNotFound => warn!("Authentication failed: User from token not found."),
        AuthError::PermissionDenied { user_email, path } => {
            warn!(
                "Authorization failed: User {} tried to access admin endpoint: {}",
                user_email, path
            );
        }
        AuthError::Internal(e) => {
            tracing::error!("Internal server error during authentication: {:?}", e);
        }
    }
}

/// Converts an `AuthError` into an Axum `Response`, including logging and a JSON error body.
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        log_auth_failure(&self);

        let (status, error_message) = match self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid email or password"),
            Self::MissingToken
            | Self::InvalidToken
            | Self::UserNotFound
            | Self::RefreshTokenExpiredOrNotFound => {
                (StatusCode::UNAUTHORIZED, "Authentication failed")
            }
            Self::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "A user with this email already exists",
            ),
            Self::PermissionDenied { .. } => (StatusCode::FORBIDDEN, "Permission denied"),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred",
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

/// Allows conversion from any type that can be converted into `eyre::Report` into `AuthError::Internal`.
impl<E> From<E> for AuthError
where
    E: Into<eyre::Report>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}
