// crates/api/src/routes/photos/service.rs

use crate::auth::db_model::User;
use crate::pb::api::{MediaItem, MonthGroup};
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    GetMediaByMonthParams, MediaItemDto, MonthGroupDto, MonthlyRatiosDto, PaginatedMediaResponse,
    RandomPhotoResponse, TimelineSummary,
};
use chrono::{Datelike, NaiveDate, NaiveTime};
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
        SELECT id,
               is_video::int as "is_video!",
               taken_at_local,
               duration_ms,
               use_panorama_viewer::int as "use_panorama_viewer!"
        FROM media_item
        WHERE user_id = $1
          AND deleted = false
          AND (EXTRACT(YEAR FROM taken_at_local), EXTRACT(MONTH FROM taken_at_local)) IN
              (SELECT * FROM UNNEST($2::integer[], $3::integer[]))
        ORDER BY taken_at_local DESC
        "#,
        user.id,
        &month_tuples.iter().map(|(y, _)| *y).collect::<Vec<i32>>(),
        &month_tuples.iter().map(|(_, m)| *m).collect::<Vec<i32>>(),
    )
    .fetch_all(pool)
    .await?;

    let months = group_media_by_month(media_items);

    Ok(PaginatedMediaResponse { months })
}

pub async fn get_media_by_month(
    month_str: &str, // e.g., "2024-10"
    user: &User,
    pool: &PgPool,
) -> Result<MonthGroup, PhotosError> {
    // 1. Parse the year and month from the input string
    let parts: Vec<&str> = month_str.split('-').collect();
    if parts.len() != 2 {
        return Err(PhotosError::InvalidMonthFormat(month_str.to_string()));
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| PhotosError::InvalidMonthFormat(month_str.to_string()))?;
    let month: i32 = parts[1]
        .parse()
        .map_err(|_| PhotosError::InvalidMonthFormat(month_str.to_string()))?;

    // 2. Query the database, mapping directly to the generated MediaItem struct
    let media_items_proto: Vec<MediaItem> = sqlx::query_as!(
        MediaItem,
        r#"
        SELECT
            id AS "i!",
            is_video::INT AS "v!",
            duration_ms AS d,
            use_panorama_viewer::INT AS "p!",
            taken_at_local::TEXT AS "t!"
        FROM
            media_item
        WHERE
            user_id = $1
            AND EXTRACT(YEAR FROM taken_at_local)::INT = $2
            AND EXTRACT(MONTH FROM taken_at_local)::INT = $3
            AND deleted = false
        ORDER BY
            taken_at_local
        "#,
        user.id,
        year,
        month
    )
        .fetch_all(pool)
        .await?;

    // 3. Construct the final MonthGroup (no intermediate mapping step!)
    let month_group = MonthGroup {
        month: month_str.to_string(),
        media_items: media_items_proto,
    };

    Ok(month_group)
}

/// Groups a flat, sorted list of `MediaItemDto`s into a `Vec<MonthGroup>`.
fn group_media_by_month(media_items: Vec<MediaItemDto>) -> Vec<MonthGroupDto> {
    let mut month_groups: Vec<MonthGroupDto> = Vec::new();
    let mut current_month_group: Option<MonthGroupDto> = None;

    for item in media_items {
        let item_month = item.taken_at_local.format("%Y-%m").to_string();
        if let Some(current) = &mut current_month_group
            && current.month == item_month
        {
            // Same month as previous media item, push item to month struct.
            current.media_items.push(item);
        } else {
            // Different month, or first month
            if let Some(current) = current_month_group {
                // Different month, push previous month to month_groups
                month_groups.push(current);
            }
            current_month_group = Some(MonthGroupDto {
                month: item_month,
                media_items: vec![item],
            });
        }
    }

    if let Some(current) = current_month_group {
        // Push last month to month_groups
        month_groups.push(current);
    }

    month_groups
}

pub async fn get_all_photo_ratios1(
    user: &User,
    pool: &PgPool,
) -> Result<Vec<Vec<f32>>, PhotosError> {
    // This query now reads pre-aggregated data from the summary table.
    // It's much faster as it avoids on-the-fly calculations and grouping.
    let ratios_by_month = sqlx::query_scalar!(
        r#"
        SELECT
            ratios as "ratios!"
        FROM
            monthly_photo_ratios
        WHERE
            user_id = $1
        ORDER BY
            month_start DESC
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(ratios_by_month)
}

pub async fn get_all_photo_ratios2(
    user: &User,
    pool: &PgPool,
) -> Result<Vec<MonthlyRatiosDto>, PhotosError> {
    let results = sqlx::query_as!(
        MonthlyRatiosDto,
        r#"
        SELECT
            TO_CHAR(DATE_TRUNC('month', taken_at_local), 'YYYY-MM')               as "month!",
            array_agg((width::float / height)::real ORDER BY taken_at_local DESC) as "ratios!"
        FROM media_item
        WHERE user_id = $1
          AND deleted = false
        GROUP BY
            DATE_TRUNC('month', taken_at_local)
        ORDER BY
            DATE_TRUNC('month', taken_at_local) DESC
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}
