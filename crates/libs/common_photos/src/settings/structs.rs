use serde::{Deserialize, Serialize};

/// Overall application configuration structure.
#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub directories: DirectoriesSettings,
    pub logging: LoggingSettings,
    pub database: DatabaseSettings,
    pub thumbnail_generation: ThumbnailGenerationSettings,
    pub api: ApiSettings,
    pub auth: AuthSettings,
    pub setup: SetupSettings,
    pub analyzer: AnalyzerSettings,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzerSettings {
    pub theme_generation: ThemeGenerationSettings,
    pub ocr: OCRSettings,
    pub fallback_timezone: String,
}

#[derive(Debug, Deserialize)]
pub struct ThemeGenerationSettings {
    pub variant: Variant,
    pub contrast_level: f64,
}

#[derive(Debug, Deserialize)]
pub struct OCRSettings {
    pub languages: Vec<String>,
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
    pub album_invitation_expiry_minutes: i64,
    pub public_url: String,
}

/// Defines paths for media and thumbnail storage.
#[derive(Debug, Deserialize)]
pub struct DirectoriesSettings {
    /// Folder with source photos and video
    pub media_folder: String,
    /// Folder where generated thumbnails will reside.
    pub thumbnails_folder: String,
}

/// Database connection and related configuration.
#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub min_connection: u32,
    pub max_lifetime: u64,
    pub idle_timeout: u64,
    pub acquire_timeout: u64,
    /// Length of generated `id` to use for media item in database.
    pub media_item_id_length: usize,
    pub album_id_length: usize,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum Variant {
    Monochrome,
    Neutral,
    TonalSpot,
    Vibrant,
    Expressive,
    Fidelity,
    Content,
    Rainbow,
    FruitSalad,
}

impl Variant {
    /// Converts the enum variant to its uppercase string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Monochrome => "MONOCHROME",
            Self::Neutral => "NEUTRAL",
            Self::TonalSpot => "TONAL_SPOT",
            Self::Vibrant => "VIBRANT",
            Self::Expressive => "EXPRESSIVE",
            Self::Fidelity => "FIDELITY",
            Self::Content => "CONTENT",
            Self::Rainbow => "RAINBOW",
            Self::FruitSalad => "FRUIT_SALAD",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoOutputFormat {
    /// The height of the output video in pixels. The width will be scaled automatically to maintain aspect ratio.
    pub height: u64,
    /// The quality setting for the video encoding. For VP9, this is the CRF (Constant Rate Factor) value.
    pub quality: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvifOptions {
    /// Quality 1..=100. Panics if out of range.
    pub quality: f32,
    /// Quality for the alpha channel only. `1..=100`. Panics if out of range.
    pub alpha_quality: f32,
    /// - 1 = very slow, but max compression.
    /// - 10 = quick, but larger file sizes and lower quality.
    ///
    /// Panics if outside 1..=10.
    pub speed: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoThumbOptions {
    /// The specific time in seconds from the start of the video to generate multi-size stills from.
    pub thumb_time: f64,
    /// A vector of percentages of the video's total duration at which to capture still images.
    pub percentages: Vec<u64>,
    /// The height in pixels for the thumbnails generated based on the `percentages` field.
    pub height: u64,
    /// A list of video formats to generate as previews from the source video.
    pub transcode_outputs: Vec<VideoOutputFormat>,
    /// The file extension for video transcoding (e.g., "webm", "mp4").
    pub extension: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThumbOptions {
    /// Which extensions are categorized as video
    pub video_extensions: Vec<String>,
    /// Which extensions are categorized as photos
    pub photo_extensions: Vec<String>,
    /// A vector of heights for generating multiple thumbnails.
    /// - For videos, these are the heights for stills taken at `thumb_time`.
    /// - For images, these are the heights for the generated thumbnails.
    pub heights: Vec<u64>,
    /// The file extension for photo thumbnails (e.g., "avif", "webp", "jpg").
    pub thumbnail_extension: String,
    pub avif_options: AvifOptions,
    pub video_options: VideoThumbOptions,
    pub skip_if_exists: bool,
}
