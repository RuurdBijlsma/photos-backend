use crate::api::auth::error::AuthError;
use crate::api::auth::hashing::{hash_password, verify_password};
use crate::api::auth::interfaces::{AuthClaims, CreateUser, Tokens};
use crate::api::auth::token::{
    RefreshTokenParts, generate_refresh_token_parts, split_refresh_token, verify_token,
};
use crate::database::app_user::{User, UserRole, UserWithPassword};
use crate::database::user_store::UserStore;
use app_state::constants;
use axum::Json;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::{Executor, PgPool, Postgres};
use tracing::info;

/// Authenticates a user based on email and password.
///
/// # Errors
///
/// * `AuthError::InvalidCredentials` if the email or password is incorrect.
/// * `sqlx::Error` for database-related issues.
pub async fn authenticate_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<UserWithPassword, AuthError> {
    let user = UserStore::find_by_email_with_password(pool, email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    let valid = verify_password(password.as_ref(), &user.password)?;
    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(user)
}

/// Creates a new user in the database.
///
/// # Errors
///
/// * `AuthError::UserAlreadyExists` if a user with the given email already exists.
/// * `sqlx::Error` for other database-related issues.
/// * `AuthError::Internal` for hashing errors.
/// * `AuthError::InvalidUsername` when the username contains illegal characters.
pub async fn create_user(pool: &PgPool, payload: &CreateUser) -> Result<User, AuthError> {
    let username = &payload.name;
    if !username.chars().all(|c| c.is_alphanumeric() || c == ' ')
        || username.starts_with(' ')
        || username.ends_with(' ')
    {
        return Err(AuthError::InvalidUsername);
    }
    let hashed = hash_password(payload.password.as_ref())?;
    info!(
        "Creating user email={}, name={}",
        payload.email, payload.name
    );
    let is_first_user = sqlx::query_scalar!("SELECT 1 FROM app_user")
        .fetch_optional(pool)
        .await?
        .flatten()
        .is_none();
    let role = if is_first_user {
        UserRole::Admin
    } else {
        UserRole::User
    };

    if role == UserRole::User {
        // New users will need an invite code, this isn't implemented yet.
        // TODO: add invite code functionality.
        return Err(AuthError::PermissionDenied {
            user_email: payload.email.clone(),
            path: String::new(),
        });
    }

    Ok(UserStore::create(pool, &payload.email, &payload.name, &hashed, role, None).await?)
}

/// Stores a refresh token in the database.
///
/// # Errors
///
/// * `sqlx::Error` for database-related issues.
pub async fn store_refresh_token<'c, E>(
    executor: E,
    user_id: i32,
    parts: &RefreshTokenParts,
) -> Result<(), AuthError>
where
    E: Executor<'c, Database = Postgres>,
{
    let exp = Utc::now() + Duration::days(constants().auth.refresh_token_expiry_days);
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

/// Creates a new access token for a given user ID and role.
///
/// # Errors
///
/// * `jsonwebtoken::Error` if token encoding fails.
pub fn create_access_token(
    jwt_secret: &str,
    user_id: i32,
    role: UserRole,
) -> Result<(String, u64), AuthError> {
    let exp =
        (Utc::now() + Duration::minutes(constants().auth.access_token_expiry_minutes)).timestamp();
    let claims = AuthClaims {
        sub: user_id,
        role,
        exp,
    };
    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(Into::<AuthError>::into)?;

    Ok((access_token, exp as u64))
}

/// Handles refresh token rotation, invalidating the old token and issuing a new pair.
///
/// # Errors
/// * `AuthError::InvalidToken` if the provided refresh token is malformed or invalid.
/// * `AuthError::RefreshTokenExpiredOrNotFound` if the refresh token is not found or has expired.
/// * `AuthError::UserNotFound` if the user associated with the token cannot be found.
/// * `sqlx::Error` for database transaction issues.
pub async fn refresh_tokens(
    pool: &PgPool,
    jwt_secret: &str,
    raw_token: &str,
) -> Result<Json<Tokens>, AuthError> {
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
        // If the verifier is wrong, assume token theft and delete all refresh tokens for that user.
        sqlx::query!(
            "DELETE FROM refresh_token WHERE user_id = $1",
            record.user_id
        )
        .execute(pool)
        .await
        .ok(); // Ignore error if deletion fails
        return Err(AuthError::InvalidToken);
    }

    let user_role = UserStore::get_user_role(pool, record.user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
        .execute(&mut *tx)
        .await?;

    let new_parts = generate_refresh_token_parts()?;
    store_refresh_token(&mut *tx, record.user_id, &new_parts).await?;

    tx.commit().await?;

    let (access_token, expiry) = create_access_token(jwt_secret, record.user_id, user_role)?;
    Ok(Json(Tokens {
        expiry,
        access_token,
        refresh_token: new_parts.raw_token,
    }))
}

/// Deletes the refresh token matching the provided one, effectively logging out the user.
///
/// # Errors
///
/// * `sqlx::Error` for database-related issues.
pub async fn logout_user(pool: &PgPool, raw_token: &str) -> Result<StatusCode, AuthError> {
    // If the token is malformed, we just ignore it and succeed silently.
    if let Ok((selector, verifier_bytes)) = split_refresh_token(raw_token)
        && let Some(rec) = sqlx::query!(
            "SELECT user_id, verifier_hash
            FROM refresh_token
            WHERE selector = $1",
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
