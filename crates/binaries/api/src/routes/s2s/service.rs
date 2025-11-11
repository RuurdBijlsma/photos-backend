use crate::routes::s2s::error::S2SError;
use common_photos::{media_dir, InviteSummaryResponse};
use sqlx::PgPool;
use tracing::instrument;

/// Validates an invitation token and returns the summary of the album.
#[instrument(skip(pool))]
pub async fn get_invite_summary(
    pool: &PgPool,
    token: &str,
) -> Result<InviteSummaryResponse, S2SError> {
    // The token format is inv-{secure_part}-{user@host}
    // We only need the secure part for the DB lookup.
    let token_parts: Vec<&str> = token.split('-').collect();
    if token_parts.len() < 2 || token_parts[0] != "inv" {
        return Err(S2SError::TokenInvalid);
    }
    let secure_token = token_parts[1];

    let summary = sqlx::query!(
        r#"
        SELECT
            a.name AS album_name,
            a.description AS album_description,
            (SELECT array_agg(ami.media_item_id) FROM album_media_item ami WHERE ami.album_id = a.id) as "media_item_ids!"
        FROM album_invites ai
        JOIN album a ON ai.album_id = a.id
        WHERE ai.token = $1 AND ai.expires_at > now()
        "#,
        secure_token
    )
        .fetch_optional(pool)
        .await?
        .ok_or(S2SError::TokenInvalid)?;

    Ok(InviteSummaryResponse {
        album_name: summary.album_name,
        album_description: summary.album_description,
        media_item_ids: summary.media_item_ids,
    })
}

/// A validated token and its associated album_id.
pub struct ValidatedToken {
    pub album_id: String,
}

/// Validates a token and checks that a media_item_id belongs to the token's album.
/// This is a critical security check.
#[instrument(skip(pool))]
pub async fn validate_token_for_media_item(
    pool: &PgPool,
    token: &str,
    media_item_id: &str,
) -> Result<ValidatedToken, S2SError> {
    let token_parts: Vec<&str> = token.split('-').collect();
    if token_parts.len() < 2 || token_parts[0] != "inv" {
        return Err(S2SError::TokenInvalid);
    }
    let secure_token = token_parts[1];

    let result = sqlx::query!(
        r#"
        SELECT ai.album_id
        FROM album_invites ai
        JOIN album_media_item ami ON ai.album_id = ami.album_id
        WHERE ai.token = $1
          AND ami.media_item_id = $2
          AND ai.expires_at > now()
        "#,
        secure_token,
        media_item_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(S2SError::PermissionDenied)?;

    Ok(ValidatedToken {
        album_id: result.album_id,
    })
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
