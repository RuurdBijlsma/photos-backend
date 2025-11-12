//! This module defines the HTTP handlers for the initial application setup process.

use crate::api_state::ApiState;
use crate::auth::db_model::User;
use crate::setup::error::SetupError;
use crate::setup::interfaces::{
    DiskResponse, FolderQuery, MakeFolderBody, MediaSampleResponse, StartProcessingBody,
    UnsupportedFilesResponse,
};
use crate::setup::service::{
    create_folder, get_disk_info, get_folder_unsupported_files, get_media_sample, get_subfolders,
    start_processing, validate_user_folder,
};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};

/// Retrieves information about the configured media and thumbnail disks.
#[utoipa::path(
    get,
    path = "/setup/disks",
    responses(
        (status = 200, description = "Disk information retrieved successfully", body = DiskResponse),
        (status = 500, description = "A configured path is not a valid directory"),
    )
)]
pub async fn get_disk_response() -> Result<Json<DiskResponse>, SetupError> {
    let disk_info = get_disk_info()?;
    Ok(Json(disk_info))
}

/// Retrieves a sample of media files from a specified folder.
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

/// Scans a folder and returns a list of unsupported file types.
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

/// Lists the subfolders within a given directory.
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
    let folders = get_subfolders(&query.folder).await?;
    Ok(Json(folders))
}

/// Creates a new folder within a specified base directory.
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
    create_folder(&params.base_folder, &params.new_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Start scanning the user folder and process the photos and videos.
///
/// # Errors
///
/// Returns a `SetupError` if a database connection cannot be established or the query fails.
#[utoipa::path(
    get,
    path = "/setup/start-processing",
    responses(
        (status = 200, description = "Processing job enqueued successfully.", body = bool),
        (status = 500, description = "Database error"),
    )
)]
pub async fn post_start_processing(
    State(api_state): State<ApiState>,
    Extension(user): Extension<User>,
    Json(payload): Json<StartProcessingBody>,
) -> Result<Json<bool>, SetupError> {
    start_processing(&user, &api_state.pool, payload.user_folder).await?;
    Ok(Json(true))
}
