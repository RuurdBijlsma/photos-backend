use ruurd_photos_thumbnail_generation::{AvifOptions, VideoThumbOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub directories: DirectoriesConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub worker: WorkerConfig,
    pub thumbnail_generation: ThumbnailGenerationConfig,
    pub api: ApiConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_expiry_minutes: i64,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u32,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DirectoriesConfig {
    /// Folder with source photos and video
    pub media_folder: String,
    /// Folder where generated thumbnails will reside.
    pub thumbnails_folder: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkerConfig {
    pub wait_after_empty_queue_s: u64,
    pub wait_after_error_s: u64,
    pub max_retries: i32,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    /// Length of generated `id` to use for media item in database.
    pub media_item_id_length: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailGenerationConfig {
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
