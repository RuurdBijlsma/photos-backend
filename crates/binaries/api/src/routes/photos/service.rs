// crates/api/src/routes/photos/service.rs

use sqlx::types::Json;
use crate::auth::db_model::User;
use crate::pb::api::{ByMonthResponse, MediaItem, MediaMonth, TimelineMonth, TimelineResponse};
use crate::photos::error::PhotosError;
use crate::photos::full_item_interfaces::{CaptureDetails, Details, FullMediaItem, FullMediaItemRow, Gps, Panorama, TimeDetails, VisualAnalysis, Weather};
use crate::photos::interfaces::RandomPhotoResponse;
use chrono::NaiveDate;
use rand::Rng;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::warn;

pub async fn fetch_full_media_item(
    user: &User,
    pool: &PgPool,
    id: &str,
) -> Result<Option<FullMediaItem>, sqlx::Error> {
    // NOTE: Your supporting structs (FullMediaItemRow, From impl) are correct and do not need to change.

    let row_result = sqlx::query_as!(
        FullMediaItemRow,
        r#"
WITH full_visual_analyses AS (
    -- This CTE is correct from the previous step --
    SELECT
        va.media_item_id,
        jsonb_agg(
            jsonb_build_object(
                'id', va.id, 'mediaItemId', va.media_item_id, 'createdAt', va.created_at,
                'embedding', to_jsonb(va.embedding::REAL[]),
                'quality', (SELECT jsonb_build_object('visualAnalysisId', qd.visual_analysis_id, 'blurriness', qd.blurriness, 'noisiness', qd.noisiness, 'exposure', qd.exposure, 'qualityScore', qd.quality_score) FROM quality_data qd WHERE qd.visual_analysis_id = va.id),
                'colors', (SELECT jsonb_build_object('visualAnalysisId', cld.visual_analysis_id, 'themes', cld.themes, 'prominentColors', cld.prominent_colors, 'averageHue', cld.average_hue, 'averageSaturation', cld.average_saturation, 'averageLightness', cld.average_lightness, 'histogram', cld.histogram) FROM color_data cld WHERE cld.visual_analysis_id = va.id),
                'caption', (SELECT jsonb_build_object('visualAnalysisId', cpd.visual_analysis_id, 'defaultCaption', cpd.default_caption, 'mainSubject', cpd.main_subject, 'containsPets', cpd.contains_pets, 'containsVehicle', cpd.contains_vehicle, 'containsLandmarks', cpd.contains_landmarks, 'containsPeople', cpd.contains_people, 'containsAnimals', cpd.contains_animals, 'isIndoor', cpd.is_indoor, 'isFoodOrDrink', cpd.is_food_or_drink, 'isEvent', cpd.is_event, 'isDocument', cpd.is_document, 'isLandscape', cpd.is_landscape, 'isCityscape', cpd.is_cityscape, 'isActivity', cpd.is_activity, 'setting', cpd.setting, 'petType', cpd.pet_type, 'animalType', cpd.animal_type, 'foodOrDrinkType', cpd.food_or_drink_type, 'vehicleType', cpd.vehicle_type, 'eventType', cpd.event_type, 'landmarkName', cpd.landmark_name, 'documentType', cpd.document_type, 'peopleCount', cpd.people_count, 'peopleMood', cpd.people_mood, 'photoType', cpd.photo_type, 'activityDescription', cpd.activity_description) FROM caption_data cpd WHERE cpd.visual_analysis_id = va.id),
                'faces', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', f.id, 'visualAnalysisId', f.visual_analysis_id, 'positionX', f.position_x, 'positionY', f.position_y, 'width', f.width, 'height', f.height, 'confidence', f.confidence, 'age', f.age, 'sex', f.sex, 'mouthLeftX', f.mouth_left_x, 'mouthLeftY', f.mouth_left_y, 'mouthRightX', f.mouth_right_x, 'mouthRightY', f.mouth_right_y, 'noseTipX', f.nose_tip_x, 'noseTipY', f.nose_tip_y, 'eyeLeftX', f.eye_left_x, 'eyeLeftY', f.eye_left_y, 'eyeRightX', f.eye_right_x, 'eyeRightY', f.eye_right_y, 'embedding', to_jsonb(f.embedding::REAL[]))), '[]'::jsonb) FROM face f WHERE f.visual_analysis_id = va.id),
                'detectedObjects', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', obj.id, 'visualAnalysisId', obj.visual_analysis_id, 'positionX', obj.position_x, 'positionY', obj.position_y, 'width', obj.width, 'height', obj.height, 'confidence', obj.confidence, 'label', obj.label)), '[]'::jsonb) FROM detected_object obj WHERE obj.visual_analysis_id = va.id),
                'ocrData', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', ocr.id, 'visualAnalysisId', ocr.visual_analysis_id, 'hasLegibleText', ocr.has_legible_text, 'ocrText', ocr.ocr_text, 'boxes', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', b.id, 'ocrDataId', b.ocr_data_id, 'text', b.text, 'positionX', b.position_x, 'positionY', b.position_y, 'width', b.width, 'height', b.height, 'confidence', b.confidence)), '[]'::jsonb) FROM ocr_box b WHERE b.ocr_data_id = ocr.id))), '[]'::jsonb) FROM ocr_data ocr WHERE ocr.visual_analysis_id = va.id)
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

    -- ================= FIX STARTS HERE: All remaining subqueries are now explicit =================
    (SELECT jsonb_build_object(
        'mediaItemId', g.media_item_id, 'latitude', g.latitude, 'longitude', g.longitude,
        'altitude', g.altitude, 'imageDirection', g.image_direction,
        'location', (SELECT jsonb_build_object('id', l.id, 'name', l.name, 'admin1', l.admin1, 'admin2', l.admin2, 'countryCode', l.country_code, 'countryName', l.country_name) FROM location l WHERE l.id = g.location_id)
    ) FROM gps g WHERE g.media_item_id = mi.id) AS "gps: Json<Gps>",

    (SELECT jsonb_build_object(
        'mediaItemId', td.media_item_id, 'timezoneName', td.timezone_name, 'timezoneOffsetSeconds', td.timezone_offset_seconds,
        'source', td.source, 'sourceDetails', td.source_details, 'sourceConfidence', td.source_confidence
    ) FROM time_details td WHERE td.media_item_id = mi.id) AS "time_details: Json<TimeDetails>",

    (SELECT jsonb_build_object(
        'mediaItemId', w.media_item_id, 'temperature', w.temperature, 'dewPoint', w.dew_point, 'relativeHumidity', w.relative_humidity,
        'precipitation', w.precipitation, 'snow', w.snow, 'windDirection', w.wind_direction, 'windSpeed', w.wind_speed, 'peakWindGust', w.peak_wind_gust,
        'pressure', w.pressure, 'sunshineMinutes', w.sunshine_minutes, 'condition', w.condition, 'sunrise', w.sunrise, 'sunset', w.sunset,
        'dawn', w.dawn, 'dusk', w.dusk, 'isDaytime', w.is_daytime
    ) FROM weather w WHERE w.media_item_id = mi.id) AS "weather: Json<Weather>",

    (SELECT jsonb_build_object(
        'mediaItemId', d.media_item_id, 'mimeType', d.mime_type, 'sizeBytes', d.size_bytes, 'isMotionPhoto', d.is_motion_photo,
        'motionPhotoPresentationTimestamp', d.motion_photo_presentation_timestamp, 'isHdr', d.is_hdr, 'isBurst', d.is_burst,
        'burstId', d.burst_id, 'captureFps', d.capture_fps, 'videoFps', d.video_fps, 'isNightsight', d.is_nightsight,
        'isTimelapse', d.is_timelapse, 'exif', d.exif
    ) FROM details d WHERE d.media_item_id = mi.id) AS "details: Json<Details>",

    (SELECT jsonb_build_object(
        'mediaItemId', cd.media_item_id, 'iso', cd.iso, 'exposureTime', cd.exposure_time, 'aperture', cd.aperture,
        'focalLength', cd.focal_length, 'cameraMake', cd.camera_make, 'cameraModel', cd.camera_model
    ) FROM capture_details cd WHERE cd.media_item_id = mi.id) AS "capture_details: Json<CaptureDetails>",

    (SELECT jsonb_build_object(
        'mediaItemId', p.media_item_id, 'isPhotosphere', p.is_photosphere, 'projectionType', p.projection_type,
        'horizontalFovDeg', p.horizontal_fov_deg, 'verticalFovDeg', p.vertical_fov_deg,
        'centerYawDeg', p.center_yaw_deg, 'centerPitchDeg', p.center_pitch_deg
    ) FROM panorama p WHERE p.media_item_id = mi.id) AS "panorama: Json<Panorama>"
    -- ================= FIX ENDS HERE =================
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
