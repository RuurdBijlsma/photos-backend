use axum::{Extension, Json, extract::State, http::StatusCode};
use sqlx::PgPool;

use crate::auth::{
    model::{
        AdminResponse, CreateUser, LoginUser, ProtectedResponse, RefreshTokenPayload, Tokens, User,
    },
    service::{
        authenticate_user, create_access_token, create_user, logout_user, refresh_tokens,
        store_refresh_token,
    },
    token::generate_refresh_token_parts,
};

use crate::routes::auth::error::AuthError;

/// Login to get a new session.
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

/// Register a new user.
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

/// Refresh the session using a refresh token.
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

/// Logout and invalidate the refresh token.
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

/// Get the current user's details.
#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user data", body = ProtectedResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_me(
    Extension(user): Extension<User>,
) -> Result<Json<ProtectedResponse>, StatusCode> {
    Ok(Json(ProtectedResponse {
        message: "You are accessing a protected route!".into(),
        user_email: user.email,
        user_id: user.id,
    }))
}

/// Check if the current user is an admin.
#[utoipa::path(
    get,
    path = "/auth/admin-check",
    responses(
        (status = 200, description = "Admin check successful", body = AdminResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin privileges required"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn check_admin(
    Extension(user): Extension<User>,
) -> Result<Json<AdminResponse>, StatusCode> {
    Ok(Json(AdminResponse {
        message: "You are an admin!".into(),
        user_email: user.email,
        user_id: user.id,
        user_role: user.role,
    }))
}
