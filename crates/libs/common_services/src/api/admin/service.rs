//! This module provides the core service logic for the setup process.

use crate::api::admin::error::AdminError;
use crate::api::admin::helpers::{check_drive_info, list_folders};
use crate::api::admin::interfaces::{
    AdminUserInfo, DiskResponse, MediaSampleResponse, UnsupportedFilesResponse,
};
use crate::database::app_user::User;
use crate::database::user_store::UserStore;
use crate::database::{UpdateField, UpdateUserPayload};
use crate::job_queue::enqueue_full_scan;
use app_state::{AppSettings, IngestSettings, MakeRelativePath, constants, to_posix_string};
use sqlx::{PgPool, query_scalar};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tracing::{debug, warn};
use walkdir::WalkDir;
use crate::api::album::backup_restore::backup_albums;

/// Gathers information about the media and thumbnail directories.
pub fn get_disk_info(
    media_root: &Path,
    thumbnails_root: &Path,
) -> Result<DiskResponse, AdminError> {
    if !media_root.is_dir() {
        return Err(AdminError::InvalidPath(to_posix_string(media_root)));
    }

    if !thumbnails_root.is_dir() {
        return Err(AdminError::InvalidPath(to_posix_string(thumbnails_root)));
    }

    let media_folder_info = check_drive_info(media_root)?;
    let thumbnail_folder_info = check_drive_info(thumbnails_root)?;

    Ok(DiskResponse {
        media_folder: media_folder_info,
        thumbnails_folder: thumbnail_folder_info,
    })
}

/// Creates a new folder within a specified base directory.
pub async fn create_folder(
    media_root: &Path,
    base_folder: &str,
    new_name: &str,
) -> Result<(), AdminError> {
    if !new_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AdminError::DirectoryCreation(new_name.to_string()));
    }

    let user_path = validate_user_folder(media_root, base_folder).await?;
    tokio_fs::create_dir_all(user_path.join(new_name)).await?;
    Ok(())
}

/// Lists subfolders within a given user-provided folder, returning only the folder names.
pub async fn get_subfolders(
    ingestion: &IngestSettings,
    folder: &str,
) -> Result<Vec<String>, AdminError> {
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
                .ok_or_else(|| AdminError::InvalidPath(path))
        })
        .collect()
}

/// Provides a sample of media files from a given folder.
pub fn get_media_sample(
    ingestion: &IngestSettings,
    user_folder: &Path,
) -> Result<MediaSampleResponse, AdminError> {
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
pub fn get_folder_unsupported_files(
    ingestion: &IngestSettings,
    user_folder: &Path,
) -> Result<UnsupportedFilesResponse, AdminError> {
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
pub async fn validate_user_folder(
    media_root: &Path,
    user_folder: &str,
) -> Result<PathBuf, AdminError> {
    let user_path = media_root.join(user_folder);

    let canon_user_path = tokio_fs::canonicalize(&user_path).await?;
    let canon_media_root = tokio_fs::canonicalize(&media_root).await?;

    let metadata = tokio_fs::metadata(&canon_user_path).await?;
    if !metadata.is_dir() {
        warn!("User path {} is not a directory", canon_user_path.display());
        return Err(AdminError::InvalidPath(to_posix_string(&canon_user_path)));
    }

    if !canon_user_path.starts_with(&canon_media_root) {
        warn!(
            "User path {} escapes media directory {}",
            canon_user_path.display(),
            canon_media_root.display()
        );
        return Err(AdminError::InvalidPath(to_posix_string(&canon_user_path)));
    }

    Ok(canon_user_path)
}

/// Check if folder is already in use by another user
pub async fn check_folder_in_use(
    pool: &PgPool,
    folder_relative_path: &str,
    ignore_user: Option<i32>,
) -> Result<bool, AdminError> {
    let requested = Path::new(folder_relative_path);

    let existing_user_folders = UserStore::list_users(pool)
        .await?
        .into_iter()
        .filter(|user| Some(user.id) != ignore_user)
        .filter_map(|user| user.media_folder);

    let pending_user_folders = query_scalar!("SELECT media_folder FROM user_invite")
        .fetch_all(pool)
        .await?;

    for other_folder in existing_user_folders.chain(pending_user_folders) {
        let other = Path::new(&other_folder);
        if requested.starts_with(other) || other.starts_with(requested) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Measures the recursive folder size of files on disk.
fn get_folder_size(path: &Path) -> u64 {
    if !path.exists() || !path.is_dir() {
        return 0;
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.metadata().map_or(0, |m| m.len()))
        .sum()
}

/// Identifies if the media folder and the thumbnail folder reside on the same drive.
#[must_use]
pub fn are_on_same_drive(p1: &Path, p2: &Path) -> bool {
    let p1_canon = std::fs::canonicalize(p1).unwrap_or_else(|_| p1.to_path_buf());
    let p2_canon = std::fs::canonicalize(p2).unwrap_or_else(|_| p2.to_path_buf());

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(m1), Ok(m2)) = (std::fs::metadata(&p1_canon), std::fs::metadata(&p2_canon)) {
            return m1.dev() == m2.dev();
        }
    }

    p1_canon.components().next() == p2_canon.components().next()
}

/// Retrieves a list of all users along with their parsed disk space statistics.
pub async fn list_admin_users(
    pool: &PgPool,
    settings: &AppSettings,
) -> Result<Vec<AdminUserInfo>, AdminError> {
    let db_users = UserStore::list_users(pool).await?;
    let mut admin_users = Vec::with_capacity(db_users.len());

    let media_root = &settings.ingest.media_root;

    for user in db_users {
        let main_drive_used = if let Some(ref relative_folder) = user.media_folder {
            let full_user_path = media_root.join(relative_folder);
            let path_to_walk = full_user_path.clone();
            tokio::task::spawn_blocking(move || get_folder_size(&path_to_walk)).await?
        } else {
            0
        };

        admin_users.push(AdminUserInfo {
            id: user.id,
            username: user.name,
            email: user.email,
            avatar_id: user.avatar_id,
            media_folder: user.media_folder,
            main_drive_used,
        });
    }

    Ok(admin_users)
}

/// Updates the media folder for a specific user and starts scanning jobs.
pub async fn admin_update_user_media_folder(
    pool: &PgPool,
    settings: &IngestSettings,
    target_user_id: i32,
    user_folder: &str,
) -> Result<User, AdminError> {
    let media_root = &settings.media_root;
    let user_folder_path = validate_user_folder(media_root, user_folder).await?;
    let relative = user_folder_path.make_relative_canon(&settings.media_root_canon)?;
    if check_folder_in_use(pool, &relative, Some(target_user_id)).await? {
        return Err(AdminError::FolderInUse);
    }

    let updated_user = UserStore::update(
        pool,
        target_user_id,
        UpdateUserPayload {
            name: None,
            email: None,
            password: None,
            role: None,
            media_folder: Some(relative),
            avatar_id: UpdateField::Ignore,
        },
    )
    .await?;

    sqlx::query!("DELETE FROM daily_card WHERE user_id = $1", target_user_id)
        .execute(pool)
        .await?;
    backup_albums(pool, target_user_id).await?;
    enqueue_full_scan(pool, settings, target_user_id).await?;

    Ok(updated_user)
}

/// Deletes a specific user account. Prevents an administrator from deleting themselves.
pub async fn admin_delete_user(
    pool: &PgPool,
    target_user_id: i32,
    current_user_id: i32,
) -> Result<(), AdminError> {
    if target_user_id == current_user_id {
        return Err(AdminError::CannotDeleteSelf);
    }

    UserStore::delete(pool, target_user_id).await?;
    Ok(())
}
