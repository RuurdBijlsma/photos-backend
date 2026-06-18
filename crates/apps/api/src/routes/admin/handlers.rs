//! This module defines the HTTP handlers for the initial application onboarding process.

use crate::api_state::ApiContext;
use app_state::IngestSettings;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use common_services::api::admin::error::AdminError;
use common_services::api::admin::interfaces::{
    DiskResponse, FolderParams, MakeFolderBody, MediaSampleResponse, StartProcessingBody,
    UnsupportedFilesResponse,
};
use common_services::api::admin::service::{
    create_folder, get_disk_info, get_folder_unsupported_files, get_media_sample, get_subfolders,
    start_processing, validate_user_folder,
};
use common_services::database::app_user::User;

/// Retrieves information about the configured media and thumbnail disks.
pub async fn get_disk_response(
    State(ingestion): State<IngestSettings>,
) -> Result<Json<DiskResponse>, AdminError> {
    let disk_info = get_disk_info(&ingestion.media_root, &ingestion.thumbnail_root)?;
    Ok(Json(disk_info))
}

/// Retrieves a sample of media files from a specified folder.
pub async fn get_folder_media_sample(
    State(ingestion): State<IngestSettings>,
    Query(query): Query<FolderParams>,
) -> Result<Json<MediaSampleResponse>, AdminError> {
    let user_path = validate_user_folder(&ingestion.media_root, &query.folder).await?;
    let response = get_media_sample(&ingestion, &user_path)?;
    Ok(Json(response))
}

/// Scans a folder and returns a list of unsupported file types.
pub async fn get_folder_unsupported(
    State(ingestion): State<IngestSettings>,
    Query(query): Query<FolderParams>,
) -> Result<Json<UnsupportedFilesResponse>, AdminError> {
    let user_path = validate_user_folder(&ingestion.media_root, &query.folder).await?;
    let response = get_folder_unsupported_files(&ingestion, &user_path)?;
    Ok(Json(response))
}

/// Lists the subfolders within a given directory.
pub async fn get_folders(
    State(ingestion): State<IngestSettings>,
    Query(query): Query<FolderParams>,
) -> Result<Json<Vec<String>>, AdminError> {
    let folders = get_subfolders(&ingestion, &query.folder).await?;
    Ok(Json(folders))
}

/// Creates a new folder within a specified base directory.
pub async fn make_folder(
    State(ingestion): State<IngestSettings>,
    Json(params): Json<MakeFolderBody>,
) -> Result<StatusCode, AdminError> {
    create_folder(&ingestion.media_root, &params.base_folder, &params.new_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Start scanning the user folder and process the photos and videos.
///
/// # Errors
///
/// Returns a `OnboardingError` if a database connection cannot be established or the query fails.
pub async fn post_start_processing(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Json(payload): Json<StartProcessingBody>,
) -> Result<Json<bool>, AdminError> {
    start_processing(
        &context.pool,
        &context.settings,
        user.id,
        payload.user_folder,
    )
    .await?;
    Ok(Json(true))
}
