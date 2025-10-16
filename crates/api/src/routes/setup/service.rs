use crate::routes::setup::error::SetupError;
use crate::routes::setup::helpers::{check_drive_info, list_folders};
use crate::routes::setup::interfaces::{
    DiskResponse, MediaSampleResponse, UnsupportedFilesResponse,
};
use common_photos::{
    is_media_file, is_photo_file, media_dir, relative_path_canon, settings, thumbnails_dir,
    to_posix_string,
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::fs as tokio_fs;
use tracing::{debug, warn};
use walkdir::WalkDir;

static WELCOME_NEEDED: AtomicBool = AtomicBool::new(true);

/// Checks if the initial setup is required by checking for any admin users.
pub async fn is_welcome_needed(pool: &PgPool) -> Result<bool, SetupError> {
    if !WELCOME_NEEDED.load(Ordering::Relaxed) {
        return Ok(false);
    }

    let user_option = sqlx::query_scalar!(r"SELECT 1 FROM app_user LIMIT 1")
        .fetch_optional(pool)
        .await?
        .flatten();

    if user_option.is_some() {
        WELCOME_NEEDED.store(false, Ordering::Relaxed);
        return Ok(false);
    }
    Ok(true)
}

/// Gathers information about the media and thumbnail directories.
pub fn get_disk_info() -> Result<DiskResponse, SetupError> {
    let media_path = media_dir();
    if !media_path.is_dir() {
        return Err(SetupError::InvalidPath(to_posix_string(media_path)));
    }

    let thumbnail_path = thumbnails_dir();
    if !thumbnail_path.is_dir() {
        return Err(SetupError::InvalidPath(to_posix_string(thumbnail_path)));
    }

    let media_folder_info = check_drive_info(media_path)?;
    let thumbnail_folder_info = check_drive_info(thumbnail_path)?;

    Ok(DiskResponse {
        media_folder: media_folder_info,
        thumbnails_folder: thumbnail_folder_info,
    })
}

/// Creates a new folder within a specified base directory.
pub async fn create_folder(base_folder: &str, new_name: &str) -> Result<(), SetupError> {
    if !new_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(SetupError::DirectoryCreation(new_name.to_string()));
    }

    let user_path = validate_user_folder(base_folder).await?;
    tokio_fs::create_dir_all(user_path.join(new_name)).await?;
    Ok(())
}

/// Lists subfolders within a given user-provided folder.
pub async fn get_subfolders(folder: &str) -> Result<Vec<String>, SetupError> {
    let user_path = validate_user_folder(folder).await?;
    let folders = list_folders(&user_path).await?;
    folders
        .iter()
        .map(relative_path_canon)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Provides a sample of media files from a given folder.
pub fn get_media_sample(user_folder: &Path) -> Result<MediaSampleResponse, SetupError> {
    let media_folder_info = check_drive_info(user_folder)?;
    let folder_relative = relative_path_canon(user_folder)?;

    if !media_folder_info.read_access {
        return Ok(MediaSampleResponse::unreadable(folder_relative));
    }

    let n_samples = settings().setup.n_media_samples;
    let mut samples = Vec::with_capacity(n_samples);
    let mut photo_count = 0;
    let mut file_count = 0;

    for entry in WalkDir::new(user_folder).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        file_count += 1;

        if is_photo_file(entry.path()) {
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
        .map(relative_path_canon)
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
pub fn get_folder_unsupported_files(
    user_folder: &Path,
) -> Result<UnsupportedFilesResponse, SetupError> {
    let media_folder_info = check_drive_info(user_folder)?;
    let folder_relative = relative_path_canon(user_folder)?;

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
                    inaccessible_entries.push(relative_path_canon(path)?);
                }
                debug!("Skipping inaccessible entry: {}", e);
                continue;
            }
        };

        if entry.file_type().is_file() && !is_media_file(entry.path()) {
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let relative_path = relative_path_canon(entry.path())?;
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

/// Validates that a user-provided folder path is a valid, existing directory
/// within the configured media directory to prevent path traversal attacks.
pub async fn validate_user_folder(user_folder: &str) -> Result<PathBuf, SetupError> {
    let media_path = media_dir();
    let user_path = media_path.join(user_folder);

    let canonical_user_path = tokio_fs::canonicalize(&user_path).await?;
    let canonical_media_path = tokio_fs::canonicalize(&media_path).await?;

    let metadata = tokio_fs::metadata(&canonical_user_path).await?;
    if !metadata.is_dir() {
        warn!(
            "User path {} is not a directory",
            canonical_user_path.display()
        );
        return Err(SetupError::InvalidPath(to_posix_string(
            &canonical_user_path,
        )));
    }

    if !canonical_user_path.starts_with(&canonical_media_path) {
        warn!(
            "User path {} escapes media directory {}",
            canonical_user_path.display(),
            canonical_media_path.display()
        );
        return Err(SetupError::InvalidPath(to_posix_string(
            &canonical_user_path,
        )));
    }

    Ok(canonical_user_path)
}
