//! This module provides the core service logic for the setup process.

use crate::api::onboarding::error::OnboardingError;
use crate::api::onboarding::helpers::{check_drive_info, list_folders};
use crate::api::onboarding::interfaces::{
    DiskResponse, MediaSampleResponse, UnsupportedFilesResponse,
};
use crate::database::app_user::User;
use crate::database::jobs::JobType;
use crate::database::user_store::UserStore;
use crate::job_queue::enqueue_job;
use app_state::{AppSettings, IngestSettings, MakeRelativePath, constants, to_posix_string};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tracing::{debug, warn};
use walkdir::WalkDir;

/// Gathers information about the media and thumbnail directories.
///
/// # Errors
///
/// Returns `OnboardingError` if the configured media or thumbnail paths are not valid directories.
pub fn get_disk_info(
    media_root: &Path,
    thumbnails_root: &Path,
) -> Result<DiskResponse, OnboardingError> {
    if !media_root.is_dir() {
        return Err(OnboardingError::InvalidPath(to_posix_string(media_root)));
    }

    if !thumbnails_root.is_dir() {
        return Err(OnboardingError::InvalidPath(to_posix_string(
            thumbnails_root,
        )));
    }

    let media_folder_info = check_drive_info(media_root)?;
    let thumbnail_folder_info = check_drive_info(thumbnails_root)?;

    Ok(DiskResponse {
        media_folder: media_folder_info,
        thumbnails_folder: thumbnail_folder_info,
    })
}

/// Creates a new folder within a specified base directory.
///
/// # Errors
///
/// Returns `OnboardingError` if the folder name contains invalid characters or if an I/O error occurs.
pub async fn create_folder(
    media_root: &Path,
    base_folder: &str,
    new_name: &str,
) -> Result<(), OnboardingError> {
    if !new_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(OnboardingError::DirectoryCreation(new_name.to_string()));
    }

    let user_path = validate_user_folder(media_root, base_folder).await?;
    tokio_fs::create_dir_all(user_path.join(new_name)).await?;
    Ok(())
}

/// Lists subfolders within a given user-provided folder, returning only the folder names.
///
/// # Errors
///
/// Returns `OnboardingError` if path validation or canonicalization fails.
pub async fn get_subfolders(
    ingestion: &IngestSettings,
    folder: &str,
) -> Result<Vec<String>, OnboardingError> {
    let user_path = validate_user_folder(&ingestion.media_root, folder).await?;
    let folders = list_folders(&user_path).await?;

    folders
        .iter()
        .map(|i| i.make_relative_canon(&ingestion.media_root_canon))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|path| {
            Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
                .ok_or_else(|| OnboardingError::InvalidPath(path))
        })
        .collect()
}

/// Provides a sample of media files from a given folder.
///
/// # Errors
///
/// Returns `OnboardingError` if there's an I/O error reading the directory or its files.
pub fn get_media_sample(
    ingestion: &IngestSettings,
    user_folder: &Path,
) -> Result<MediaSampleResponse, OnboardingError> {
    let media_folder_info = check_drive_info(user_folder)?;
    let folder_relative = user_folder.make_relative_canon(&ingestion.media_root_canon)?;

    if !media_folder_info.read_access {
        return Ok(MediaSampleResponse::unreadable(folder_relative));
    }

    let n_samples = constants().onboarding_n_media_samples;
    let mut samples = Vec::with_capacity(n_samples);
    let mut photo_count = 0;
    let mut file_count = 0;

    for entry in WalkDir::new(user_folder).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        file_count += 1;

        if ingestion.is_photo_file(entry.path()) {
            photo_count += 1;
            if samples.len() < n_samples {
                samples.push(entry.into_path());
            } else {
                let random_index = fastrand::usize(..photo_count);
                if let Some(sample) = samples.get_mut(random_index) {
                    *sample = entry.into_path();
                }
            }
        }
    }

    let relative_samples = samples
        .iter()
        .map(|i| i.make_relative_canon(&ingestion.media_root_canon))
        .collect::<Result<_, _>>()?;

    Ok(MediaSampleResponse {
        read_access: true,
        folder: folder_relative,
        photo_count,
        video_count: file_count - photo_count,
        samples: relative_samples,
    })
}

