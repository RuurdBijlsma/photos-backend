use axum::http::StatusCode;
use axum::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use common_photos::get_config;
use crate::auth::model::*;
use crate::auth::token::*;

pub async fn authenticate_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<UserRecord, (StatusCode, String)> {
    let user = sqlx::query_as!(
        UserRecord,
        r#"SELECT id, email, name, password, role as "role: UserRole", created_at, updated_at
           FROM app_user WHERE email = $1"#,
        email
    )
        .fetch_optional(pool)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    let valid = verify(password, &user.password).map_err(internal_err)?;
    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    Ok(user)
}

pub async fn create_user(
    pool: &PgPool,
    payload: &CreateUser,
) -> Result<User, (StatusCode, String)> {
    let hashed = hash(&payload.password, DEFAULT_COST).map_err(internal_err)?;
    sqlx::query_as!(
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
        .await
        .map_err(internal_err)
}

pub async fn store_refresh_token(
    pool: &PgPool,
    user_id: i32,
    parts: &RefreshTokenParts,
) -> Result<(), (StatusCode, String)> {
    let exp = Utc::now() + Duration::days(get_config().auth.refresh_token_expiry_days);
    sqlx::query!(
        "INSERT INTO refresh_token (user_id, selector, verifier_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
        user_id,
        parts.selector,
        parts.verifier_hash,
        exp
    )
        .execute(pool)
        .await
        .map_err(internal_err)?;
    Ok(())
}

pub fn create_access_token(user_id: i32, role: UserRole) -> Result<String, (StatusCode, String)> {
    let cfg = get_config();
    let exp = (Utc::now() + Duration::minutes(cfg.auth.access_token_expiry_minutes)).timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        role,
        exp,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(cfg.auth.jwt_secret.as_ref()))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to sign token".into()))
}

/// Handles refresh token rotation
pub async fn refresh_tokens(
    pool: &PgPool,
    raw_token: &str,
) -> Result<Json<Tokens>, (StatusCode, String)> {
    let (selector, verifier_bytes) = split_refresh_token(raw_token)?;
    let record = sqlx::query!(
        "SELECT user_id, verifier_hash FROM refresh_token
         WHERE selector = $1 AND expires_at > NOW()",
        selector
    )
        .fetch_optional(pool)
        .await
        .map_err(internal_err)?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token not found or expired".into()))?;

    let valid = verify_token(&verifier_bytes, &record.verifier_hash)?;
    if !valid {
        sqlx::query!("DELETE FROM refresh_token WHERE user_id = $1", record.user_id)
            .execute(pool)
            .await
            .ok();
        return Err((StatusCode::UNAUTHORIZED, "Invalid token".into()));
    }

    let user_role = sqlx::query!(
        r#"SELECT role as "role: UserRole" FROM app_user WHERE id = $1"#,
        record.user_id
    )
        .fetch_one(pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".into()))?
        .role;

    let mut tx = pool.begin().await.map_err(internal_err)?;
    sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;

    let new_parts = generate_refresh_token_parts()?;
    let exp = Utc::now() + Duration::days(get_config().auth.refresh_token_expiry_days);
    sqlx::query!(
        "INSERT INTO refresh_token (user_id, selector, verifier_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
        record.user_id,
        new_parts.selector,
        new_parts.verifier_hash,
        exp
    )
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    let access_token = create_access_token(record.user_id, user_role)?;
    Ok(Json(Tokens {
        access_token,
        refresh_token: new_parts.raw_token,
    }))
}

/// Deletes the refresh token matching the provided one
pub async fn logout_user(
    pool: &PgPool,
    raw_token: &str,
) -> Result<StatusCode, (StatusCode, String)> {
    let (selector, verifier_bytes) = match split_refresh_token(raw_token) {
        Ok(v) => v,
        Err(_) => return Ok(StatusCode::NO_CONTENT),
    };

    let record = sqlx::query!(
        "SELECT user_id, verifier_hash FROM refresh_token WHERE selector = $1",
        selector
    )
        .fetch_optional(pool)
        .await
        .map_err(internal_err)?;

    if let Some(rec) = record {
        if verify_token(&verifier_bytes, &rec.verifier_hash).unwrap_or(false) {
            sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
                .execute(pool)
                .await
                .ok();
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
