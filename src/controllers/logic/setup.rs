use crate::common::image_utils::{is_photo_file, is_video_file};
use derive_more::Constructor;
use fs2::available_space;
use fs2::total_space;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::task;
use tracing::{debug, error, warn};
use walkdir::WalkDir;

/// Custom error type for media directory operations.
#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Invalid media directory: {0}")]
    InvalidMediaDir(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] io::Error),

    #[error("Path conversion error: {0}")]
    PathConversion(String),
}

#[derive(Constructor, Serialize)]
pub struct PathInfoResponse {
    folder: String,
    disk_available: u64,
    disk_used: u64,
    disk_total: u64,
    read_access: bool,
    write_access: bool,
}

#[derive(Constructor, Serialize)]
pub struct MediaSampleResponse {
    read_access: bool,
    folder: String,
    photo_count: usize,
    video_count: usize,
    samples: Vec<String>,
}

#[derive(Constructor, Serialize)]
pub struct UnsupportedFilesResponse {
    read_access: bool,
    folder: String,
    inaccessible_entries: Vec<String>,
    unsupported_files: HashMap<String, Vec<String>>,
    unsupported_count: usize,
}

#[derive(Constructor, Serialize)]
pub struct DiskResponse {
    media_folder: PathInfoResponse,
    thumbnails_folder: PathInfoResponse,
}

/// Get info about the media and thumbnail dirs.
///
/// # Errors
///
/// Returns a `MediaError` if there is a filesystem issue or if a path cannot be converted to a UTF-8 string.
pub fn validate_disks(
    media_path: &Path,
    thumbnail_path: &Path,
) -> Result<DiskResponse, MediaError> {
    let media_folder_info = check_drive_info(media_path)?;
    let thumbnail_folder_info = check_drive_info(thumbnail_path)?;

    Ok(DiskResponse::new(media_folder_info, thumbnail_folder_info))
}

const N_SAMPLES: usize = 8;

pub fn get_media_sample(
    media_path: &Path,
    user_folder: &Path,
) -> Result<MediaSampleResponse, MediaError> {
    let mut count = 0;
    let mut photo_count = 0;
    let mut samples = Vec::with_capacity(N_SAMPLES);

    let media_folder_info = check_drive_info(user_folder)?;

    let folder_relative = relative_path(media_path, user_folder)
        .map_err(|_| MediaError::PathConversion(to_posix_string(user_folder)))?;

    if !media_folder_info.read_access {
        return Ok(MediaSampleResponse::new(
            false,
            folder_relative,
            0,
            0,
            Vec::new(),
        ));
    }

    for entry in WalkDir::new(user_folder)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(_) => None,
        })
    {
        let is_photo_path = is_photo_file(entry.path());
        let is_file = entry.file_type().is_file();

        if is_file {
            count += 1;
        }

        if is_file && is_photo_path {
            photo_count += 1;
            // for the first N files, just push. After that, replace a random element.
            if photo_count <= N_SAMPLES {
                samples.push(entry);
            } else {
                let random_index = fastrand::usize(0..photo_count);
                if random_index < N_SAMPLES {
                    samples[random_index] = entry;
                }
            }
        }
    }

    // Convert absolute paths to paths relative to the media directory.
    let relative_samples: Vec<String> = samples
        .into_iter()
        .map(|entry| relative_path(media_path, entry.path()))
        .collect::<Result<_, _>>()?;

    Ok(MediaSampleResponse::new(
        true,
        folder_relative,
        photo_count,
        count - photo_count,
        relative_samples,
    ))
}

pub fn get_folder_unsupported_files(
    media_path: &Path,
    user_folder: &Path,
) -> Result<UnsupportedFilesResponse, MediaError> {
    let mut unsupported_count = 0;
    let mut unsupported_files: HashMap<String, Vec<String>> = HashMap::new();
    let mut inaccessible_entries = Vec::new();

    let media_folder_info = check_drive_info(user_folder)?;

    let folder_relative = relative_path(media_path, user_folder)
        .map_err(|_| MediaError::PathConversion(to_posix_string(user_folder)))?;

    if !media_folder_info.read_access {
        return Ok(UnsupportedFilesResponse::new(
            false,
            folder_relative,
            Vec::new(),
            unsupported_files,
            0,
        ));
    }

    for entry in WalkDir::new(user_folder)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(e) => {
                if let Some(path) = e.path() {
                    let owned_path = path.to_path_buf();
                    inaccessible_entries.push(owned_path);
                }
                debug!("Skipping inaccessible entry: {}", e);
                None
            }
        })
    {
        let is_photo_path = is_photo_file(entry.path());
        let is_file = entry.file_type().is_file();
        if !is_file || (!is_photo_path && !is_video_file(entry.path())) {
            if entry.file_type().is_file() {
                unsupported_count += 1;
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(String::from)
                    .unwrap_or_else(String::new);
                let relative_path = relative_path(media_path, entry.path())?;
                unsupported_files
                    .entry(ext)
                    .or_default()
                    .push(relative_path);
            }
            continue;
        }
    }

    // Convert inaccesible entries to relative path strings
    let inaccessible_entries_str: Vec<String> = inaccessible_entries
        .into_iter()
        .map(|entry| relative_path(media_path, &entry))
        .collect::<Result<_, _>>()?;

    Ok(UnsupportedFilesResponse::new(
        true,
        folder_relative,
        inaccessible_entries_str,
        unsupported_files,
        unsupported_count,
    ))
}

