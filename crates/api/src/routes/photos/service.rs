//! This module provides the core service logic for photos endpoints.

use crate::auth::db_model::User;
use crate::photos::interfaces::RandomPhotoResponse;
use crate::setup::error::SetupError;
use rand::{Rng, rng};
use sqlx::PgPool;
use tracing::warn;

/// Fetches a random photo with its color theme data for a specific user.
///
/// # Errors
///
/// Returns an error if either of the database queries fail.
pub async fn get_random_photo(
    user: &User,
    pool: &PgPool,
) -> Result<Option<RandomPhotoResponse>, SetupError> {
    // Count the total number of photos with associated color data for the given user.
    let Some(count) = sqlx::query_scalar!(
        r#"
        SELECT COUNT(cd.visual_analysis_id)
        FROM color_data AS cd
        JOIN visual_analysis AS va ON cd.visual_analysis_id = va.id
        JOIN media_item AS mi ON va.media_item_id = mi.id
        WHERE mi.user_id = $1 AND mi.deleted = false
        "#,
        user.id
    )
    .fetch_one(pool)
    .await?
    else {
        return Ok(None);
    };

    if count == 0 {
        warn!("No photos with color data for user {}", user.id);
        return Ok(None);
    }

    let random_offset = rng().random_range(0..count);

    // Fetch a single row from `color_data` using the random offset,
    // along with the associated `media_item_id`.
    let Some(random_data) = sqlx::query_as!(
        RandomPhotoResponse,
        r#"
        SELECT
            cd.themes,
            mi.id as media_id
        FROM color_data AS cd
        JOIN visual_analysis AS va ON cd.visual_analysis_id = va.id
        JOIN media_item AS mi ON va.media_item_id = mi.id
        WHERE mi.user_id = $1 AND mi.deleted = false
        ORDER BY mi.id
        LIMIT 1
        OFFSET $2
        "#,
        user.id,
        random_offset
    )
    .fetch_optional(pool)
    .await?
    else {
        // This can happen in a race condition if photos are deleted between the COUNT and this query.
        warn!(
            "No photo found at offset {} for user {}",
            random_offset, user.id
        );
        return Ok(None);
    };

    Ok(Some(random_data))
}
