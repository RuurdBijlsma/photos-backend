use crate::insert_query;
use crate::utils::nice_id;
use media_analyzer::{AnalyzeResult, LocationName};
use sqlx::{PgTransaction, Postgres, Transaction};

async fn get_or_create_location(
    tx: &mut Transaction<'_, Postgres>,
    location_data: &LocationName,
) -> Result<i32, sqlx::Error> {
    let existing_id: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM location WHERE name = $1 AND admin1 = $2 AND country_code = $3",
    )
        .bind(&location_data.name)
        .bind(&location_data.admin1)
        .bind(&location_data.country_code)
        .fetch_optional(&mut **tx)
        .await?;

    if let Some(id) = existing_id {
        Ok(id)
    } else {
        let new_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO location (name, admin1, admin2, country_code, country_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(&location_data.name)
            .bind(&location_data.admin1)
            .bind(&location_data.admin2)
            .bind(&location_data.country_code)
            .bind(&location_data.country_name)
            .fetch_one(&mut **tx)
            .await?;
        Ok(new_id)
    }
}


/// Inserts a full media item using your `AnalyzeResult` struct within a single transaction.
pub async fn store_media_item(
    tx: &mut PgTransaction<'_>,
    relative_path: &str,
    data: &AnalyzeResult,
) -> Result<String, sqlx::Error> {

    let existing_id: Option<String> =
        sqlx::query_scalar("SELECT id FROM media_item WHERE relative_path = $1")
            .bind(relative_path)
            .fetch_optional(&mut **tx)
            .await?;

    if let Some(id) = existing_id {
        return Ok(id);
    }

    let media_item_id = nice_id(10);

    insert_query!(tx, "media_item", {
        id: &media_item_id,
        relative_path: relative_path,
        width: data.metadata.width as i32,
        height: data.metadata.height as i32,
        is_video: data.tags.is_video,
        data_url: &data.data_url,
        duration_ms: data.metadata.duration.map(|d| (d * 1000.0) as i64),
        taken_at_naive: data.time_info.datetime_naive,
        use_panorama_viewer: data.pano_info.use_panorama_viewer,
    }).await?;

    if let Some(gps_info) = &data.gps_info {
        let location_id = get_or_create_location(tx, &gps_info.location).await?;
        insert_query!(tx, "gps", {
            media_item_id: &media_item_id,
            location_id: location_id,
            latitude: gps_info.latitude,
            longitude: gps_info.longitude,
            altitude: gps_info.altitude,
            image_direction: gps_info.image_direction,
        }).await?;
    }

    insert_query!(tx, "time_details", {
        media_item_id: &media_item_id,
        datetime_utc: data.time_info.datetime_utc,
        timezone_name: data.time_info.timezone.as_ref().map(|tz| &tz.name),
        timezone_offset_seconds: data.time_info.timezone.as_ref().map(|tz| tz.offset_seconds),
        source: data.time_info.timezone.as_ref().map(|tz| &tz.source),
        source_details: &data.time_info.source_details.time_source,
        source_confidence: &data.time_info.source_details.confidence,
    }).await?;

    if let Some(weather_info) = &data.weather_info {
        let hourly = weather_info.hourly.as_ref();
        let condition = hourly.and_then(|h| h.condition).map(|c| c.to_string());
        insert_query!(tx, "weather", {
            media_item_id: &media_item_id,
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
        }).await?;
    }

    insert_query!(tx, "details", {
        media_item_id: &media_item_id,
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
    }).await?;

    insert_query!(tx, "capture_details", {
        media_item_id: &media_item_id,
        iso: data.capture_details.iso.map(|i| i as i32),
        exposure_time: data.capture_details.exposure_time.map(|a| a as f32),
        aperture: data.capture_details.aperture.map(|a| a as f32),
        focal_length: data.capture_details.focal_length.map(|fl| fl as f32),
        camera_make: &data.capture_details.camera_make,
        camera_model: &data.capture_details.camera_model,
    }).await?;

    insert_query!(tx, "panorama", {
        media_item_id: &media_item_id,
        is_photosphere: data.pano_info.is_photosphere,
        projection_type: &data.pano_info.projection_type,
        horizontal_fov_deg: data.pano_info.view_info.as_ref().map(|vi| vi.horizontal_fov_deg as f32),
        vertical_fov_deg: data.pano_info.view_info.as_ref().map(|vi| vi.vertical_fov_deg as f32),
        center_yaw_deg: data.pano_info.view_info.as_ref().map(|vi| vi.center_yaw_deg as f32),
        center_pitch_deg: data.pano_info.view_info.as_ref().map(|vi| vi.center_pitch_deg as f32),
    }).await?;

    Ok(media_item_id)
}