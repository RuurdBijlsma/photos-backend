// crates/api/src/routes/photos/service.rs

use crate::auth::db_model::User;
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    DayGroup, GetMediaByMonthParams, MediaItemDto, MonthGroup, PaginatedMediaResponse,
    RandomPhotoResponse, TimelineSummary,
};
use rand::Rng;
use sqlx::PgPool;
use tracing::warn;

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

/// Fetches the timeline summary for a given user.
pub async fn get_timeline_summary(
    user: &User,
    pool: &PgPool,
) -> Result<Vec<TimelineSummary>, PhotosError> {
    let summary = sqlx::query_as!(
        TimelineSummary,
        r#"
        SELECT
            year as "year!",
            month as "month!",
            media_count as "media_count!"
        FROM timeline_summary
        WHERE user_id = $1
        ORDER BY year DESC, month DESC
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(summary)
}

/// Fetches media items for the given months and groups them by month, then by day.
pub async fn get_media_by_months(
    params: &GetMediaByMonthParams,
    user: &User,
    pool: &PgPool,
) -> Result<PaginatedMediaResponse, PhotosError> {
    let month_tuples: Vec<(i32, i32)> = params
        .months
        .split(',')
        .filter_map(|s| {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2 {
                let year = parts[0].parse::<i32>().ok();
                let month = parts[1].parse::<i32>().ok();
                year.and_then(|y| month.map(|m| (y, m)))
            } else {
                None
            }
        })
        .collect();

    if month_tuples.is_empty() {
        return Ok(PaginatedMediaResponse { months: vec![] });
    }

    let media_items = sqlx::query_as!(
        MediaItemDto,
        r#"
        SELECT id, width, height, is_video, taken_at_local, duration_ms, use_panorama_viewer
        FROM media_item
        WHERE user_id = $1 AND deleted = false AND
              (EXTRACT(YEAR FROM taken_at_local), EXTRACT(MONTH FROM taken_at_local)) IN
              (SELECT * FROM UNNEST($2::integer[], $3::integer[]))
        ORDER BY taken_at_local DESC
        "#,
        user.id,
        &month_tuples.iter().map(|(y, _)| *y).collect::<Vec<i32>>(),
        &month_tuples.iter().map(|(_, m)| *m).collect::<Vec<i32>>(),
    )
    .fetch_all(pool)
    .await?;

    let months = group_media_by_month_and_day(media_items);

    Ok(PaginatedMediaResponse { months })
}

/// Groups a flat, sorted list of `MediaItemDto`s into a `Vec<MonthGroup>`.
fn group_media_by_month_and_day(media_items: Vec<MediaItemDto>) -> Vec<MonthGroup> {
    let mut month_groups: Vec<MonthGroup> = Vec::new();

    for item in media_items {
        let item_month = item.taken_at_local.format("%Y-%m").to_string();
        let item_date = item.taken_at_local.date().to_string();

        match month_groups.last_mut() {
            // Check if the item belongs to the most recent month group
            Some(last_month) if last_month.month == item_month => {
                match last_month.days.last_mut() {
                    // Check if the item belongs to the most recent day group in that month
                    Some(last_day) if last_day.date == item_date => {
                        last_day.media_items.push(item);
                    }
                    // Otherwise, create a new day group in the current month
                    _ => {
                        let new_day_group = DayGroup {
                            date: item_date,
                            media_items: vec![item],
                        };
                        last_month.days.push(new_day_group);
                    }
                }
            }
            // Otherwise, create a new month group (which also contains a new day group)
            _ => {
                let new_month_group = MonthGroup {
                    month: item_month,
                    days: vec![DayGroup {
                        date: item_date,
                        media_items: vec![item],
                    }],
                };
                month_groups.push(new_month_group);
            }
        }
    }
    month_groups
}
