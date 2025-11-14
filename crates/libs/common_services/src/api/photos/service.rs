use crate::api::photos::error::PhotosError;
use crate::database::media_item::capture_details::CaptureDetails;
use crate::database::media_item::details::Details;
use crate::database::media_item::gps::Gps;
use crate::database::media_item::panorama::Panorama;
use crate::database::media_item::time_details::TimeDetails;
use crate::database::media_item::weather::Weather;
use crate::database::visual_analysis::visual_analysis::VisualAnalysis;

use crate::api::photos::interfaces::RandomPhotoResponse;
use crate::database::app_user::User;
use crate::database::media_item::media_item::{FullMediaItem, FullMediaItemRow};
use chrono::NaiveDate;
use common_types::pb::api::{
    ByMonthResponse, MediaItem, MediaMonth, TimelineMonth, TimelineResponse,
};
use rand::Rng;
use sqlx::PgPool;
use sqlx::types::Json;
use std::collections::HashMap;
use tracing::warn;

/// Fetches a full media item with all related analyses and metadata.
///
/// # Errors
///
/// Returns an error if the database query fails or the connection pool is invalid.
#[allow(clippy::too_many_lines)]
pub async fn fetch_full_media_item(
    user: &User,
    pool: &PgPool,
    id: &str,
) -> Result<Option<FullMediaItem>, sqlx::Error> {
    let row_result = sqlx::query_as!(
        FullMediaItemRow,
        r#"
        WITH
        -- 1️⃣ Collect OCR data and their boxes
        ocr_json AS (
            SELECT
                o.visual_analysis_id,
                jsonb_agg(
                    jsonb_build_object(
                        'id', o.id,
                        'has_legible_text', o.has_legible_text,
                        'ocr_text', o.ocr_text,
                        'boxes', (
                            SELECT COALESCE(
                                jsonb_agg(
                                    jsonb_build_object(
                                        'id', b.id,
                                        'text', b.text,
                                        'position_x', b.position_x,
                                        'position_y', b.position_y,
                                        'width', b.width,
                                        'height', b.height,
                                        'confidence', b.confidence
                                    )
                                ),
                                '[]'::jsonb
                            )
                            FROM ocr_box b
                            WHERE b.ocr_data_id = o.id
                        )
                    )
                ) AS data
            FROM ocr_data o
            GROUP BY o.visual_analysis_id
        ),

        -- 2️⃣ Collect all visual analyses and nested data
        visual_analyses AS (
            SELECT
                va.media_item_id,
                jsonb_agg(
                    jsonb_build_object(
                        'id', va.id,
                        'created_at', va.created_at,
                        'quality', (
                            SELECT to_jsonb(qd)
                            FROM quality_data qd WHERE qd.visual_analysis_id = va.id
                        ),
                        'colors', (
                            SELECT to_jsonb(cld)
                            FROM color_data cld WHERE cld.visual_analysis_id = va.id
                        ),
                        'caption', (
                            SELECT to_jsonb(cpd)
                            FROM caption_data cpd WHERE cpd.visual_analysis_id = va.id
                        ),
                        'faces', (
                            SELECT COALESCE(
                                jsonb_agg(to_jsonb(f)),
                                '[]'::jsonb
                            ) FROM face f WHERE f.visual_analysis_id = va.id
                        ),
                        'detected_objects', (
                            SELECT COALESCE(
                                jsonb_agg(to_jsonb(obj)),
                                '[]'::jsonb
                            ) FROM detected_object obj WHERE obj.visual_analysis_id = va.id
                        ),
                        'ocr_data', COALESCE(ocr.data, '[]'::jsonb)
                    )
                    ORDER BY va.created_at DESC
                ) AS data
            FROM visual_analysis va
            LEFT JOIN ocr_json ocr ON ocr.visual_analysis_id = va.id
            GROUP BY va.media_item_id
        )

        SELECT
            mi.id,
            mi.hash,
            mi.relative_path,
            mi.created_at,
            mi.updated_at,
            mi.width,
            mi.height,
            mi.is_video,
            mi.duration_ms,
            mi.taken_at_local,
            mi.taken_at_utc,
            mi.use_panorama_viewer,

            COALESCE(va.data, '[]'::jsonb) AS "visual_analyses: Json<Vec<VisualAnalysis>>",

            (SELECT to_jsonb(g)
                    || jsonb_build_object('location',
                        (SELECT to_jsonb(l.*) FROM location l WHERE l.id = g.location_id))
                FROM gps g WHERE g.media_item_id = mi.id
            ) AS "gps: Json<Gps>",

            (SELECT to_jsonb(td) FROM time_details td WHERE td.media_item_id = mi.id)
                AS "time_details: Json<TimeDetails>",

            (SELECT to_jsonb(w) FROM weather w WHERE w.media_item_id = mi.id)
                AS "weather: Json<Weather>",

            (SELECT to_jsonb(d) FROM details d WHERE d.media_item_id = mi.id)
                AS "details: Json<Details>",

            (SELECT to_jsonb(cd) FROM capture_details cd WHERE cd.media_item_id = mi.id)
                AS "capture_details: Json<CaptureDetails>",

            (SELECT to_jsonb(p) FROM panorama p WHERE p.media_item_id = mi.id)
                AS "panorama: Json<Panorama>"

        FROM media_item mi
        LEFT JOIN visual_analyses va ON mi.id = va.media_item_id
        WHERE mi.id = $1 AND mi.user_id = $2 AND mi.deleted = false;
        "#,
        id,
        user.id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row_result.map(FullMediaItem::from))
}

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

/// Fetches a timeline of media item ratios, grouped by month.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn get_timeline_ratios(
    user: &User,
    pool: &PgPool,
) -> Result<TimelineResponse, PhotosError> {
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
pub async fn get_timeline_ids(user: &User, pool: &PgPool) -> Result<Vec<String>, PhotosError> {
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
