// crates/api/src/routes/photos/service.rs

use crate::auth::db_model::User;
use crate::pb::api::{ByMonthResponse, MediaItem, MediaMonth, TimelineMonth, TimelineResponse};
use crate::photos::error::PhotosError;
use crate::photos::interfaces::RandomPhotoResponse;
use rand::Rng;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::warn;

/// Fetches a random photo with its color theme data for a specific user.
///
/// # Errors
///
/// Returns an error if either of the database queries fail.
pub async fn random_photo(
    user: &User,
    pool: &PgPool,
) -> Result<Option<RandomPhotoResponse>, PhotosError> {
    // Count the total number of photos with associated color data for the given user.
    let count: i64 = sqlx::query_scalar!(
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
        .unwrap_or(0); // Default to 0 if count is NULL

    if count == 0 {
        warn!("No photos with color data for user {}", user.id);
        return Ok(None);
    }

    // Use a thread-safe random number generator to select a random offset.
    let random_offset = rand::rng().random_range(0..count);

    // Fetch a single row from `color_data` using the random offset,
    // along with the associated `media_item_id`.
    let random_data = sqlx::query_as!(
        RandomPhotoResponse,
        r#"
        SELECT
            cd.themes,
            mi.id as media_id
        FROM color_data AS cd
        JOIN visual_analysis AS va ON cd.visual_analysis_id = va.id
        JOIN media_item AS mi ON va.media_item_id = mi.id
        WHERE mi.user_id = $1 AND mi.deleted = false
        ORDER BY mi.id -- Consistent ordering is important for OFFSET
        LIMIT 1
        OFFSET $2
        "#,
        user.id,
        random_offset
    )
        .fetch_optional(pool)
        .await?;

    if random_data.is_none() {
        // This can happen in a race condition if photos are deleted between the COUNT and this query.
        warn!(
            "No photo found at offset {} for user {}",
            random_offset, user.id
        );
    }

    Ok(random_data)
}

/// Fetches a timeline of media items, grouped by month.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_timeline(user: &User, pool: &PgPool) -> Result<TimelineResponse, PhotosError> {
    let months = sqlx::query_as!(
        TimelineMonth,
        r#"
        SELECT
            TO_CHAR(taken_at_local, 'YYYY-MM') as "month_id!",
            COUNT(*)::INT AS "count!",
            array_agg(width::real / height::real) AS "ratios!"
        FROM
            media_item
        WHERE
            user_id = $1 AND deleted = false
        GROUP BY
            TO_CHAR(taken_at_local, 'YYYY-MM')
        ORDER BY
            "month_id!" DESC
        "#,
        user.id
    )
        .fetch_all(pool)
        .await?;

    Ok(TimelineResponse { months })
}

/// Fetches media items for a given list of month IDs, grouped by month.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_photos_by_month(
    user: &User,
    pool: &PgPool,
    month_ids: &[String],
) -> Result<ByMonthResponse, PhotosError> {
    let items = sqlx::query_as!(
        MediaItem,
        r#"
        SELECT
            id as "id!",
            is_video as "is_video!",
            use_panorama_viewer as "is_panorama!",
            duration_ms::INT,
            taken_at_local::TEXT as "timestamp!"
        FROM
            media_item
        WHERE
            user_id = $1
            AND deleted = false
            AND TO_CHAR(taken_at_local, 'YYYY-MM') = ANY($2)
        ORDER BY
            taken_at_local DESC
        "#,
        user.id,
        month_ids,
    )
        .fetch_all(pool)
        .await?;

    let mut months_map: HashMap<String, Vec<MediaItem>> = HashMap::new();
    for item in items {
        let month_id = item.timestamp[0..7].to_string();
        months_map.entry(month_id).or_default().push(item);
    }

    let months = months_map
        .into_iter()
        .map(|(month_id, items)| MediaMonth { month_id, items })
        .collect();

    Ok(ByMonthResponse { months })
}