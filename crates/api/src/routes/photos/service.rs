//! This module provides the core service logic for photos endpoints.

use crate::auth::db_model::User;
use crate::photos::interfaces::RandomPhotoResponse;
use crate::setup::error::SetupError;
use rand::{rng, Rng};
use sqlx::PgPool;
use tracing::warn;

pub async fn get_random_photo(
    user: &User,
    pool: &PgPool,
) -> Result<Option<RandomPhotoResponse>, SetupError> {
    let Some(count) = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM media_item WHERE user_id = $1 AND deleted = false",
        user.id
    )
    .fetch_one(pool)
    .await?
    else {
        return Ok(None);
    };
    if count == 0 {
        warn!("No photos for user {}", user.id);
        return Ok(None);
    }
    let random_offset = rng().random_range(0..count);
    let Some(random_id) = sqlx::query_scalar!(
        "SELECT id FROM media_item WHERE user_id = $1 AND deleted = false ORDER BY created_at LIMIT 1 OFFSET $2",
        user.id,
        random_offset
    )
        .fetch_optional(pool)
        .await? else {
        warn!("No photo found at offset {} for user {}", random_offset, user.id);
        return Ok(None);
    };
    let themes = sqlx::query_scalar!(
        r"
        SELECT cd.themes
        FROM color_data AS cd
        JOIN visual_analysis AS va ON cd.visual_analysis_id = va.id
        WHERE va.media_item_id = $1
        LIMIT 1
    ",
        random_id
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    Ok(Some(RandomPhotoResponse {
        themes,
        media_id: random_id,
    }))
}
