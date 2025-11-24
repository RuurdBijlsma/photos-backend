use crate::database::media_item::capture_details::CaptureDetails;
use crate::database::media_item::gps::Gps;
use crate::database::media_item::media_details::MediaDetails;
use crate::database::media_item::panorama::Panorama;
use crate::database::media_item::time_details::TimeDetails;
use crate::database::media_item::weather::Weather;
use crate::database::visual_analysis::visual_analysis::ReadVisualAnalysis;
use chrono::{DateTime, NaiveDateTime, Utc};
use media_analyzer::AnalyzeResult;
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
    pub time_details: TimeDetails,
    pub weather: Option<Weather>,
    pub media_details: MediaDetails,
    pub capture_details: CaptureDetails,
    pub panorama: Panorama,
}

impl From<AnalyzeResult> for CreateFullMediaItem {
    fn from(result: AnalyzeResult) -> Self {
        let media_details = MediaDetails {
            mime_type: result.metadata.mime_type,
            size_bytes: result.metadata.size_bytes as i64,
            is_motion_photo: result.tags.is_motion_photo,
            motion_photo_presentation_timestamp: result.tags.motion_photo_presentation_timestamp,
            is_hdr: result.tags.is_hdr,
            is_burst: result.tags.is_burst,
            burst_id: result.tags.burst_id,
            capture_fps: result.tags.capture_fps.map(|fps| fps as f32),
            video_fps: result.tags.video_fps.map(|fps| fps as f32),
            is_nightsight: result.tags.is_night_sight,
            is_timelapse: result.tags.is_timelapse,
            exif: result.exif,
        };

        Self {
            hash: result.hash,
            width: result.metadata.width as i32,
            height: result.metadata.height as i32,
            is_video: result.tags.is_video,
            duration_ms: result
                .metadata
                .duration
                .map(|d_sec| (d_sec * 1000.0) as i64),
            taken_at_local: result.time_info.datetime_local,
            taken_at_utc: result.time_info.datetime_utc,
            use_panorama_viewer: result.pano_info.use_panorama_viewer,
            gps: result.gps_info.map(Into::into),
            time_details: result.time_info.into(),
            weather: result.weather_info.map(Into::into),
            media_details,
            capture_details: result.capture_details.into(),
            panorama: result.pano_info.into(),
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
    pub time_details: TimeDetails,
    pub weather: Option<Weather>,
    pub media_details: MediaDetails,
    pub capture_details: CaptureDetails,
    pub panorama: Panorama,
}

#[derive( Debug, Clone)]
pub struct FullMediaItemRow {
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
    pub visual_analyses: Json<Vec<ReadVisualAnalysis>>,
    pub gps: Option<Json<Gps>>,
    pub time_details: Json<TimeDetails>,
    pub weather: Option<Json<Weather>>,
    pub media_details: Json<MediaDetails>,
    pub capture_details: Json<CaptureDetails>,
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
            time_details: r.time_details.0,
            weather: r.weather.map(|w| w.0),
            media_details: r.media_details.0,
            capture_details: r.capture_details.0,
            panorama: r.panorama.0,
        }
    }
}
