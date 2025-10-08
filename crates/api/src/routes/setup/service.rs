use crate::routes::setup::error::SetupError;
use crate::routes::setup::interfaces::{
    DiskResponse, MediaSampleResponse, PathInfoResponse, UnsupportedFilesResponse,
};
use common_photos::{
    canon_relative_path, get_config, get_media_dir, is_photo_file, is_video_file, to_posix_string,
};
use fs2::available_space;
use fs2::total_space;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::task;
use tracing::{debug, warn};
use walkdir::WalkDir;

pub fn get_media_sample(user_folder: &Path) -> Result<MediaSampleResponse, SetupError> {
    let mut count = 0;
    let mut photo_count = 0;
    let n_samples = get_config().setup.n_media_samples;
    let mut samples = Vec::with_capacity(n_samples);

    let media_folder_info = check_drive_info(user_folder)?;
    let folder_relative = canon_relative_path(user_folder)?;

    if !media_folder_info.read_access {
        return Ok(MediaSampleResponse {
            read_access: false,
            folder: folder_relative,
            photo_count: 0,
            video_count: 0,
            samples: Vec::new(),
        });
    }

    for entry in WalkDir::new(user_folder).into_iter().filter_map(Result::ok) {
        let is_photo_path = is_photo_file(entry.path());
        let is_file = entry.file_type().is_file();

        if is_file {
            count += 1;
        }

        if is_file && is_photo_path {
            photo_count += 1;
            if photo_count <= n_samples {
                samples.push(entry.into_path());
            } else {
                let random_index = fastrand::usize(0..photo_count);
                if random_index < n_samples {
                    samples[random_index] = entry.into_path();
                }
            }
        }
    }

    let relative_samples: Vec<String> = samples
        .into_iter()
        .map(|path| canon_relative_path(&path))
        .collect::<Result<_, _>>()?;

    Ok(MediaSampleResponse {
        read_access: true,
        folder: folder_relative,
        photo_count,
        video_count: count - photo_count,
        samples: relative_samples,
    })
}

/// Get info about the media and thumbnail dirs.
pub fn validate_disks(
    media_path: &Path,
    thumbnail_path: &Path,
) -> Result<DiskResponse, SetupError> {
    let media_folder_info = check_drive_info(media_path)?;
    let thumbnail_folder_info = check_drive_info(thumbnail_path)?;

    Ok(DiskResponse {
        media_folder: media_folder_info,
        thumbnails_folder: thumbnail_folder_info,
    })
}

pub fn check_drive_info(folder: &Path) -> Result<PathInfoResponse, SetupError> {
    let total = total_space(folder)?;
    let available = available_space(folder)?;
    let used = total.saturating_sub(available);
    let (read, write) = check_read_write_access(folder)?;

    Ok(PathInfoResponse {
        folder: to_posix_string(folder),
        disk_available: available,
        disk_used: used,
        disk_total: total,
        read_access: read,
        write_access: write,
    })
}

fn check_read_write_access(path: &Path) -> Result<(bool, bool), io::Error> {
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    let can_read = fs::read_dir(path).is_ok();
    let can_write = NamedTempFile::new_in(path)
        .and_then(|mut file| file.write_all(b"test"))
        .is_ok();

    Ok((can_read, can_write))
}

pub async fn list_folders(user_folder: &Path) -> Result<Vec<PathBuf>, SetupError> {
    let user_folder = user_folder.to_path_buf();
    Ok(task::spawn_blocking(move || list_folders_sync(&user_folder)).await??)
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

pub async fn validate_user_folder(user_folder: &str) -> Result<PathBuf, SetupError> {
    let media_path = get_media_dir().canonicalize()?;
    let user_path = media_path.join(user_folder).canonicalize()?;

    if !user_path.starts_with(&media_path) {
        warn!(
            "User path {} escapes media directory {}",
            user_path.display(),
            media_path.display()
        );
        return Err(SetupError::InvalidPath(to_posix_string(&user_path)));
    }

    if !user_path.is_dir() {
        warn!("User path {} is not a directory", user_path.display());
        return Err(SetupError::InvalidPath(to_posix_string(&user_path)));
    }

    Ok(user_path)
}

pub fn contains_non_alphanumeric(s: &str) -> bool {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.is_match(s)
}

pub fn get_folder_unsupported_files(
    user_folder: &Path,
) -> Result<UnsupportedFilesResponse, SetupError> {
    let mut unsupported_count = 0;
    let mut unsupported_files: HashMap<String, Vec<String>> = HashMap::new();
    let mut inaccessible_entries = Vec::new();

    let media_folder_info = check_drive_info(user_folder)?;

    let folder_relative = canon_relative_path(user_folder)?;

    if !media_folder_info.read_access {
        return Ok(UnsupportedFilesResponse {
            read_access: false,
            folder: folder_relative,
            inaccessible_entries: Vec::new(),
            unsupported_files,
            unsupported_count: 0,
        });
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
        let is_video_file = is_video_file(entry.path());
        if entry.file_type().is_file() && !is_photo_path && !is_video_file {
            unsupported_count += 1;
            let ext = entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map_or_else(String::new, String::from);
            let relative_path = canon_relative_path(entry.path())?;
            unsupported_files
                .entry(ext)
                .or_default()
                .push(relative_path);
        }
    }

    let inaccessible_entries_str: Vec<String> = inaccessible_entries
        .into_iter()
        .map(|entry| canon_relative_path(&entry))
        .collect::<Result<_, _>>()?;

    Ok(UnsupportedFilesResponse {
        read_access: true,
        folder: folder_relative,
        inaccessible_entries: inaccessible_entries_str,
        unsupported_files,
        unsupported_count,
    })
}
