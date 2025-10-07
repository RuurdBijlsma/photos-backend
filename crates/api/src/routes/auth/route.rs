use crate::auth::structs::UserRole;
use crate::routes::auth::structs::{
    Claims, CreateUser, LoginUser, RefreshTokenPayload, Tokens, User, UserRecord,
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use common_photos::get_config;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{rng, RngCore};
use serde::Serialize;
use sqlx::PgPool;
//=========================================================================================
// HELPER FUNCTION: This encapsulates the logic for creating refresh token components.
//=========================================================================================

struct RefreshTokenParts {
    raw_token: String,
    selector: String,
    verifier_hash: String,
}

fn generate_refresh_token_parts() -> Result<RefreshTokenParts, (StatusCode, String)> {
    // Generate 32 random bytes for the raw token
    let mut raw_bytes = [0u8; 32];
    rng().fill_bytes(&mut raw_bytes);

    // The first 16 bytes are the selector, the last 16 are the verifier
    let selector_bytes = &raw_bytes[..16];
    let verifier_bytes = &raw_bytes[16..];

    // URL-safe base64 encoding
    let selector = URL_SAFE_NO_PAD.encode(selector_bytes);
    let raw_token = URL_SAFE_NO_PAD.encode(raw_bytes);

    // Hash the verifier part
    let verifier_hash = hash(verifier_bytes, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash token".to_string(),
        )
    })?;

    Ok(RefreshTokenParts {
        raw_token,
        selector,
        verifier_hash,
    })
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Tokens>, (StatusCode, String)> {
    // --- 1. User Verification (No change) ---
    let user = sqlx::query_as!(
        UserRecord,
        r#"
        SELECT id, email, name, password, role as "role: UserRole", created_at, updated_at
        FROM app_user
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let is_valid = verify(&payload.password, &user.password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to verify password".to_string(),
        )
    })?;

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let config = &get_config();
    // --- 2. Generate Access Token (No change) ---
    let now = Utc::now();
    let access_token_exp =
        (now + Duration::minutes(config.auth.access_token_expiry_minutes)).timestamp() as usize;
    let access_claims = Claims {
        sub: user.id,
        role: user.role.to_string(),
        exp: access_token_exp,
    };
    let secret = &config.auth.jwt_secret;
    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create access token".to_string(),
        )
    })?;

    // --- 3. Generate and Store Refresh Token using Selector-Verifier Pattern ---
    let token_parts = generate_refresh_token_parts()?;
    let refresh_token_exp = now + Duration::days(config.auth.refresh_token_expiry_days);

    sqlx::query!(
        r#"
        INSERT INTO refresh_token (user_id, selector, verifier_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        token_parts.selector,
        token_parts.verifier_hash,
        refresh_token_exp
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        eprintln!("Failed to insert refresh token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not save session".to_string(),
        )
    })?;

    // --- 4. Return Both Tokens ---
    Ok(Json(Tokens {
        access_token,
        refresh_token: token_parts.raw_token, // Send the full raw token to the client
    }))
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let hashed_password = hash(&payload.password, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    // The SQL query is updated here
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO app_user (email, name, password)
        VALUES ($1, $2, $3)
        RETURNING id, email, name, media_folder, role as "role: UserRole", created_at, updated_at
        "#,
        payload.email,
        payload.name,
        hashed_password
    )
    .fetch_one(&pool)
    .await
    //     todo: better error here, and in general
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}

struct RefreshTokenRecord {
    user_id: i32,
    verifier_hash: String,
}

