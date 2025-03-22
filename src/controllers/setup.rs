use axum::debug_handler;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, warn};

use crate::common::settings::Settings;
use crate::controllers::logic::setup::{process_media_dir, MediaError};
use crate::models::users::users;
use loco_rs::prelude::*;

impl From<MediaError> for Error {
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

    let response = process_media_dir(media_path).map_err(Into::<Error>::into)?;
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
