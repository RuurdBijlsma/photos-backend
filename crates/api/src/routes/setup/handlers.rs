use crate::routes::setup::error::SetupError;
use crate::routes::setup::interfaces::{
    DiskResponse, FolderQuery, MakeFolderBody, MediaSampleResponse, UnsupportedFilesResponse,
};
use crate::routes::setup::service;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use sqlx::PgPool;

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
    let disk_info = service::get_disk_info()?;
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
    let user_path = service::validate_user_folder(&query.folder).await?;
    let response = service::get_media_sample(&user_path)?;
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
    let user_path = service::validate_user_folder(&query.folder).await?;
    let response = service::get_folder_unsupported_files(&user_path)?;
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
    let folders = service::get_subfolders(&query.folder).await?;
    Ok(Json(folders))
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
    service::create_folder(&params.base_folder, &params.new_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Check if the welcome page should be shown (i.e., if no user exists).
#[utoipa::path(
    get,
    path = "/setup/welcome-needed",
    responses(
        (status = 200, description = "Welcome status retrieved successfully", body = bool),
        (status = 500, description = "Database error"),
    )
)]
pub async fn welcome_needed(State(pool): State<PgPool>) -> Result<Json<bool>, SetupError> {
    let needed = service::is_welcome_needed(&pool).await?;
    Ok(Json(needed))
}
