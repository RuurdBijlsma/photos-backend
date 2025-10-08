use crate::{get_config, get_media_dir};
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::path::absolute;
use std::path::Path;

#[must_use]
pub fn to_posix_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Get the relative path string for a given file.
/// # Errors
///
/// * `absolute` can return an error if the path cannot be resolved.
/// * `strip_prefix` can return an error if the media directory is not a prefix of the file's absolute path.
pub fn get_relative_path_str(file: impl AsRef<Path>) -> color_eyre::Result<String> {
    let file_abs = absolute(file)?;
    let relative_path = file_abs.strip_prefix(get_media_dir())?;
    let relative_path_str = to_posix_string(relative_path);
    Ok(relative_path_str)
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
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    Ok(pool)
}

#[must_use]
pub fn is_media_file(file: &Path) -> bool {
    let config = &get_config().thumbnail_generation;
    let photo_extensions = &config.photo_extensions;
    let video_extensions = &config.video_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    photo_extensions.contains(&extension) || video_extensions.contains(&extension)
}

#[must_use]
pub fn is_photo_file(file: &Path) -> bool {
    let photo_extensions = &get_config().thumbnail_generation.photo_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    photo_extensions.contains(&extension)
}

#[must_use]
pub fn is_video_file(file: &Path) -> bool {
    let video_extensions = &get_config().thumbnail_generation.video_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    video_extensions.contains(&extension)
}
