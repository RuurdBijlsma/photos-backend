//! This module defines the HTTP handlers for the initial application onboarding process.

use crate::api_state::ApiContext;
use app_state::IngestSettings;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use common_services::api::onboarding::error::OnboardingError;
use common_services::api::onboarding::interfaces::{
    DiskResponse, FolderParams, MakeFolderBody, MediaSampleResponse, StartProcessingBody,
    UnsupportedFilesResponse,
};
use common_services::api::onboarding::service::{
    create_folder, get_disk_info, get_folder_unsupported_files, get_media_sample, get_subfolders,
    start_processing, validate_user_folder,
};
use common_services::database::app_user::User;

/// Retrieves information about the configured media and thumbnail disks.
#[utoipa::path(
    get,
    path = "/onboarding/disk-info",
    responses(
        (status = 200, description = "Disk information retrieved successfully", body = DiskResponse),
        (status = 500, description = "A configured path is not a valid directory"),
    )
)]
pub async fn get_disk_response(
    State(ingestion): State<IngestSettings>,
) -> Result<Json<DiskResponse>, OnboardingError> {
    let disk_info = get_disk_info(&ingestion.media_root, &ingestion.thumbnail_root)?;
    Ok(Json(disk_info))
}

/// Retrieves a sample of media files from a specified folder.
#[utoipa::path(
    get,
    path = "/onboarding/media-sample",
    params(
        ("folder" = String, Query, description = "The folder to sample media from")
    ),
    responses(
        (status = 200, description = "Media sample retrieved successfully", body = MediaSampleResponse),
        (status = 400, description = "Invalid folder path provided"),
    )
)]
pub async fn get_folder_media_sample(
    State(ingestion): State<IngestSettings>,
    Query(query): Query<FolderParams>,
) -> Result<Json<MediaSampleResponse>, OnboardingError> {
    let user_path = validate_user_folder(&ingestion.media_root, &query.folder).await?;
    let response = get_media_sample(&ingestion, &user_path)?;
    Ok(Json(response))
}

/// Scans a folder and returns a list of unsupported file types.
#[utoipa::path(
    get,
    path = "/onboarding/unsupported-files",
    params(
        ("folder" = String, Query, description = "The folder to scan for unsupported files")
    ),
    responses(
        (status = 200, description = "Unsupported files listed successfully", body = UnsupportedFilesResponse),
        (status = 400, description = "Invalid folder path provided"),
    )
)]
pub async fn get_folder_unsupported(
    State(ingestion): State<IngestSettings>,
    Query(query): Query<FolderParams>,
) -> Result<Json<UnsupportedFilesResponse>, OnboardingError> {
    let user_path = validate_user_folder(&ingestion.media_root, &query.folder).await?;
    let response = get_folder_unsupported_files(&ingestion, &user_path)?;
    Ok(Json(response))
}

/// Lists the subfolders within a given directory.
#[utoipa::path(
    get,
    path = "/onboarding/folders",
    params(
        ("folder" = String, Query, description = "The base folder to list subdirectories from")
    ),
    responses(
        (status = 200, description = "Folders listed successfully", body = Vec<String>),
        (status = 400, description = "Invalid base folder path provided"),
    )
)]
pub async fn get_folders(
    State(ingestion): State<IngestSettings>,
    Query(query): Query<FolderParams>,
) -> Result<Json<Vec<String>>, OnboardingError> {
    let folders = get_subfolders(&ingestion, &query.folder).await?;
    Ok(Json(folders))
}

/// Creates a new folder within a specified base directory.
#[utoipa::path(
    post,
    path = "/onboarding/make-folder",
    request_body = MakeFolderBody,
    responses(
        (status = 204, description = "Folder created successfully"),
        (status = 400, description = "Invalid folder name or path"),
    )
)]
pub async fn make_folder(
    State(ingestion): State<IngestSettings>,
    Json(params): Json<MakeFolderBody>,
) -> Result<StatusCode, OnboardingError> {
    create_folder(&ingestion.media_root, &params.base_folder, &params.new_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Start scanning the user folder and process the photos and videos.
///
/// # Errors
///
/// Returns a `OnboardingError` if a database connection cannot be established or the query fails.
#[utoipa::path(
    get,
    path = "/onboarding/start-processing",
    responses(
        (status = 200, description = "Processing job enqueued successfully.", body = bool),
        (status = 500, description = "Database error"),
    )
)]
pub async fn post_start_processing(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Json(payload): Json<StartProcessingBody>,
) -> Result<Json<bool>, OnboardingError> {
    start_processing(
        &context.pool,
        &context.settings,
        user.id,
        payload.user_folder,
    )
    .await?;
    Ok(Json(true))
}
