//! This module defines the HTTP handlers for authentication-related routes.

use crate::api_state::ApiContext;
use axum::{Extension, Json, extract::State, http::StatusCode};
use common_services::api::auth::error::AuthError;
use common_services::api::auth::interfaces::{CreateUser, LoginUser, RefreshTokenPayload, Tokens};
use common_services::api::auth::service::{
    authenticate_user, create_access_token, create_user, logout_user, refresh_tokens,
    store_refresh_token,
};
use common_services::api::auth::token::generate_refresh_token_parts;
use common_services::database::app_user::User;
use tracing::instrument;

/// Handles user login and returns a new set of tokens.
///
/// # Errors
///
/// Returns `AuthError` if the user credentials are invalid or if there's a
/// problem creating or storing the tokens.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Login successful", body = Tokens),
        (status = 401, description = "Invalid credentials"),
    )
)]
#[instrument(skip(context, payload), err(Debug))]
pub async fn login(
    State(context): State<ApiContext>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Tokens>, AuthError> {
    let user = authenticate_user(&context.pool, &payload.email, &payload.password).await?;
    let (access_token, expiry) =
        create_access_token(&context.settings.secrets.jwt, user.id, user.role)?;
    let token_parts = generate_refresh_token_parts()?;
    store_refresh_token(&context.pool, user.id, &token_parts).await?;

    Ok(Json(Tokens {
        expiry,
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
    tag = "Auth",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 409, description = "User with this email already exists"),
    )
)]
#[instrument(skip(context, payload), err(Debug))]
pub async fn register(
    State(context): State<ApiContext>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, AuthError> {
    let user = create_user(&context.pool, &payload).await?;
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
    tag = "Auth",
    request_body = RefreshTokenPayload,
    responses(
        (status = 200, description = "Session refreshed successfully", body = Tokens),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
#[instrument(skip(context, payload), err(Debug))]
pub async fn refresh_session(
    State(context): State<ApiContext>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<Tokens>, AuthError> {
    refresh_tokens(
        &context.pool,
        &context.settings.secrets.jwt,
        &payload.refresh_token,
    )
    .await
}

/// Handles user logout by invalidating the provided refresh token.
///
/// # Errors
///
/// Returns `AuthError` if the refresh token is invalid or could not be found.
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    request_body = RefreshTokenPayload,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
pub async fn logout(
    State(context): State<ApiContext>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<StatusCode, AuthError> {
    logout_user(&context.pool, &payload.refresh_token).await
}

/// Get current user info.
///
/// # Errors
///
/// Returns `AuthError` if the refresh token is invalid or could not be found.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "Auth",
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
