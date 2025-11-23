use crate::api::timeline::error::TimelineError;
use crate::database::app_user::User;
use chrono::NaiveDate;
use common_types::pb::api::{
    ByMonthResponse, MediaItem, MediaMonth, TimelineMonth, TimelineResponse,
};
use sqlx::PgPool;
use std::collections::HashMap;

/// Fetches a timeline of media item ratios, grouped by month.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_timeline_ratios(
    user: &User,
    pool: &PgPool,
) -> Result<TimelineResponse, TimelineError> {
    let months = sqlx::query_as!(
        TimelineMonth,
        r#"
        SELECT
            month_id::TEXT as "month_id!",
            COUNT(*)::INT AS "count!",
            array_agg(width::real / height::real ORDER BY sort_timestamp DESC) AS "ratios!"
        FROM media_item
        WHERE user_id = $1
          AND deleted = false
        GROUP BY month_id
        ORDER BY month_id DESC
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(TimelineResponse { months })
}

/// Fetches a timeline of media item ids.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_timeline_ids(user: &User, pool: &PgPool) -> Result<Vec<String>, TimelineError> {
    let months = sqlx::query_scalar!(
        r#"
        SELECT id 
        FROM media_item 
        WHERE user_id = $1 AND deleted IS NOT TRUE 
        ORDER BY sort_timestamp DESC
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(months)
}

/// Fetches media items for a given list of month IDs, grouped by month.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_photos_by_month(
    user: &User,
    pool: &PgPool,
    month_ids: &[NaiveDate],
) -> Result<ByMonthResponse, TimelineError> {
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
            AND month_id = ANY($2)
        ORDER BY
            sort_timestamp DESC
        "#,
        user.id,
        month_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut months_map: HashMap<String, Vec<MediaItem>> = HashMap::new();
    for item in items {
        let month_id = format!("{}-01", &item.timestamp[0..7]);
        months_map.entry(month_id).or_default().push(item);
    }

    let months = months_map
        .into_iter()
        .map(|(month_id, items)| MediaMonth { month_id, items })
        .collect();

    Ok(ByMonthResponse { months })
}
