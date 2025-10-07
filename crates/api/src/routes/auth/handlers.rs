use axum::{Extension, Json, extract::State, http::StatusCode};
use sqlx::PgPool;

use crate::auth::{model::*, service::*, token::*};

/// POST /auth/login
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Tokens>, (StatusCode, String)> {
    let user = authenticate_user(&pool, &payload.email, &payload.password).await?;
    let access_token = create_access_token(user.id, user.role)?;

    let token_parts = generate_refresh_token_parts()?;
    store_refresh_token(&pool, user.id, &token_parts).await?;

    Ok(Json(Tokens {
        access_token,
        refresh_token: token_parts.raw_token,
    }))
}

/// POST /auth/register
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user = create_user(&pool, &payload).await?;
    Ok(Json(user))
}

/// POST /auth/refresh
pub async fn refresh_session(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<Tokens>, (StatusCode, String)> {
    refresh_tokens(&pool, &payload.refresh_token).await
}

/// POST /auth/logout
pub async fn logout(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    logout_user(&pool, &payload.refresh_token).await
}

/// GET /auth/me
pub async fn get_me(
    Extension(user): Extension<User>,
) -> Result<Json<ProtectedResponse>, StatusCode> {
    Ok(Json(ProtectedResponse {
        message: "You are accessing a protected route!".into(),
        user_email: user.email,
        user_id: user.id,
    }))
}

/// GET /auth/admin-check
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
