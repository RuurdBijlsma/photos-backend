use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::eyre;
use serde_json::json;
use tracing::{info, warn};

pub enum AuthError {
    MissingToken,
    InvalidToken,
    InvalidCredentials,            // New
    RefreshTokenExpiredOrNotFound, // New
    UserAlreadyExists,             // New
    UserNotFound,
    PermissionDenied { user_email: String, path: String },
    Internal(eyre::Report),
}

// Helper function to log failures.
fn log_auth_failure(error: &AuthError) {
    match error {
        AuthError::MissingToken => warn!("Authentication failed: Missing Authorization token."),
        AuthError::InvalidToken => warn!("Authentication failed: Invalid token provided."),
        AuthError::InvalidCredentials => {
            info!("Authentication failed: Invalid credentials provided.")
        } // Use info to reduce noise
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
            tracing::error!("Internal server error during authentication: {:?}", e)
        }
    }
}

// Implementation to turn an AuthError into a user-facing response.
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        log_auth_failure(&self);

        let (status, error_message) = match self {
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password")
            }
            AuthError::MissingToken
            | AuthError::InvalidToken
            | AuthError::UserNotFound
            | AuthError::RefreshTokenExpiredOrNotFound => {
                (StatusCode::UNAUTHORIZED, "Authentication failed")
            }
            AuthError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "A user with this email already exists",
            ),
            AuthError::PermissionDenied { .. } => (StatusCode::FORBIDDEN, "Permission denied"),
            AuthError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred",
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

// This allows us to use `?` to convert `sqlx::Error` and other errors into `AuthError::Internal`.
impl<E> From<E> for AuthError
where
    E: Into<eyre::Report>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}