/// Finds all unsupported files in a given folder.
///
/// # Errors
///
/// Returns `OnboardingError` if there is an issue reading the directory or canonicalizing file paths.
pub fn get_folder_unsupported_files(
    ingestion: &IngestSettings,
    user_folder: &Path,
) -> Result<UnsupportedFilesResponse, OnboardingError> {
    let media_folder_info = check_drive_info(user_folder)?;
    let folder_relative = user_folder.make_relative_canon(&ingestion.media_root_canon)?;

    if !media_folder_info.read_access {
        return Ok(UnsupportedFilesResponse::unreadable(folder_relative));
    }

    let mut unsupported_files: HashMap<String, Vec<String>> = HashMap::new();
    let mut inaccessible_entries = Vec::new();
    let mut unsupported_count = 0;

    for entry in WalkDir::new(user_folder) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                if let Some(path) = e.path() {
                    inaccessible_entries
                        .push(path.make_relative_canon(&ingestion.media_root_canon)?);
                }
                debug!("Skipping inaccessible entry: {}", e);
                continue;
            }
        };

        if entry.file_type().is_file() && !ingestion.is_media_file(entry.path()) {
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let relative_path = entry
                .path()
                .make_relative_canon(&ingestion.media_root_canon)?;
            unsupported_files
                .entry(ext)
                .or_default()
                .push(relative_path);
            unsupported_count += 1;
        }
    }

    Ok(UnsupportedFilesResponse {
        read_access: true,
        folder: folder_relative,
        inaccessible_entries,
        unsupported_files,
        unsupported_count,
    })
}

/// Validates that a user-provided folder path is a valid, existing directory.
///
/// # Errors
///
/// Returns `OnboardingError` if the path is invalid, not a directory, or outside the media root.
pub async fn validate_user_folder(
    media_root: &Path,
    user_folder: &str,
) -> Result<PathBuf, OnboardingError> {
    let user_path = media_root.join(user_folder);

    let canon_user_path = tokio_fs::canonicalize(&user_path).await?;
    let canon_media_root = tokio_fs::canonicalize(&media_root).await?;

    let metadata = tokio_fs::metadata(&canon_user_path).await?;
    if !metadata.is_dir() {
        warn!("User path {} is not a directory", canon_user_path.display());
        return Err(OnboardingError::InvalidPath(to_posix_string(
            &canon_user_path,
        )));
    }

    if !canon_user_path.starts_with(&canon_media_root) {
        warn!(
            "User path {} escapes media directory {}",
            canon_user_path.display(),
            canon_media_root.display()
        );
        return Err(OnboardingError::InvalidPath(to_posix_string(
            &canon_user_path,
        )));
    }

    Ok(canon_user_path)
}

/// Updates the user's media folder and triggers a system-wide media scan.
///
/// # Errors
///
/// Returns `OnboardingError` if folder validation fails, the database update fails, or the scan job cannot be enqueued.
pub async fn start_processing(
    pool: &PgPool,
    settings: &AppSettings,
    user_id: i32,
    user_folder: String,
) -> Result<User, OnboardingError> {
    let media_root = &settings.ingest.media_root;
    let user_folder = validate_user_folder(media_root, &user_folder).await?;
    let relative = user_folder.make_relative_canon(&settings.ingest.media_root_canon)?;
    let existing_folder = UserStore::get_user_media_folder(pool, user_id).await?;
    if existing_folder.is_some() {
        return Err(OnboardingError::MediaFolderAlreadySet);
    }
    let updated_user =
        UserStore::update(pool, user_id, None, None, None, None, Some(relative)).await?;

    enqueue_job::<()>(pool, settings, JobType::Scan)
        .user_id(user_id)
        .call()
        .await?;

    Ok(updated_user)
}
