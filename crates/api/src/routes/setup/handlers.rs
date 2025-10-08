use crate::routes::setup::error::SetupError;
use crate::routes::setup::interfaces::{
    DiskResponse, FolderQuery, MakeFolderBody, MediaSampleResponse, UnsupportedFilesResponse,
};
use crate::routes::setup::service::{
    contains_non_alphanumeric, get_folder_unsupported_files, get_media_sample, list_folders,
    validate_disks, validate_user_folder,
};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use common_photos::{canon_relative_path, get_media_dir, get_thumbnails_dir, to_posix_string};
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::fs;
use tracing::warn;

/// Get information about the configured media and thumbnail disks.
#[utoipa::path(
    get,
    path = "/setup/disks",
    responses(
        (status = 200, description = "Disk information retrieved successfully", body = DiskResponse),
        (status = 500, description = "A configured path is not a valid directory"),
    )
)]
pub async fn get_disk_response() -> Result<Json<DiskResponse>, SetupError> {
    let media_path = get_media_dir();
    let thumbnail_path = get_thumbnails_dir();

    if !media_path.is_dir() {
        let path_str = to_posix_string(&media_path);
        warn!("Media path {} is not a valid directory", path_str);
        return Err(SetupError::InvalidPath(path_str));
    }

    if !thumbnail_path.is_dir() {
        let path_str = to_posix_string(&thumbnail_path);
        warn!("Thumbnail path {} is not a valid directory", path_str);
        return Err(SetupError::InvalidPath(path_str));
    }

    let disk_info = validate_disks(&media_path, &thumbnail_path)?;
    Ok(Json(disk_info))
}

/// Get a sample of media files from a specific folder.
#[utoipa::path(
    get,
    path = "/setup/media-sample",
    params(
        ("folder" = String, Query, description = "The folder to sample media from")
    ),
    responses(
        (status = 200, description = "Media sample retrieved successfully", body = MediaSampleResponse),
        (status = 400, description = "Invalid folder path provided"),
    )
)]
pub async fn get_folder_media_sample(
    Query(query): Query<FolderQuery>,
) -> Result<Json<MediaSampleResponse>, SetupError> {
    let user_path = validate_user_folder(&query.folder).await?;
    let response = get_media_sample(&user_path)?;
    Ok(Json(response))
}

/// Get a list of unsupported files in a specific folder.
#[utoipa::path(
    get,
    path = "/setup/unsupported-files",
    params(
        ("folder" = String, Query, description = "The folder to scan for unsupported files")
    ),
    responses(
        (status = 200, description = "Unsupported files listed successfully", body = UnsupportedFilesResponse),
        (status = 400, description = "Invalid folder path provided"),
    )
)]
pub async fn get_folder_unsupported(
    Query(query): Query<FolderQuery>,
) -> Result<Json<UnsupportedFilesResponse>, SetupError> {
    let user_path = validate_user_folder(&query.folder).await?;
    let response = get_folder_unsupported_files(&user_path)?;
    Ok(Json(response))
}

/// List the subfolders within a given folder.
#[utoipa::path(
    get,
    path = "/setup/folders",
    params(
        ("folder" = String, Query, description = "The base folder to list subdirectories from")
    ),
    responses(
        (status = 200, description = "Folders listed successfully", body = Vec<String>),
        (status = 400, description = "Invalid base folder path provided"),
    )
)]
pub async fn get_folders(
    Query(query): Query<FolderQuery>,
) -> Result<Json<Vec<String>>, SetupError> {
    let user_path = validate_user_folder(&query.folder).await?;
    let folders = list_folders(&user_path).await?;

    let relative_folders = folders
        .iter()
        .map(canon_relative_path)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(relative_folders))
}

/// Create a new folder.
#[utoipa::path(
    post,
    path = "/setup/make-folder",
    request_body = MakeFolderBody,
    responses(
        (status = 204, description = "Folder created successfully"),
        (status = 400, description = "Invalid folder name or path"),
    )
)]
pub async fn make_folder(Json(params): Json<MakeFolderBody>) -> Result<StatusCode, SetupError> {
    if contains_non_alphanumeric(&params.new_name) {
        return Err(SetupError::DirectoryCreation(params.new_name));
    }

    let user_path = validate_user_folder(&params.base_folder).await?;
    fs::create_dir_all(user_path.join(params.new_name)).await?;
    Ok(StatusCode::NO_CONTENT)
}

static SETUP_DONE: AtomicBool = AtomicBool::new(false);

/// Check if the initial setup is required (i.e., if any admin user exists).
#[utoipa::path(
    get,
    path = "/setup/needed",
    responses(
        (status = 200, description = "Setup status retrieved successfully", body = bool),
        (status = 500, description = "Database error"),
    )
)]
pub async fn setup_needed(State(pool): State<PgPool>) -> Result<Json<bool>, SetupError> {
    if SETUP_DONE.load(Ordering::Relaxed) {
        return Ok(Json(false));
    }

    let count: Option<i64> = sqlx::query_scalar!("SELECT count(id) FROM app_user")
        .fetch_one(&pool)
        .await?;

    let user_exists = count.unwrap_or(0) > 0;

    if user_exists {
        SETUP_DONE.store(true, Ordering::Relaxed);
    }

    Ok(Json(!user_exists))
}