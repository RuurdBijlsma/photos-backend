use crate::{canon_media_dir, media_dir, settings};
use sqlx::{PgPool, Pool, Postgres};
use std::fs::canonicalize;
use std::path::Path;
use std::path::absolute;

#[must_use]
pub fn to_posix_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Get the relative path string for a given file.
/// Can be used if the `file` may not exist.
/// # Errors
///
/// * `absolute` can return an error if the path cannot be resolved.
/// * `strip_prefix` can return an error if the media directory is not a prefix of the file's absolute path.
pub fn relative_path_no_exist(file: impl AsRef<Path>) -> color_eyre::Result<String> {
    let file_abs = absolute(file)?;
    let relative_path = file_abs.strip_prefix(media_dir())?;
    let relative_path_str = to_posix_string(relative_path);
    Ok(relative_path_str)
}

/// Get the relative path string for a given file, canonicalizes the file and media dir first.
/// Can only be used if the `file` exists.
/// # Errors
///
/// * `canonicalize` can return an error if the `file` cannot be resolved.
/// * `canonicalize` can return an error if the `media_dir` cannot be resolved.
/// * `strip_prefix` can return an error if the media directory is not a prefix of the file's canonicalized path.
pub fn relative_path_exists(file: impl AsRef<Path>) -> color_eyre::Result<String> {
    let file = canonicalize(file)?;
    let relative_path = file.strip_prefix(canon_media_dir())?;
    Ok(to_posix_string(relative_path))
}

/// Generate a URL-safe random ID of a given length.
#[must_use]
pub fn nice_id(length: usize) -> String {
    const URL_SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    (0..length)
        .map(|_| {
            let idx = rand::random_range(0..URL_SAFE.len());
            URL_SAFE[idx] as char
        })
        .collect()
}

/// Run migrations and get a database connection pool.
/// # Errors
///
/// * `env::var` can return an error if `DATABASE_URL` is not found in the environment.
/// * `PgPool::connect` can return an error if the database connection fails.
/// * `sqlx::migrate` can return an error if migrations fail.
pub async fn get_db_pool() -> color_eyre::Result<Pool<Postgres>> {
    dotenv::from_path(".env").ok();
    let database_url = &settings().database.url;
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    Ok(pool)
}

#[must_use]
pub fn is_media_file(file: &Path) -> bool {
    let photo_extensions = &settings().thumbnail_generation.photo_extensions;
    let video_extensions = &settings().thumbnail_generation.video_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    photo_extensions.contains(&extension) || video_extensions.contains(&extension)
}

#[must_use]
pub fn is_photo_file(file: &Path) -> bool {
    let photo_extensions = &settings().thumbnail_generation.photo_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    photo_extensions.contains(&extension)
}

#[must_use]
pub fn is_video_file(file: &Path) -> bool {
    let video_extensions = &settings().thumbnail_generation.video_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    video_extensions.contains(&extension)
}
