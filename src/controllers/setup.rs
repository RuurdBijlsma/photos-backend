use axum::debug_handler;
use axum::extract::Query;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, warn};

use crate::common::settings::Settings;
use crate::controllers::logic::setup::{
    list_folders, to_posix_string, validate_disks, validate_media_and_user_directory,
    validate_user_folder, MediaError,
};
use crate::models::users::users;
use loco_rs::prelude::*;
use serde::Deserialize;

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

/// Asynchronous handler to check if the configured disks are valid and process it.
///
/// # Returns
///
/// A JSON response with info on the disks.
#[debug_handler]
async fn get_disk_response(_: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let settings = Settings::from_context(&ctx);
    let media_path = Path::new(&settings.media_dir);
    let thumbnail_path = Path::new(&settings.thumbnails_dir);

    if !media_path.exists() {
        warn!("Media path {} does not exist", media_path.display());
        return not_found();
    }

    if !media_path.is_dir() {
        let msg = format!("{} is not a directory", media_path.display());
        warn!("{}", msg);
        return bad_request(msg);
    }

    if !thumbnail_path.exists() {
        warn!("Thumbnail path {} does not exist", thumbnail_path.display());
        return not_found();
    }

    if !thumbnail_path.is_dir() {
        let msg = format!("{} is not a directory", thumbnail_path.display());
        warn!("{}", msg);
        return bad_request(msg);
    }

    let response = validate_disks(media_path, thumbnail_path).map_err(Into::<Error>::into)?;
    format::json(response)
}

#[derive(Deserialize)]
struct FolderQuery {
    folder: PathBuf,
}

/// Asynchronous handler to check if a given media directory is valid and process it.
///
/// Validates that the media directory exists and is a directory, then processes it to count media files.
///
/// # Returns
///
/// A JSON response with the count and sample file paths.
#[debug_handler]
async fn get_user_folder_response(
    _: auth::JWT,
    State(ctx): State<AppContext>,
    Query(query): Query<FolderQuery>,
) -> Result<Response> {
    let settings = Settings::from_context(&ctx);
    match validate_media_and_user_directory(&settings.media_dir, &query.folder).await {
        Ok((media_path, user_path)) => {
            let response =
                validate_user_folder(&media_path, &user_path).map_err(Into::<Error>::into)?;
            format::json(response)
        }
        Err(_) => bad_request("Incorrect folder or MEDIA_DIR provided."),
    }
}

static SETUP_DONE: AtomicBool = AtomicBool::new(false);

/// Determines if the application setup is needed.
///
/// Checks whether a user exists in the database and sets a static flag accordingly.
///
/// # Returns
///
/// A JSON response indicating whether setup is needed.
///
/// # Errors
///
/// DB connection error is possible when requesting user accounts.
async fn setup_needed(State(ctx): State<AppContext>) -> Result<Response> {
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
async fn get_folders(
    _: auth::JWT,
    Query(query): Query<FolderQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let settings = Settings::from_context(&ctx);
    match validate_media_and_user_directory(&settings.media_dir, &query.folder).await {
        Ok((_, user_path)) => {
            let folders = list_folders(&user_path)
                .await
                .map_err(Into::<Error>::into)?;

            let relative_folders: Vec<String> = folders
                .into_iter()
                .map(|entry| {
                    entry
                        .strip_prefix(&user_path)
                        .map_err(|_| MediaError::PathConversion(to_posix_string(&entry)))?
                        .to_str()
                        .ok_or_else(|| MediaError::PathConversion(to_posix_string(&entry)))
                        .map(String::from)
                })
                .collect::<std::result::Result<_, _>>()?;
            format::json(relative_folders)
        }
        Err(_) => bad_request("Incorrect folder or MEDIA_DIR provided."),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/setup")
        .add("/needed", get(setup_needed))
        .add("/disk-info", get(get_disk_response))
        .add("/user-folder-info", get(get_user_folder_response))
        .add("/folders", get(get_folders))
}
