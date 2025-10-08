use crate::routes::setup::error::SetupError;
use crate::routes::setup::interfaces::{
    DiskResponse, FolderQuery, MakeFolderBody, MediaSampleResponse, UnsupportedFilesResponse,
};
use crate::routes::setup::service::{
    contains_non_alphanumeric, get_folder_unsupported_files, get_media_sample, list_folders,
    validate_disks, validate_media_and_user_directory,
};
use axum::extract::{Query, State};
use axum::Json;
use common_photos::{get_media_dir, get_relative_path_str, get_thumbnails_dir, to_posix_string};
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::fs;
use tracing::warn;

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

pub async fn get_folder_media_sample(
    Query(query): Query<FolderQuery>,
) -> Result<Json<MediaSampleResponse>, SetupError> {
    let user_path = validate_media_and_user_directory(&query.folder).await?;
    let response = get_media_sample(&user_path)?;
    Ok(Json(response))
}

pub async fn get_folder_unsupported(
    Query(query): Query<FolderQuery>,
) -> Result<Json<UnsupportedFilesResponse>, SetupError> {
    let user_path = validate_media_and_user_directory(&query.folder).await?;
    let response = get_folder_unsupported_files(&user_path)?;
    Ok(Json(response))
}

pub async fn get_folders(
    Query(query): Query<FolderQuery>,
) -> Result<Json<Vec<String>>, SetupError> {
    let user_path = validate_media_and_user_directory(&query.folder).await?;
    let folders = list_folders(&user_path).await?;

    let relative_folders = folders
        .iter()
        .map(get_relative_path_str)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(relative_folders))
}

pub async fn make_folder(Json(params): Json<MakeFolderBody>) -> Result<Json<()>, SetupError> {
    if contains_non_alphanumeric(&params.new_name) {
        return Err(SetupError::DirectoryCreation(params.new_name));
    }

    let user_path = validate_media_and_user_directory(&params.base_folder).await?;
    fs::create_dir_all(user_path.join(params.new_name)).await?;
    Ok(Json(()))
}

static SETUP_DONE: AtomicBool = AtomicBool::new(false);

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