//! # Thumbnail Generation Crate
//!
//! A personal library for generating a variety of thumbnails from image and video files
//! using `FFmpeg` and `FFprobe`.
//!
//! This crate provides a unified interface, `generate_thumbnails`, which can handle
//! both image and video files based on their extension. The generation process is highly
//! configurable through the `ThumbOptions` struct, allowing for the creation of:
//! - Multiple sizes of still images from a single timestamp in a video.
//! - Stills from multiple timestamps (as percentages) in a video.
//! - Lower-resolution video previews (e.g., `WebM`).
//! - Multiple sizes of thumbnails from a source image.
//!
//! All operations are performed asynchronously using `tokio`.
//!
//! ## Requirements
//!
//! - **`FFmpeg`**: Must be installed and accessible in the system's `PATH`.
//! - **`FFprobe`**: Must be installed and accessible in the system's `PATH`.
//!
//! ## Example
//!
//! ```no_run
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> color_eyre::Result<()> {
//!     let source_file = Path::new("path/to/video.mp4");
//!     let output_dir = Path::new("path/to/thumbnails");
//!
//!     let config = ThumbOptions {
//!         photo_extensions: ["jpg", "jpeg", "png", "gif", "tiff", "tga", "avif"]
//!             .iter()
//!             .map(|x| x.to_string())
//!             .collect(),
//!         video_extensions: [
//!             "mp4", "webm", "av1", "3gp", "mov", "mkv", "flv", "m4v", "m4p",
//!         ]
//!         .iter()
//!         .map(|x| x.to_string())
//!         .collect(),
//!         skip_if_exists: true,
//!         heights: vec![10, 144, 240, 360, 480, 720, 1080],
//!         thumbnail_extension: "avif".to_string(),
//!         avif_options: AvifOptions {
//!             quality: 80.,
//!             alpha_quality: 80.,
//!             speed: 4,
//!         },
//!         video_options: VideoThumbOptions {
//!             extension: "webm".to_string(),
//!             thumb_time: 0.5,
//!             percentages: vec![0, 33, 66, 99],
//!             height: 720,
//!             transcode_outputs: vec![
//!                 VideoOutputFormat {
//!                     height: 480,
//!                     quality: 35,
//!                 },
//!                 VideoOutputFormat {
//!                     height: 144,
//!                     quality: 40,
//!                 },
//!             ],
//!         },
//!     };
//!
//!     if let Err(e) = generate_thumbnails(source_file, &output_dir.join("vid_thumbs"), &config).await {
//!         eprintln!("Failed to generate thumbnails: {}", e);
//!     }
//!
//!     Ok(())
//! }
//! ```

// Internal module for utility functions, like moving files.
mod utils;
// The core module for generating thumbnails.
mod thumbnails;
// Module for interacting with the `ffprobe` command-line tool.
mod ffprobe;
// Module for interacting with the `ffmpeg` command-line tool.
mod ffmpeg;

// Re-export the primary configuration structs and the main function for easy access.
pub use thumbnails::generic_thumbnails::{
    generate_thumbnails,
};
