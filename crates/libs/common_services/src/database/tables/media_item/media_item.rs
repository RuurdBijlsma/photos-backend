use crate::database::media_item::camera_settings::CameraSettings;
use crate::database::media_item::gps::Gps;
use crate::database::media_item::media_features::MediaFeatures;
use crate::database::media_item::panorama::Panorama;
use crate::database::media_item::time_details::TimeDetails;
use crate::database::media_item::weather::Weather;
use crate::database::visual_analysis::visual_analysis::ReadVisualAnalysis;
use chrono::{DateTime, NaiveDateTime, Utc};
use media_analyzer::MediaMetadata;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use utoipa::ToSchema;

/// The root struct representing a '`media_item`' and all its available, nested information.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateFullMediaItem {
    pub hash: String,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_local: NaiveDateTime,
    pub taken_at_utc: Option<DateTime<Utc>>,
    pub use_panorama_viewer: bool,
    pub gps: Option<Gps>,
    pub time: TimeDetails,
    pub weather: Option<Weather>,
    pub media_features: MediaFeatures,
    pub camera_settings: CameraSettings,
    pub panorama: Panorama,
    pub orientation: i32,
}

impl From<MediaMetadata> for CreateFullMediaItem {
    fn from(result: MediaMetadata) -> Self {
        let media_features = MediaFeatures {
            mime_type: result.basic.mime_type,
            size_bytes: result.basic.size_bytes as i64,
            is_motion_photo: result.features.is_motion_photo,
            motion_photo_presentation_timestamp: result
                .features
                .motion_photo_presentation_timestamp,
            is_hdr: result.features.is_hdr,
            is_burst: result.features.is_burst,
            burst_id: result.features.burst_id,
            capture_fps: result.features.capture_fps.map(|fps| fps as f32),
            video_fps: result.features.video_fps.map(|fps| fps as f32),
            is_nightsight: result.features.is_night_sight,
            is_timelapse: result.features.is_timelapse,
            exif: result.exif,
        };

        Self {
            hash: result.hash,
            width: result.basic.width as i32,
            height: result.basic.height as i32,
            is_video: result.features.is_video,
            duration_ms: result.basic.duration.map(|d_sec| (d_sec * 1000.0) as i64),
            taken_at_local: result.time.datetime_local,
            taken_at_utc: result.time.datetime_utc,
            use_panorama_viewer: result.panorama.use_panorama_viewer,
            gps: result.gps.map(Into::into),
            time: result.time.into(),
            weather: result.weather.map(Into::into),
            media_features,
            camera_settings: result.camera.into(),
            panorama: result.panorama.into(),
            orientation: result.basic.orientation.unwrap_or(1) as i32,
        }
    }
}

/// The root struct representing a '`media_item`' and all its available, nested information.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FullMediaItem {
    pub id: String,
    pub user_id: i32,
    pub hash: String,
    pub relative_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_local: NaiveDateTime,
    pub taken_at_utc: Option<DateTime<Utc>>,
    pub use_panorama_viewer: bool,
    pub visual_analyses: Vec<ReadVisualAnalysis>,
    pub gps: Option<Gps>,
    pub time: TimeDetails,
    pub weather: Option<Weather>,
    pub media_features: MediaFeatures,
    pub camera_settings: CameraSettings,
    pub panorama: Panorama,
}

#[derive(Debug, Clone)]
pub struct FullMediaItemRow {
    pub id: String,
    pub user_id: i32,
    pub hash: String,
    pub filename: String,
    pub relative_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_local: NaiveDateTime,
    pub taken_at_utc: Option<DateTime<Utc>>,
    pub use_panorama_viewer: bool,
    pub visual_analyses: Json<Vec<ReadVisualAnalysis>>,
    pub gps: Option<Json<Gps>>,
    pub time: Json<TimeDetails>,
    pub weather: Option<Json<Weather>>,
    pub media_features: Json<MediaFeatures>,
    pub camera_settings: Json<CameraSettings>,
    pub panorama: Json<Panorama>,
}

impl From<FullMediaItemRow> for FullMediaItem {
    fn from(r: FullMediaItemRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            hash: r.hash,
            relative_path: r.relative_path,
            created_at: r.created_at,
            updated_at: r.updated_at,
            width: r.width,
            height: r.height,
            is_video: r.is_video,
            duration_ms: r.duration_ms,
            taken_at_local: r.taken_at_local,
            taken_at_utc: r.taken_at_utc,
            use_panorama_viewer: r.use_panorama_viewer,
            visual_analyses: r.visual_analyses.0,
            gps: r.gps.map(|g| g.0),
            time: r.time.0,
            weather: r.weather.map(|w| w.0),
            media_features: r.media_features.0,
            camera_settings: r.camera_settings.0,
            panorama: r.panorama.0,
        }
    }
}