pub async fn refresh_session(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<Tokens>, (StatusCode, String)> {
    // --- 1. Split the incoming token into selector and verifier ---
    let raw_token_bytes = URL_SAFE_NO_PAD
        .decode(&payload.refresh_token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token format".to_string()))?;

    if raw_token_bytes.len() != 32 {
        return Err((StatusCode::UNAUTHORIZED, "Invalid token length".to_string()));
    }

    let selector_bytes = &raw_token_bytes[..16];
    let verifier_bytes = &raw_token_bytes[16..];
    let selector = URL_SAFE_NO_PAD.encode(selector_bytes);

    // --- 2. Perform a fast, indexed lookup for the token ---
    let record = sqlx::query_as!(
        RefreshTokenRecord,
        r#"
        SELECT user_id, verifier_hash FROM refresh_token
        WHERE selector = $1 AND expires_at > NOW()
        "#,
        selector
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Token not found or expired".to_string(),
        )
    })?;

    // --- 3. Verify the verifier hash ---
    let is_valid = verify(verifier_bytes, &record.verifier_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Token verification failed".to_string(),
        )
    })?;

    if !is_valid {
        // If the verifier is invalid, it's a sign of a potential attack.
        // Invalidate all tokens for this user as a security measure.
        sqlx::query!(
            "DELETE FROM refresh_token WHERE user_id = $1",
            record.user_id
        )
        .execute(&pool)
        .await
        .ok(); // Ignore error if it fails
        return Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()));
    }

    // --- 4. Fetch user data needed for the new access token claims ---
    let user_role = sqlx::query!(
        r#"SELECT role as "role: UserRole" FROM app_user WHERE id = $1"#,
        record.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".to_string()))?
    .role;

    // --- 5. Perform Token Rotation in a Transaction ---
    let mut tx = pool.begin().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
    })?;

    // Invalidate the used token by deleting it
    sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not invalidate session".to_string(),
            )
        })?;

    // Issue a new pair of tokens
    let config = get_config();
    let new_token_parts = generate_refresh_token_parts()?;
    let new_refresh_token_exp = Utc::now() + Duration::days(config.auth.refresh_token_expiry_days);

    sqlx::query!(
        "INSERT INTO refresh_token (user_id, selector, verifier_hash, expires_at) VALUES ($1, $2, $3, $4)",
        record.user_id, new_token_parts.selector, new_token_parts.verifier_hash, new_refresh_token_exp
    ).execute(&mut *tx).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Could not save new session".to_string()))?;

    // Create the new access token
    let now = Utc::now();
    let access_token_exp =
        (now + Duration::minutes(config.auth.access_token_expiry_minutes)).timestamp() as usize;
    let access_claims = Claims {
        sub: record.user_id,
        role: user_role.to_string(),
        exp: access_token_exp,
    };
    let secret = &config.auth.jwt_secret;
    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create access token".to_string(),
        )
    })?;

    tx.commit().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not save session".to_string(),
        )
    })?;

    Ok(Json(Tokens {
        access_token,
        refresh_token: new_token_parts.raw_token,
    }))
}

pub async fn logout(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    // --- 1. Split token into selector and verifier ---
    let raw_token_bytes = match URL_SAFE_NO_PAD.decode(&payload.refresh_token) {
        Ok(bytes) if bytes.len() == 32 => bytes,
        _ => return Ok(StatusCode::NO_CONTENT), // Don't leak info, just act like it worked
    };
    let selector_bytes = &raw_token_bytes[..16];
    let verifier_bytes = &raw_token_bytes[16..];
    let selector = URL_SAFE_NO_PAD.encode(selector_bytes);

    // --- 2. Find the token by selector ---
    let record = match sqlx::query_as!(
        RefreshTokenRecord,
        "SELECT user_id, verifier_hash FROM refresh_token WHERE selector = $1",
        selector
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(rec)) => rec,
        _ => return Ok(StatusCode::NO_CONTENT), // Not found or DB error, don't leak info
    };

    // --- 3. Verify the token to ensure it belongs to the user logging out ---
    if verify(verifier_bytes, &record.verifier_hash).unwrap_or(false) {
        // Verification successful, delete the token
        sqlx::query!("DELETE FROM refresh_token WHERE selector = $1", selector)
            .execute(&pool)
            .await
            .ok(); // Ignore potential error, the goal is to revoke access
    }

    // Always return a success response to prevent token scanning attacks
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ProtectedResponse {
    message: String,
    user_email: String,
    user_id: i32,
}

pub async fn protected_route(
    Extension(user): Extension<User>,
) -> Result<Json<ProtectedResponse>, StatusCode> {
    // If this code runs, the user is authenticated.
    // We can now safely use the `user` data.
    let response = ProtectedResponse {
        message: "You are accessing a protected route!".to_string(),
        user_email: user.email,
        user_id: user.id,
    };

    Ok(Json(response))
}
