//! This module defines the HTTP handlers for authentication-related routes.

use crate::api_state::ApiState;
use axum::{Extension, Json, extract::State, http::StatusCode};
use common_services::api::auth::error::AuthError;
use common_services::api::auth::interfaces::{CreateUser, LoginUser, RefreshTokenPayload, Tokens};
use common_services::api::auth::service::{
    authenticate_user, create_access_token, create_user, logout_user, refresh_tokens,
    store_refresh_token,
};
use common_services::api::auth::token::generate_refresh_token_parts;
use common_services::database::app_user::User;

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
    State(api_state): State<ApiState>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Tokens>, AuthError> {
    let user = authenticate_user(&api_state.pool, &payload.email, &payload.password).await?;
    let (access_token, expiry) = create_access_token(user.id, user.role)?;
    let token_parts = generate_refresh_token_parts()?;
    store_refresh_token(&api_state.pool, user.id, &token_parts).await?;

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
    request_body = CreateUser,
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 409, description = "User with this email already exists"),
    )
)]
pub async fn register(
    State(api_state): State<ApiState>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, AuthError> {
    let user = create_user(&api_state.pool, &payload).await?;
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
    State(api_state): State<ApiState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<Tokens>, AuthError> {
    refresh_tokens(&api_state.pool, &payload.refresh_token).await
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
    State(api_state): State<ApiState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<StatusCode, AuthError> {
    logout_user(&api_state.pool, &payload.refresh_token).await
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
