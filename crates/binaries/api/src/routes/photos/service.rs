// crates/api/src/routes/photos/service.rs

use crate::auth::db_model::User;
use crate::pb::api::{ByMonthResponse, MediaItem, MediaMonth, TimelineMonth, TimelineResponse};
use crate::photos::error::PhotosError;
use crate::photos::full_item_interfaces::{
    CaptureDetails, Details, FullMediaItem, FullMediaItemRow, Gps, Panorama, TimeDetails,
    VisualAnalysis, Weather,
};
use crate::photos::interfaces::RandomPhotoResponse;
use chrono::NaiveDate;
use rand::Rng;
use sqlx::types::Json;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::warn;

pub async fn fetch_full_media_item(
    user: &User,
    pool: &PgPool,
    id: &str,
) -> Result<Option<FullMediaItem>, sqlx::Error> {
    // NOTE: This assumes you've updated your Rust structs as described above.
    // The FullMediaItemRow and From<...> impl are still used but their
    // underlying types (like Gps, Face, etc.) are now simpler.

    let row_result = sqlx::query_as!(
        FullMediaItemRow,
        r#"
WITH full_visual_analyses AS (
    SELECT
        va.media_item_id,
        jsonb_agg(
            jsonb_build_object(
                'id', va.id,
                'created_at', va.created_at,
                'quality', (SELECT jsonb_build_object('blurriness', qd.blurriness, 'noisiness', qd.noisiness, 'exposure', qd.exposure, 'quality_score', qd.quality_score) FROM quality_data qd WHERE qd.visual_analysis_id = va.id),
                'colors', (SELECT jsonb_build_object('themes', cld.themes, 'prominent_colors', cld.prominent_colors, 'average_hue', cld.average_hue, 'average_saturation', cld.average_saturation, 'average_lightness', cld.average_lightness, 'histogram', cld.histogram) FROM color_data cld WHERE cld.visual_analysis_id = va.id),
                'caption', (SELECT to_jsonb(cpd) - 'visual_analysis_id' FROM caption_data cpd WHERE cpd.visual_analysis_id = va.id),
                'faces', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', f.id, 'position_x', f.position_x, 'position_y', f.position_y, 'width', f.width, 'height', f.height, 'confidence', f.confidence, 'age', f.age, 'sex', f.sex, 'mouth_left_x', f.mouth_left_x, 'mouth_left_y', f.mouth_left_y, 'mouth_right_x', f.mouth_right_x, 'mouth_right_y', f.mouth_right_y, 'nose_tip_x', f.nose_tip_x, 'nose_tip_y', f.nose_tip_y, 'eye_left_x', f.eye_left_x, 'eye_left_y', f.eye_left_y, 'eye_right_x', f.eye_right_x, 'eye_right_y', f.eye_right_y)), '[]'::jsonb) FROM face f WHERE f.visual_analysis_id = va.id),
                'detected_objects', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', obj.id, 'position_x', obj.position_x, 'position_y', obj.position_y, 'width', obj.width, 'height', obj.height, 'confidence', obj.confidence, 'label', obj.label)), '[]'::jsonb) FROM detected_object obj WHERE obj.visual_analysis_id = va.id),
                'ocr_data', (
                    SELECT COALESCE(jsonb_agg(
                        jsonb_build_object(
                            'id', ocr.id,
                            'has_legible_text', ocr.has_legible_text,
                            'ocr_text', ocr.ocr_text,
                            'boxes', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', b.id, 'text', b.text, 'position_x', b.position_x, 'position_y', b.position_y, 'width', b.width, 'height', b.height, 'confidence', b.confidence)), '[]'::jsonb) FROM ocr_box b WHERE b.ocr_data_id = ocr.id)
                        )
                    ), '[]'::jsonb)
                    FROM ocr_data ocr WHERE ocr.visual_analysis_id = va.id
                )
            ) ORDER BY va.created_at DESC
        ) AS data
    FROM visual_analysis va
    WHERE va.media_item_id = $1
    GROUP BY va.media_item_id
)
SELECT
    mi.id, mi.hash, mi.relative_path, mi.created_at, mi.updated_at, mi.width, mi.height,
    mi.is_video, mi.duration_ms, mi.taken_at_local, mi.taken_at_utc, mi.use_panorama_viewer,

    COALESCE(fva.data, '[]'::jsonb) AS "visual_analyses: Json<Vec<VisualAnalysis>>",

    -- NOTE: A trick for removing a single key is `to_jsonb(table) - 'key_to_remove'`. It's cleaner.
    (SELECT to_jsonb(g) - 'media_item_id' || jsonb_build_object('location', (SELECT to_jsonb(l.*) FROM location l WHERE l.id = g.location_id)) FROM gps g WHERE g.media_item_id = mi.id) AS "gps: Json<Gps>",
    (SELECT to_jsonb(td) - 'media_item_id' FROM time_details td WHERE td.media_item_id = mi.id) AS "time_details: Json<TimeDetails>",
    (SELECT to_jsonb(w) - 'media_item_id' FROM weather w WHERE w.media_item_id = mi.id) AS "weather: Json<Weather>",
    (SELECT to_jsonb(d) - 'media_item_id' FROM details d WHERE d.media_item_id = mi.id) AS "details: Json<Details>",
    (SELECT to_jsonb(cd) - 'media_item_id' FROM capture_details cd WHERE cd.media_item_id = mi.id) AS "capture_details: Json<CaptureDetails>",
    (SELECT to_jsonb(p) - 'media_item_id' FROM panorama p WHERE p.media_item_id = mi.id) AS "panorama: Json<Panorama>"
FROM
    media_item mi
LEFT JOIN
    full_visual_analyses fva ON mi.id = fva.media_item_id
WHERE
    mi.id = $1 AND mi.user_id = $2 AND mi.deleted = false
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
            month_id::TEXT as "month_id!",
            COUNT(*)::INT AS "count!",
            array_agg(width::real / height::real ORDER BY taken_at_local DESC) AS "ratios!"
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
            taken_at_local DESC
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
