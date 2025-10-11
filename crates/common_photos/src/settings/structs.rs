use ruurd_photos_thumbnail_generation::{AvifOptions, VideoThumbOptions};
use serde::Deserialize;

/// Overall application configuration structure.
#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub directories: DirectoriesSettings,
    pub logging: LoggingSettings,
    pub database: DatabaseSettings,
    pub worker: WorkerSettings,
    pub thumbnail_generation: ThumbnailGenerationSettings,
    pub api: ApiSettings,
    pub auth: AuthSettings,
    pub setup: SetupSettings,
}

#[derive(Debug, Deserialize)]
pub struct SetupSettings {
    pub n_media_samples: usize,
}

/// Configuration for authentication, including JWT secrets and token expiry.
#[derive(Debug, Deserialize)]
pub struct AuthSettings {
    pub jwt_secret: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
}

/// Configuration for the API server.
#[derive(Debug, Deserialize)]
pub struct ApiSettings {
    pub host: String,
    pub port: u32,
    pub allowed_origins: Vec<String>,
}

/// Defines paths for media and thumbnail storage.
#[derive(Debug, Deserialize)]
pub struct DirectoriesSettings {
    /// Folder with source photos and video
    pub media_folder: String,
    /// Folder where generated thumbnails will reside.
    pub thumbnails_folder: String,
}

/// Configuration for the background worker process.
#[derive(Debug, Deserialize)]
pub struct WorkerSettings {
    pub wait_after_empty_queue_s: u64,
    pub wait_after_error_s: u64,
    pub max_retries: i32,
}

/// Database connection and related configuration.
#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub pool_size: u32,
    /// Length of generated `id` to use for media item in database.
    pub media_item_id_length: usize,
}

/// Logging configuration.
#[derive(Debug, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
}

/// Configuration for thumbnail generation settings.
#[derive(Debug, Deserialize)]
pub struct ThumbnailGenerationSettings {
    /// The file extension for photo thumbnails (e.g., "avif", "webp", "jpg").
    pub thumbnail_extension: String,
    /// A vector of heights for generating multiple thumbnails.
    /// - For videos, these are the heights for stills taken at `thumb_time`.
    /// - For images, these are the heights for the generated thumbnails.
    pub heights: Vec<u64>,
    /// Which extensions are categorized as videos
    pub video_extensions: Vec<String>,
    /// Which extensions are categorized as photos
    pub photo_extensions: Vec<String>,
    pub avif_options: AvifOptions,
    pub video_options: VideoThumbOptions,
}
