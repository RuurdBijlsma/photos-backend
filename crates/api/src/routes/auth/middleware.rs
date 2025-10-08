use crate::auth::UserRole;
use crate::auth::model::{Claims, User};
use crate::routes::auth::error::AuthError;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use color_eyre::eyre::eyre;
use common_photos::get_config;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sqlx::PgPool;

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    State<PgPool>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    /// Extracts a `User` from the request parts by validating a JWT from the "Authorization" header.
    /// # Errors
    ///
    /// * `AuthError::MissingToken` if the "Authorization" header is missing.
    /// * `AuthError::InvalidToken` if the token is not a "Bearer" token or is invalid.
    /// * `AuthError::Internal` if the `PgPool` state is not configured correctly.
    /// * `sqlx::Error` if there's a database error.
    /// * `AuthError::UserNotFound` if the user ID from the token does not correspond to an existing user.
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        let cfg = get_config();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(cfg.auth.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        let user_id = token_data.claims.sub;

        let State(pool) = State::<PgPool>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                tracing::error!(
                    "FATAL: Could not extract PgPool state. The router is likely missing `.with_state(pool)`."
                );
                AuthError::Internal(eyre!("Server state is not configured correctly."))
            })?;

        let user = sqlx::query_as!(
            User,
            r#"SELECT id, email, name, media_folder, role as "role: UserRole",
                      created_at, updated_at
               FROM app_user WHERE id = $1"#,
            user_id
        )
        .fetch_optional(&pool)
        .await?
        .ok_or(AuthError::UserNotFound)?;

        parts.extensions.insert(user.clone());
        Ok(user)
    }
}

/// Middleware to require a specific user role for accessing a route.
/// # Errors
///
/// * `AuthError::UserNotFound` if the `User` is not present in the request extensions (i.e., `User` extractor failed).
/// * `AuthError::PermissionDenied` if the authenticated user's role does not match the required role.
pub async fn require_role(
    State(required_role): State<UserRole>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = req
        .extensions()
        .get::<User>()
        .ok_or(AuthError::UserNotFound)?;

    if user.role != required_role {
        // The logging is now handled inside the IntoResponse impl for AuthError
        return Err(AuthError::PermissionDenied {
            user_email: user.email.clone(),
            path: req.uri().to_string(),
        });
    }

    Ok(next.run(req).await)
}
