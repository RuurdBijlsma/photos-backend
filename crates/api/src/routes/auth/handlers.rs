//! This module defines the HTTP handlers for authentication-related routes.

use axum::{Extension, Json, extract::State, http::StatusCode};
use sqlx::PgPool;
use crate::auth::db_model::User;
use crate::auth::error::AuthError;
use crate::auth::interfaces::{CreateUser, LoginUser, RefreshTokenPayload, Tokens};
use crate::auth::service::{
    authenticate_user, create_access_token, create_user, logout_user, refresh_tokens,
    store_refresh_token,
};
use crate::auth::token::generate_refresh_token_parts;

/// Handles user login and returns a new set of tokens.
///
/// # Errors
///
/// Returns `AuthError` if the user credentials are invalid or if there's a
/// problem creating or storing the tokens.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Login successful", body = Tokens),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Tokens>, AuthError> {
    let user = authenticate_user(&pool, &payload.email, &payload.password).await?;
    let access_token = create_access_token(user.id, user.role)?;
    let token_parts = generate_refresh_token_parts()?;
    store_refresh_token(&pool, user.id, &token_parts).await?;

    Ok(Json(Tokens {
        access_token,
        refresh_token: token_parts.raw_token,
    }))
}

/// Handles the registration of a new user.
///
/// # Errors
///
/// Returns `AuthError` if a user with the provided email already exists or
/// if a database error occurs during user creation.
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 409, description = "User with this email already exists"),
    )
)]
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, AuthError> {
    let user = create_user(&pool, &payload).await?;
    Ok(Json(user))
}

/// Handles refreshing the session using a valid refresh token.
///
/// # Errors
///
/// Returns `AuthError` if the refresh token is invalid, expired, or not found in the database.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshTokenPayload,
    responses(
        (status = 200, description = "Session refreshed successfully", body = Tokens),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
pub async fn refresh_session(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<Tokens>, AuthError> {
    refresh_tokens(&pool, &payload.refresh_token).await
}

/// Handles user logout by invalidating the provided refresh token.
///
/// # Errors
///
/// Returns `AuthError` if the refresh token is invalid or could not be found.
#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = RefreshTokenPayload,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
pub async fn logout(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<StatusCode, AuthError> {
    logout_user(&pool, &payload.refresh_token).await
}

/// Get current user info.
///
/// # Errors
///
/// Returns `AuthError` if the refresh token is invalid or could not be found.
#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user data", body = User),
        (status = 401, description = "Authentication required"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_me(Extension(user): Extension<User>) -> Result<Json<User>, StatusCode> {
    Ok(Json(user))
}
