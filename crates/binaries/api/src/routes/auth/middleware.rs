use std::convert::Infallible;
use async_trait::async_trait;
use crate::auth::db_model::User;
use crate::routes::auth::error::AuthError;
use crate::routes::auth::interfaces::Claims;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use color_eyre::eyre::eyre;
use http::header;
use http::request::Parts;
use common_photos::{UserRole, settings};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sqlx::PgPool;
use tracing::warn;

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
        parts: &mut Parts,
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

        let jwt_secret = &settings().auth.jwt_secret;
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
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

/// A newtype wrapper for `Option<User>`.
/// This is used as an extractor that attempts to authenticate a user
/// but does not fail if they are not logged in.
#[derive(Clone, Debug)]
pub struct OptionalUser(pub Option<User>);

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
    State<PgPool>: FromRequestParts<S>,
{
    // This extractor will never fail, so the rejection is Infallible.
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Attempt to get the token from the "Authorization" header.
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|str_value| str_value.strip_prefix("Bearer "));

        // If no valid Bearer token is found, we succeed with `None`.
        let Some(token) = token else {
            parts.extensions.insert(Self(None));
            return Ok(Self(None));
        };

        // Attempt to decode the token.
        let jwt_secret = &settings().auth.jwt_secret;
        let claims = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token_data) => token_data.claims,
            Err(e) => {
                // Token is invalid, but we don't fail the request.
                warn!("Invalid token found for optional authentication: {}", e);
                parts.extensions.insert(Self(None));
                return Ok(Self(None));
            }
        };

        // This should not fail if the router is configured correctly with `.with_state()`.
        let Ok(State(pool)) = State::<PgPool>::from_request_parts(parts, state).await else {
            tracing::error!("Could not extract PgPool state for OptionalUser. Router is misconfigured.");
            parts.extensions.insert(Self(None));
            return Ok(Self(None));
        };

        // Fetch the user from the database.
        let user_result = sqlx::query_as!(
            User,
            r#"SELECT id, email, name, media_folder, role as "role: UserRole",
                      created_at, updated_at
               FROM app_user WHERE id = $1"#,
            claims.sub
        )
            .fetch_optional(&pool)
            .await;

        match user_result {
            Ok(user_opt) => {
                // If the user doesn't exist, user_opt will be None, which is the desired outcome.
                if user_opt.is_none() {
                    warn!("User ID {} from token not found in database.", claims.sub);
                }
                parts.extensions.insert(Self(user_opt.clone()));
                Ok(Self(user_opt))
            }
            Err(e) => {
                warn!("Database error during optional user lookup: {}", e);
                parts.extensions.insert(Self(None));
                Ok(Self(None))
            }
        }
    }
}