use super::error::CameraError;
use crate::api::camera::interfaces::CameraSummary;
use common_types::pb::api::{
    CameraInfo, FullCameraPhotosResponse, ListCameraResponse, SimpleTimelineItem,
};
use sqlx::PgPool;
use tracing::instrument;

#[instrument(skip(pool))]
pub async fn get_all_cameras(
    pool: &PgPool,
    user_id: i32,
) -> Result<ListCameraResponse, CameraError> {
    let cameras = sqlx::query_as!(
        CameraSummary,
        r#"
            SELECT
                MIN(TRIM(c.camera_make)) AS "camera_make!",
                REGEXP_REPLACE(MIN(TRIM(c.camera_model)), '\s*\(.*\)$', '', 'i') AS "camera_model!",
                COUNT(*)::INT AS "count!"
            FROM camera_settings c
                     INNER JOIN media_item m ON c.media_item_id = m.id
            WHERE m.user_id = $1
              AND m.deleted = false
              AND c.camera_make IS NOT NULL
              AND c.camera_model IS NOT NULL
              AND c.camera_make != '--'
              AND c.camera_model != '--'
              AND LOWER(TRIM(c.camera_make)) != 'unknown'
            GROUP BY
                LOWER(TRIM(c.camera_make)),
                LOWER(REGEXP_REPLACE(TRIM(c.camera_model), '\s*\(.*\)$', '', 'i'))
            ORDER BY COUNT(*) DESC;
            "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let camera_pb = cameras
        .into_iter()
        .map(|p| CameraInfo {
            make: p.camera_make,
            model: p.camera_model,
            photo_count: p.count,
        })
        .collect();

    Ok(ListCameraResponse { cameras: camera_pb })
}

#[instrument(skip(pool))]
pub async fn get_camera_photos(
    pool: &PgPool,
    camera_make: &str,
    camera_model: &str,
    user_id: i32,
) -> Result<FullCameraPhotosResponse, CameraError> {
    let items = sqlx::query_as!(
        SimpleTimelineItem,
        r#"
            SELECT
                mi.id,
                mi.is_video,
                mi.has_thumbnails,
                mi.duration_ms::INT AS "duration_ms",
                (mi.width::real / mi.height::real) AS "ratio!"
            FROM camera_settings c
            INNER JOIN media_item mi ON c.media_item_id = mi.id
            WHERE mi.user_id = $1
              AND mi.deleted = false
              AND LOWER(TRIM(c.camera_make)) = LOWER(TRIM($2))
              AND LOWER(REGEXP_REPLACE(TRIM(c.camera_model), '\s*\(.*\)$', '', 'i')) = LOWER(TRIM($3))
            ORDER BY mi.sort_timestamp DESC;
        "#,
        user_id,
        camera_make,
        camera_model
    )
        .fetch_all(pool)
        .await?;

    let photo_count = items.len() as i32;

    Ok(FullCameraPhotosResponse {
        camera: Some(CameraInfo {
            photo_count,
            make: camera_make.to_string(),
            model: camera_model.to_string(),
        }),
        items,
    })
}
