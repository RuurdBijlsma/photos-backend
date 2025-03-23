use crate::common::image_utils::{is_photo_file, is_video_file};
use derive_more::Constructor;
use fs2::available_space;
use fs2::total_space;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::error;
use walkdir::WalkDir;

/// Custom error type for media directory operations.
#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Invalid media directory: {0}")]
    InvalidMediaDir(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] io::Error),

    #[error("Path conversion error for path: {0}")]
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

/// Response structure for file count and sample paths.
#[derive(Constructor, Serialize)]
pub struct FileCountResponse {
    count: usize,
    samples: Vec<String>,
    unsupported_files: HashMap<String, Vec<String>>,
    unsupported_count: usize,
    media_folder: PathInfoResponse,
    thumbnails_folder: PathInfoResponse,
}

/// Processes a media directory by counting photo/video files and collecting up to 10 random samples.
///
/// Uses reservoir sampling to maintain a fixed-size sample set.
/// Returns a `FileCountResponse` containing the total count and relative file paths.
///
/// # Errors
///
/// Returns a `MediaError` if there is a filesystem issue or if a path cannot be converted to a UTF-8 string.
pub fn summarize_folders(
    media_path: &Path,
    thumbnail_path: &Path,
) -> Result<FileCountResponse, MediaError> {
    let mut unsupported_count = 0;
    let mut count = 0;
    let mut unsupported_files: HashMap<String, Vec<String>> = HashMap::new();
    let mut samples = Vec::with_capacity(10);
    let media_path_buf = PathBuf::from(media_path);

    for entry in WalkDir::new(media_path).into_iter().filter_map(|e| {
        match e {
            Ok(entry) => Some(entry),
            Err(e) => {
                // Convert walkdir::Error to std::io::Error and log the error.
                let io_error = e
                    .into_io_error()
                    .unwrap_or_else(|| std::io::Error::other("walkdir error"));
                error!("Directory walk error: {}", io_error);
                None
            }
        }
    }) {
        if !entry.file_type().is_file()
            || (!is_photo_file(entry.path()) && !is_video_file(entry.path()))
        {
            if entry.file_type().is_file() {
                unsupported_count += 1;
                if let Some(ext) = entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(String::from)
                {
                    if !unsupported_files.contains_key(&ext) {
                        unsupported_files.insert(ext.clone(), Vec::new());
                    }
                    if let Some(val) = unsupported_files.get_mut(&ext) {
                        let relative_path =
                            entry.path().strip_prefix(&media_path_buf).map_err(|_| {
                                MediaError::PathConversion(to_posix_string(entry.path()))
                            })?;
                        val.push(to_posix_string(relative_path));
                    }
                }
            }
            continue;
        }

        count += 1;

        // for the first 10 files, just push. After that, replace a random element.
        if count <= 10 {
            samples.push(entry);
        } else {
            let random_index = fastrand::usize(0..count);
            if random_index < 10 {
                samples[random_index] = entry;
            }
        }
    }

    // Convert absolute paths to paths relative to the media directory.
    let relative_samples: Vec<String> = samples
        .into_iter()
        .map(|entry| {
            entry
                .path()
                .strip_prefix(&media_path_buf)
                .map_err(|_| MediaError::PathConversion(to_posix_string(entry.path())))?
                .to_str()
                .ok_or_else(|| MediaError::PathConversion(to_posix_string(entry.path())))
                .map(String::from)
        })
        .collect::<Result<_, _>>()?;

    let media_folder_info = check_folder_info(media_path)?;
    let thumbnail_folder_info = check_folder_info(thumbnail_path)?;

    Ok(FileCountResponse::new(
        count,
        relative_samples,
        unsupported_files,
        unsupported_count,
        media_folder_info,
        thumbnail_folder_info,
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
pub fn check_folder_info(folder: &Path) -> Result<PathInfoResponse, MediaError> {
    let total = total_space(folder)?;
    let available = available_space(folder)?;
    let used = total - available;
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

fn to_posix_string(path: &std::path::Path) -> String {
    path.to_str().unwrap_or_default().replace('\\', "/")
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

    // Check write access by trying to create and delete a temporary file
    let tmp_file_path = path.join(".tmp_access_test");
    let can_write = File::create(&tmp_file_path)
        .and_then(|mut file| file.write_all(b"test")) // Try writing some data
        .and_then(|()| fs::remove_file(&tmp_file_path)) // Try deleting it
        .is_ok();

    Ok((can_read, can_write))
}