/// Retrieves storage and access information for a given folder.
///
/// This function gathers details about the total, used, and available storage
/// for the specified folder, as well as its read and write permissions.
///
/// # Arguments
///
/// * `folder` - A reference to the folder path.
///
/// # Returns
///
/// * `Ok(PathInfoResponse)` containing:
///   - `available`: The available storage space in bytes.
///   - `used`: The used storage space in bytes.
///   - `total`: The total storage space in bytes.
///   - `read`: Whether the folder is readable.
///   - `write`: Whether the folder is writable.
///
/// # Errors
///
/// This function returns an `Err(MediaError)` if:
/// * Retrieving the total or available storage space fails.
/// * Checking read/write permissions encounters an error.
/// ```
pub fn check_drive_info(folder: &Path) -> Result<PathInfoResponse, MediaError> {
    let total = total_space(folder)?;
    let available = available_space(folder)?;
    let used = total.saturating_sub(available);
    let (read, write) = check_read_write_access(folder)?;

    Ok(PathInfoResponse::new(
        to_posix_string(folder),
        available,
        used,
        total,
        read,
        write,
    ))
}

fn relative_path(base: &Path, path: &Path) -> Result<String, MediaError> {
    let stripped = path.strip_prefix(base).map_err(|e| {
        MediaError::PathConversion(format!(
            "{} (base: {}): {}",
            to_posix_string(path),
            to_posix_string(base),
            e
        ))
    })?;
    Ok(to_posix_string(stripped))
}

pub fn to_posix_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Checks whether the given folder has both read and write access.
///
/// This function attempts to:
/// 1. Verify if the folder can be read by checking the ability to list its contents.
/// 2. Check if the folder is writable by creating, writing, and deleting a temporary file.
///
/// # Arguments
///
/// * `path` - A reference to the folder path.
///
/// # Returns
///
/// * `Ok((bool, bool))`:
///   - The first boolean indicates if the folder is readable (`true` if readable, `false` otherwise).
///   - The second boolean indicates if the folder is writable (`true` if writable, `false` otherwise).
///
/// # Errors
///
/// This function returns an `Err(io::Error)` if:
/// * The path provided is not a directory (checked with `path.is_dir()`).
/// * The read access check fails (due to insufficient permissions, for example).
/// * The write access check fails (due to permissions, full disk, or other errors).
///
/// The errors are propagated from the `fs::read_dir` and file creation/removal operations.
fn check_read_write_access(path: &Path) -> Result<(bool, bool), io::Error> {
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    // Check read access
    let can_read = fs::read_dir(path).is_ok();

    // Check write access by trying to write to a temporary file
    let can_write = NamedTempFile::new_in(path)
        .and_then(|mut file| file.write_all(b"test"))
        .is_ok();

    Ok((can_read, can_write))
}

pub async fn list_folders(user_folder: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let user_folder = user_folder.to_path_buf();
    task::spawn_blocking(move || list_folders_sync(&user_folder)).await?
}

pub fn list_folders_sync(user_folder: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut folders = Vec::new();
    for entry in fs::read_dir(user_folder)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            folders.push(entry.path());
        }
    }
    Ok(folders)
}

pub async fn validate_media_and_user_directory(
    media_dir: &str,
    user_folder: &Path,
) -> Result<(PathBuf, PathBuf), ()> {
    // Canonicalize the media directory to get an absolute path.
    let media_path = tokio::fs::canonicalize(media_dir).await.map_err(|e| {
        warn!(
            "Failed to canonicalize media directory {}: {}",
            media_dir, e
        );
    })?;

    if !media_path.is_dir() {
        warn!("Media path {} is not a directory", media_path.display());
        return Err(());
    }

    // Resolve the user's directory within the media directory.
    let user_path = tokio::fs::canonicalize(media_path.join(user_folder))
        .await
        .map_err(|e| {
            warn!(
                "Failed to canonicalize user directory {}: {}",
                media_path.join(user_folder).display(),
                e
            );
            ()
        })?;

    // Ensure that the resolved user_path is inside the media_path.
    if user_path.strip_prefix(&media_path).is_err() {
        warn!(
            "User path {} escapes media directory {}",
            user_path.display(),
            media_path.display()
        );
        return Err(());
    }

    if !user_path.is_dir() {
        warn!("User path {} is not a directory", user_path.display());
        return Err(());
    }

    Ok((media_path, user_path))
}

pub fn contains_non_alphanumeric(s: &str) -> bool {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.is_match(s)
}
