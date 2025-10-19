// crates/api/src/routes/photos/service.rs

use crate::auth::db_model::User;
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    DayGroup, GetMediaByDateParams, GetMediaParams, MediaItemDto, PaginatedMediaResponse,
    RandomPhotoResponse,
};
use rand::Rng;
use sqlx::PgPool;
use tracing::{debug, warn};

// --- Constants for Pagination ---
const DEFAULT_PAGINATION_LIMIT: u32 = 100;
const DEFAULT_DATE_JUMP_LIMIT: u32 = 50;

// --- Existing Function (modified for clarity) ---
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

    // Use a thread-safe random number generator
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

// --- New Service Functions for Media Grid ---

/// Groups a flat, sorted list of `MediaItemDto`s into a `Vec<DayGroup>`.
fn group_media_by_day(media_items: Vec<MediaItemDto>) -> Vec<DayGroup> {
    let mut day_groups: Vec<DayGroup> = Vec::new();

    for item in media_items {
        let item_date = item.taken_at_naive.date();
        match day_groups.last_mut() {
            Some(last_group) if last_group.date == item_date.to_string() => {
                last_group.media_items.push(item);
            }
            _ => {
                day_groups.push(DayGroup {
                    date: item_date.to_string(),
                    media_items: vec![item],
                });
            }
        }
    }
    day_groups
}

/// Fetches a paginated list of media items for a user, based on a time cursor.
pub async fn media_paginated(
    user: &User,
    pool: &PgPool,
    params: GetMediaParams,
) -> Result<PaginatedMediaResponse, PhotosError> {
    let limit = params.limit.unwrap_or(DEFAULT_PAGINATION_LIMIT);
    // "Fetch one more" pattern to determine if there are more pages.
    let query_limit = i64::from(limit) + 1;

    let mut has_more_after = false;
    let mut has_more_before = false;

    // Determine query based on `before` (scrolling down) or `after` (scrolling up)
    let mut media_items: Vec<MediaItemDto> = if let Some(before_ts) = params.before {
        let naive_dt = before_ts.naive_utc();
        // Scrolling down (fetching older photos)
        has_more_before = true; // We know there are newer photos
        sqlx::query_as!(
            MediaItemDto,
            r#"
            SELECT id, width, height, is_video, taken_at_naive
            FROM media_item
            WHERE user_id = $1 AND taken_at_naive < $2 AND deleted = FALSE
            ORDER BY taken_at_naive DESC
            LIMIT $3
            "#,
            user.id,
            naive_dt,
            query_limit
        )
        .fetch_all(pool)
        .await?
    } else if let Some(after_ts) = params.after {
        // Scrolling up (fetching newer photos)
        has_more_after = true; // We know there are older photos
        sqlx::query_as!(
            MediaItemDto,
            r#"
            SELECT id, width, height, is_video, taken_at_naive
            FROM media_item
            WHERE user_id = $1 AND taken_at_naive > $2 AND deleted = FALSE
            ORDER BY taken_at_naive
            LIMIT $3
            "#,
            user.id,
            after_ts.naive_utc(),
            query_limit
        )
        .fetch_all(pool)
        .await?
    } else {
        // Initial load (fetching the most recent photos)
        sqlx::query_as!(
            MediaItemDto,
            r#"
            SELECT id, width, height, is_video, taken_at_naive
            FROM media_item
            WHERE user_id = $1 AND deleted = FALSE
            ORDER BY taken_at_naive DESC
            LIMIT $2
            "#,
            user.id,
            query_limit
        )
        .fetch_all(pool)
        .await?
    };

    // Check if we fetched an extra item and adjust flags accordingly
    if params.after.is_some() {
        if media_items.len() > limit as usize {
            has_more_before = true;
            media_items.pop(); // Remove the extra item
        } else {
            has_more_before = false;
        }
        // Reverse to return in descending chronological order, which is more natural for the frontend
        media_items.reverse();
    } else if media_items.len() > limit as usize {
        has_more_after = true;
        media_items.pop(); // Remove the extra item
    } else {
        has_more_after = false;
    }

    Ok(PaginatedMediaResponse {
        days: group_media_by_day(media_items),
        has_more_after,
        has_more_before,
    })
}

/// Fetches a "window" of media items centered around a specific date.
pub async fn media_by_date(
    user: &User,
    pool: &PgPool,
    params: GetMediaByDateParams,
) -> Result<PaginatedMediaResponse, PhotosError> {
    let before_limit = i64::from(params.before_limit.unwrap_or(DEFAULT_DATE_JUMP_LIMIT));
    let after_limit = i64::from(params.after_limit.unwrap_or(DEFAULT_DATE_JUMP_LIMIT));
    let target_date = params.date.and_hms_opt(0, 0, 0).unwrap(); // Start of the target day

    debug!(
        "Fetching media by date: {} for user {}",
        params.date, user.id
    );

    // This robust UNION ALL query fetches items before and after the target date in a single go.
    // The outer SELECT ensures the final result set is correctly ordered.
    let media_items = sqlx::query_as!(
        MediaItemDto,
        r#"
        -- Assert that each column is NOT NULL using the '!' syntax because sqlx is dumb.
        SELECT
            id as "id!",
            width as "width!",
            height as "height!",
            is_video as "is_video!",
            taken_at_naive as "taken_at_naive!"
        FROM (
            -- Subquery to get photos BEFORE the target date
            (
                SELECT id, width, height, is_video, taken_at_naive
                FROM media_item
                WHERE user_id = $1 AND taken_at_naive < $2 AND deleted = FALSE
                ORDER BY taken_at_naive DESC
                LIMIT $3
            )
            UNION ALL
            -- Subquery to get photos ON or AFTER the target date
            (
                SELECT id, width, height, is_video, taken_at_naive
                FROM media_item
                WHERE user_id = $1 AND taken_at_naive >= $2 AND deleted = FALSE
                ORDER BY taken_at_naive
                LIMIT $4
            )
        ) AS media_window
        ORDER BY taken_at_naive;
        "#,
        user.id,
        target_date,
        before_limit,
        after_limit
    )
    .fetch_all(pool)
    .await?;

    // A simple check to see if there are more photos outside our window.
    // This isn't perfectly accurate but is a good, performant heuristic.
    let has_more_before = media_items
        .first()
        .is_some_and(|item| item.taken_at_naive < target_date);
    let has_more_after = media_items
        .last()
        .is_some_and(|item| item.taken_at_naive >= target_date);

    Ok(PaginatedMediaResponse {
        days: group_media_by_day(media_items),
        has_more_after,
        has_more_before,
    })
}
