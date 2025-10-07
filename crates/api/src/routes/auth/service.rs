// in service.rs

use crate::auth::model::*;
use crate::auth::token::*;
use crate::routes::auth::error::AuthError;
use crate::routes::auth::hashing::{hash_password, verify_password};
use axum::Json;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use common_photos::get_config;
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::{Executor, PgPool, Postgres};
use tracing::info;

pub async fn authenticate_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<UserRecord, AuthError> {
    let user = sqlx::query_as!(
        UserRecord,
        r#"SELECT id, email, name, password, role as "role: UserRole", created_at, updated_at
           FROM app_user WHERE email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::InvalidCredentials)?;

    let valid = verify_password(password.as_ref(), &user.password)?;
    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(user)
}

pub async fn create_user(pool: &PgPool, payload: &CreateUser) -> Result<User, AuthError> {
    let hashed = hash_password(payload.password.as_ref())?;
    info!("Creating user {:?}", payload);
    let result = sqlx::query_as!(
        User,
        r#"
        INSERT INTO app_user (email, name, password)
        VALUES ($1, $2, $3)
        RETURNING id, email, name, media_folder, role as "role: UserRole", created_at, updated_at
        "#,
        payload.email,
        payload.name,
        hashed
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AuthError::UserAlreadyExists)
        }
        Err(e) => Err(e.into()), // Convert other errors to AuthError::Internal
    }
}

pub async fn store_refresh_token<'c, E>(
    executor: E,
    user_id: i32,
    parts: &RefreshTokenParts,
) -> Result<(), AuthError>
where
    E: Executor<'c, Database = Postgres>,
{
    let exp = Utc::now() + Duration::days(get_config().auth.refresh_token_expiry_days);
    sqlx::query!(
        "INSERT INTO refresh_token (user_id, selector, verifier_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
        user_id,
        parts.selector,
        parts.verifier_hash,
        exp
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub fn create_access_token(user_id: i32, role: UserRole) -> Result<String, AuthError> {
    let cfg = get_config();
    let exp =
        (Utc::now() + Duration::minutes(cfg.auth.access_token_expiry_minutes)).timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        role,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.auth.jwt_secret.as_ref()),
    )
    .map_err(Into::into) // Convert jsonwebtoken::Error into AuthError::Internal
}

/// Handles refresh token rotation
pub async fn refresh_tokens(pool: &PgPool, raw_token: &str) -> Result<Json<Tokens>, AuthError> {
    let (selector, verifier_bytes) = split_refresh_token(raw_token)?;
    let record = sqlx::query!(
        "SELECT user_id, verifier_hash FROM refresh_token
         WHERE selector = $1 AND expires_at > NOW()",
        selector
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::RefreshTokenExpiredOrNotFound)?;

    if !verify_token(&verifier_bytes, &record.verifier_hash)? {
        // As a security measure, if the verifier is wrong, we assume token theft
        // and delete all refresh tokens for that user.
        sqlx::query!(
            "DELETE FROM refresh_token WHERE user_id = $1",
            record.user_id
        )
        .execute(pool)
        .await
        .ok(); // Ignore error if deletion fails
        return Err(AuthError::InvalidToken);
    }

    // Safely look up the user's role
    let user_role = sqlx::query!(
        r#"SELECT role as "role: UserRole" FROM app_user WHERE id = $1"#,
        record.user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AuthError::UserNotFound)?
    .role;

    // Begin transaction for token rotation
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
        .execute(&mut *tx)
        .await?;

    let new_parts = generate_refresh_token_parts()?;
    store_refresh_token(&mut *tx, record.user_id, &new_parts).await?;

    tx.commit().await?;

    let access_token = create_access_token(record.user_id, user_role)?;
    Ok(Json(Tokens {
        access_token,
        refresh_token: new_parts.raw_token,
    }))
}

/// Deletes the refresh token matching the provided one
pub async fn logout_user(pool: &PgPool, raw_token: &str) -> Result<StatusCode, AuthError> {
    // If the token is malformed, we just ignore it and succeed silently.
    if let Ok((selector, verifier_bytes)) = split_refresh_token(raw_token)
        && let Some(rec) = sqlx::query!(
            "SELECT user_id, verifier_hash FROM refresh_token WHERE selector = $1",
            selector
        )
        .fetch_optional(pool)
        .await?
        && verify_token(&verifier_bytes, &rec.verifier_hash).unwrap_or(false)
    {
        sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
            .execute(pool)
            .await?;
    }
    // Logout should always appear successful to prevent token enumeration attacks.
    Ok(StatusCode::NO_CONTENT)
}

// The internal_err function is no longer needed!
