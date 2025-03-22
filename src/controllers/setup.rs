use axum::debug_handler;
use derive_more::with_trait::Constructor;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use tracing::{error, warn};
use walkdir::WalkDir;

use crate::common::image_utils::{is_photo_file, is_video_file};
use crate::common::settings::Settings;
use crate::models::users::users;
use loco_rs::prelude::*;

/// Custom error type for media directory operations.
#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Invalid media directory: {0}")]
    InvalidMediaDir(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Path conversion error for path: {0}")]
    PathConversion(String),
}

impl From<MediaError> for loco_rs::Error {
    fn from(err: MediaError) -> Self {
        match err {
            MediaError::InvalidMediaDir(msg) => {
                error!("Invalid Media Directory: {:?}", msg);
                Self::BadRequest(msg)
            }
            MediaError::FileSystem(e) => {
                error!("File system error: {:?}", e);
                Self::InternalServerError
            }
            MediaError::PathConversion(path) => {
                error!("Path conversion error for path: {}", path);
                Self::InternalServerError
            }
        }
    }
}

/// Response structure for file count and sample paths.
#[derive(Constructor, Serialize)]
pub struct FileCountResponse {
    count: usize,
    samples: Vec<String>,
}

/// Processes a media directory by counting photo/video files and collecting up to 10 random samples.
///
/// Uses reservoir sampling to maintain a fixed-size sample set.
/// Returns a `FileCountResponse` containing the total count and relative file paths.
///
/// # Errors
///
/// Returns a `MediaError` if there is a filesystem issue or if a path cannot be converted to a UTF-8 string.
fn process_media_dir(media_path: &Path) -> Result<FileCountResponse, MediaError> {
    let mut count = 0;
    let mut samples = Vec::with_capacity(10);
    let media_path_buf = PathBuf::from(media_path);

    for entry in WalkDir::new(media_path).into_iter().filter_map(|e| {
        match e {
            Ok(entry) => Some(entry),
            Err(e) => {
                // Convert walkdir::Error to std::io::Error and log the error.
                let io_error = e
                    .into_io_error()
                    .unwrap_or_else(|| std::io::Error::other("walkdir error"));
                error!("Directory walk error: {}", io_error);
                None
            }
        }
    }) {
        if !entry.file_type().is_file()
            || (!is_photo_file(entry.path()) && !is_video_file(entry.path()))
        {
            continue;
        }

        count += 1;

        // Reservoir sampling: for the first 10 files, just push. After that, replace a random element.
        if count <= 10 {
            samples.push(entry);
        } else {
            let random_index = fastrand::usize(0..count);
            if random_index < 10 {
                samples[random_index] = entry;
            }
        }
    }

    // Convert absolute paths to paths relative to the media directory.
    let relative_samples: Vec<String> = samples
        .into_iter()
        .map(|entry| {
            entry
                .path()
                .strip_prefix(&media_path_buf)
                .map_err(|_| MediaError::PathConversion(entry.path().display().to_string()))?
                .to_str()
                .ok_or_else(|| MediaError::PathConversion(entry.path().display().to_string()))
                .map(std::string::ToString::to_string)
        })
        .collect::<Result<_, _>>()?;

    Ok(FileCountResponse::new(count, relative_samples))
}

/// Asynchronous handler to check if a given media directory is valid and process it.
///
/// Validates that the media directory exists and is a directory, then processes it to count media files.
///
/// # Returns
///
/// A JSON response with the count and sample file paths.
#[debug_handler]
async fn check_media_dir(_: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let settings = Settings::from_context(&ctx);
    let media_dir = settings.media_dir;
    let media_path = Path::new(&media_dir);

    if !media_path.exists() {
        warn!("Media path {} does not exist", media_path.display());
        return not_found();
    }

    if !media_path.is_dir() {
        let msg = format!("{} is not a directory", media_path.display());
        warn!("{}", msg);
        return bad_request(msg);
    }

    let response = process_media_dir(media_path).map_err(Into::<loco_rs::Error>::into)?;
    format::json(response)
}

static SETUP_DONE: AtomicBool = AtomicBool::new(false);

/// Determines if the application setup is needed.
///
/// Checks whether a user exists in the database and sets a static flag accordingly.
///
/// # Returns
///
/// A JSON response indicating whether setup is needed.
pub async fn setup_needed(State(ctx): State<AppContext>) -> Result<Response> {
    if SETUP_DONE.load(Ordering::Relaxed) {
        return format::json(false);
    }

    let user_exists = users::Entity::find().one(&ctx.db).await?.is_some();
    SETUP_DONE.store(user_exists, Ordering::Relaxed);

    if user_exists {
        return format::json(false);
    }

    format::json(true)
}

/// Defines API routes for setup-related endpoints.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/setup")
        .add("/needed", get(setup_needed))
        .add("/check-media-dir", get(check_media_dir))
}
