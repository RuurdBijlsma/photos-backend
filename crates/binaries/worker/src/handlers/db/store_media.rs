use crate::insert_query;
use chrono::{TimeZone, Utc};
use media_analyzer::{AnalyzeResult, LocationName};
use sqlx::PgTransaction;
use common_services::database::album::pending_album_media_item::PendingAlbumMediaItem;
use common_services::get_settings::fallback_timezone;

async fn get_or_create_remote_user(
    tx: &mut PgTransaction<'_>,
    local_user_id: i32,
    remote_identity: &str,
) -> Result<i32, sqlx::Error> {
    let remote_user_id = sqlx::query_scalar!(
        "SELECT id FROM remote_user WHERE identity = $1 AND user_id = $2",
        remote_identity,
        local_user_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(id) = remote_user_id {
        return Ok(id);
    }

    // Not found, so create it
    let new_id = sqlx::query_scalar!(
        "INSERT INTO remote_user (identity, user_id) VALUES ($1, $2) RETURNING id",
        remote_identity,
        local_user_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(new_id)
}

/// Retrieves an existing location's ID or creates a new one if it doesn't exist.
///
/// # Errors
///
/// This function will return an error if any of the database select or insert operations fail.
async fn get_or_create_location(
    tx: &mut PgTransaction<'_>,
    location_data: &LocationName,
) -> Result<i32, sqlx::Error> {
    let existing_id: Option<i32> = sqlx::query_scalar!(
        "SELECT id FROM location WHERE name = $1 AND admin1 = $2 AND country_code = $3",
        &location_data.name,
        &location_data.admin1,
        &location_data.country_code,
    )
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(id) = existing_id {
        Ok(id)
    } else {
        let country_name = location_data
            .country_name
            .as_ref()
            .expect("Country name has to be set.");
        let new_id: i32 = sqlx::query_scalar!(
            r"
            INSERT INTO location (name, admin1, admin2, country_code, country_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            ",
            &location_data.name,
            &location_data.admin1,
            &location_data.admin2,
            &location_data.country_code,
            &country_name,
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(new_id)
    }
}

/// Inserts a full media item and its associated metadata into the database.
///
/// # Errors
///
/// Returns an error if any of the database deletion or insertion queries fail.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]
pub async fn store_media_item(
    tx: &mut PgTransaction<'_>,
    relative_path: &str,
    data: &AnalyzeResult,
    item_id: &str,
    user_id: i32,
) -> Result<String, sqlx::Error> {
    // Overwrite if it already exists.
    sqlx::query_scalar!(
        "DELETE FROM media_item WHERE relative_path = $1",
        relative_path
    )
    .execute(&mut **tx)
    .await?;

    let pending_info: Option<PendingAlbumMediaItem> = sqlx::query_as!(
        PendingAlbumMediaItem,
        r#"
        DELETE FROM pending_album_media_items
        WHERE relative_path = $1
        RETURNING album_id, remote_user_identity, relative_path
        "#,
        relative_path
    )
    .fetch_optional(&mut **tx)
    .await?;

    let remote_user_id = if let Some(info) = &pending_info {
        Some(get_or_create_remote_user(tx, user_id, &info.remote_user_identity).await?)
    } else {
        None
    };

    let sort_timestamp = data.time_info.datetime_utc.unwrap_or_else(|| {
        fallback_timezone().as_ref().map_or_else(
            || data.time_info.datetime_local.and_utc(),
            |tz| {
                tz.from_local_datetime(&data.time_info.datetime_local)
                    .unwrap()
                    .with_timezone(&Utc)
            },
        )
    });

    insert_query!(tx, "media_item", {
        id: &item_id,
        remote_user_id: remote_user_id,
        user_id: user_id,
        hash: &data.hash,
        relative_path: relative_path,
        width: data.metadata.width as i32,
        height: data.metadata.height as i32,
        is_video: data.tags.is_video,
        duration_ms: data.metadata.duration.map(|d| (d * 1000.0) as i64),
        taken_at_local: data.time_info.datetime_local,
        taken_at_utc: data.time_info.datetime_utc,
        sort_timestamp: sort_timestamp,
        use_panorama_viewer: data.pano_info.use_panorama_viewer,
    })
    .await?;

    if let Some(gps_info) = &data.gps_info {
        let location_id = get_or_create_location(tx, &gps_info.location).await?;
        insert_query!(tx, "gps", {
            media_item_id: &item_id,
            location_id: location_id,
            latitude: gps_info.latitude,
            longitude: gps_info.longitude,
            altitude: gps_info.altitude,
            image_direction: gps_info.image_direction,
        })
        .await?;
    }

    insert_query!(tx, "time_details", {
        media_item_id: &item_id,
        timezone_name: data.time_info.timezone.as_ref().map(|tz| &tz.name),
        timezone_offset_seconds: data.time_info.timezone.as_ref().map(|tz| tz.offset_seconds),
        source: data.time_info.timezone.as_ref().map(|tz| &tz.source),
        source_details: &data.time_info.source_details.time_source,
        source_confidence: &data.time_info.source_details.confidence,
    })
    .await?;

    if let Some(weather_info) = &data.weather_info {
        let hourly = weather_info.hourly.as_ref();
        let condition = hourly.and_then(|h| h.condition).map(|c| c.to_string());
        insert_query!(tx, "weather", {
            media_item_id: &item_id,
            temperature: hourly.and_then(|h| h.temperature).map(|t| t as f32),
            dew_point: hourly.and_then(|h| h.dew_point).map(|dp| dp as f32),
            relative_humidity: hourly.and_then(|h| h.relative_humidity).map(|rh| rh as f32),
            precipitation: hourly.and_then(|h| h.precipitation).map(|p| p as f32),
            snow: hourly.and_then(|h| h.snow).map(|s| s as f32),
            wind_direction: hourly.and_then(|h| h.wind_direction),
            wind_speed: hourly.and_then(|h| h.wind_speed).map(|ws| ws as f32),
            peak_wind_gust: hourly.and_then(|h| h.peak_wind_gust).map(|pg| pg as f32),
            pressure: hourly.and_then(|h| h.pressure).map(|p| p as f32),
            sunshine_minutes: hourly.and_then(|h| h.sunshine_minutes),
            condition: condition,
            sunrise: weather_info.sun_info.sunrise,
            sunset: weather_info.sun_info.sunset,
            dawn: weather_info.sun_info.dawn,
            dusk: weather_info.sun_info.dusk,
            is_daytime: weather_info.sun_info.is_daytime,
        })
        .await?;
    }

    insert_query!(tx, "details", {
        media_item_id: &item_id,
        is_motion_photo: data.tags.is_motion_photo,
        motion_photo_presentation_timestamp: data.tags.motion_photo_presentation_timestamp,
        is_hdr: data.tags.is_hdr,
        is_burst: data.tags.is_burst,
        burst_id: &data.tags.burst_id,
        capture_fps: data.tags.capture_fps.map(|f| f as f32),
        video_fps: data.tags.video_fps.map(|f| f as f32),
        is_nightsight: data.tags.is_night_sight,
        is_timelapse: data.tags.is_timelapse,
        mime_type: &data.metadata.mime_type,
        size_bytes: data.metadata.size_bytes as i64,
        exif: &data.exif,
    })
    .await?;

    insert_query!(tx, "capture_details", {
        media_item_id: &item_id,
        iso: data.capture_details.iso.map(|i| i as i32),
        exposure_time: data.capture_details.exposure_time.map(|a| a as f32),
        aperture: data.capture_details.aperture.map(|a| a as f32),
        focal_length: data.capture_details.focal_length.map(|fl| fl as f32),
        camera_make: &data.capture_details.camera_make,
        camera_model: &data.capture_details.camera_model,
    })
    .await?;

    insert_query!(tx, "panorama", {
        media_item_id: &item_id,
        is_photosphere: data.pano_info.is_photosphere,
        projection_type: &data.pano_info.projection_type,
        horizontal_fov_deg: data.pano_info.view_info.as_ref().map(|vi| vi.horizontal_fov_deg as f32),
        vertical_fov_deg: data.pano_info.view_info.as_ref().map(|vi| vi.vertical_fov_deg as f32),
        center_yaw_deg: data.pano_info.view_info.as_ref().map(|vi| vi.center_yaw_deg as f32),
        center_pitch_deg: data.pano_info.view_info.as_ref().map(|vi| vi.center_pitch_deg as f32),
    }).await?;

    Ok(item_id.to_string())
}
