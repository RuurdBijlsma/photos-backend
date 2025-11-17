use crate::api::onboarding::error::OnboardingError;
use crate::api::onboarding::interfaces::PathInfoResponse;
use fs2::{available_space, total_space};
use std::path::{Path, PathBuf};
use std::{fs, io};
use tempfile::NamedTempFile;
use tokio::task;
use app_state::to_posix_string;

pub fn check_drive_info(folder: &Path) -> Result<PathInfoResponse, OnboardingError> {
    let total = total_space(folder)?;
    let available = available_space(folder)?;
    let (read, write) = check_read_write_access(folder);

    Ok(PathInfoResponse {
        folder: to_posix_string(folder),
        disk_available: available,
        disk_used: total.saturating_sub(available),
        disk_total: total,
        read_access: read,
        write_access: write,
    })
}

#[must_use]
pub fn check_read_write_access(path: &Path) -> (bool, bool) {
    let can_read = fs::read_dir(path).is_ok();
    let can_write = NamedTempFile::new_in(path).is_ok();
    (can_read, can_write)
}

pub async fn list_folders(user_folder: &Path) -> Result<Vec<PathBuf>, OnboardingError> {
    let path_buf = user_folder.to_path_buf();
    task::spawn_blocking(move || {
        fs::read_dir(path_buf)?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_dir()))
            .map(|entry| Ok(entry.path()))
            .collect::<Result<Vec<_>, io::Error>>()
    })
    .await?
    .map_err(OnboardingError::from)
}
