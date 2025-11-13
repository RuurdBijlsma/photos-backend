use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::PgPool;
use tracing::instrument;
use common_types::album::AlbumSummary;
use crate::album::interfaces::AlbumShareClaims;
use crate::s2s::error::S2SError;
use crate::settings::{media_dir, settings};


fn extract_token_claims(token: &str) -> Result<AlbumShareClaims, S2SError> {
    decode::<AlbumShareClaims>(
        token,
        &DecodingKey::from_secret(settings().auth.jwt_secret.as_ref()),
        &Validation::default(),
    )
        .map(|p| p.claims)
        .map_err(|_| S2SError::Unauthorized("Invalid token.".to_string()))
}

/// Validates an invitation token and returns the summary of the album.
#[instrument(skip(pool))]
pub async fn get_invite_summary(
    pool: &PgPool,
    token: &str,
) -> Result<AlbumSummary, S2SError> {
    let claims = extract_token_claims(token)?;

    let summary = sqlx::query_as!(AlbumSummary,
        r#"
        SELECT
            name,
            description,
            (SELECT array_agg(ami.media_item_id) FROM album_media_item ami WHERE ami.album_id = $1) as "media_item_ids!"
        FROM album 
        WHERE album.id = $1
        "#,
        claims.sub
    )
        .fetch_optional(pool)
        .await?
        .ok_or(S2SError::TokenInvalid)?;
    Ok(summary)
}

/// Validates a token and checks that a `media_item_id` belongs to the token's album.
/// This is a critical security check.
#[instrument(skip(pool))]
pub async fn validate_token_for_media_item(
    pool: &PgPool,
    token: &str,
    media_item_id: &str,
) -> Result<String, S2SError> {
    let claims = extract_token_claims(token)?;

    let album_id = sqlx::query_scalar!(
        r#"
        SELECT album_id
        FROM album_media_item ami WHERE album_id = $1
        AND media_item_id = $2
        "#,
        claims.sub,
        media_item_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(S2SError::PermissionDenied)?;

    Ok(album_id)
}

/// Retrieves the relative path for a given media item owned by a user.
#[instrument(skip(pool))]
pub async fn get_media_item_path(
    pool: &PgPool,
    media_item_id: &str,
) -> Result<std::path::PathBuf, S2SError> {
    let relative_path = sqlx::query_scalar!(
        r#"
        SELECT mi.relative_path
        FROM media_item mi
        JOIN app_user u ON mi.user_id = u.id
        WHERE mi.id = $1
        "#,
        media_item_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| S2SError::NotFound(format!("Media item {media_item_id} not found.")))?;

    Ok(media_dir().join(relative_path))
}
