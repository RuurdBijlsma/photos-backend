use crate::{canon_media_dir, media_dir, settings, thumbnails_dir, ThumbOptions};
use color_eyre::eyre::eyre;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, Pool, Postgres};
use std::fs::canonicalize;
use std::path::Path;
use std::path::absolute;
use std::time::Duration;
use tracing::info;

/// Converts a path to a POSIX-style string, replacing backslashes with forward slashes.
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
pub fn relative_path_abs(file: impl AsRef<Path>) -> color_eyre::Result<String> {
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
pub fn relative_path_canon(file: impl AsRef<Path>) -> color_eyre::Result<String> {
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
    let db_settings = &settings().database;
    let database_url = &db_settings.url;
    info!("Connecting to database.");
    let pool = PgPoolOptions::new()
        .max_connections(db_settings.max_connections)
        .min_connections(db_settings.min_connection)
        .max_lifetime(Duration::from_secs(db_settings.max_lifetime))
        .idle_timeout(Duration::from_secs(db_settings.idle_timeout))
        .acquire_timeout(Duration::from_secs(db_settings.acquire_timeout))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Checks if a file is a media file based on its extension.
#[must_use]
pub fn is_media_file(file: &Path) -> bool {
    let photo_extensions = &settings().thumbnail_generation.photo_extensions;
    let video_extensions = &settings().thumbnail_generation.video_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    photo_extensions.contains(&extension) || video_extensions.contains(&extension)
}

/// Checks if a file is a photo file based on its extension.
#[must_use]
pub fn is_photo_file(file: &Path) -> bool {
    let photo_extensions = &settings().thumbnail_generation.photo_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    photo_extensions.contains(&extension)
}

/// Checks if a file is a video file based on its extension.
#[must_use]
pub fn is_video_file(file: &Path) -> bool {
    let video_extensions = &settings().thumbnail_generation.video_extensions;
    let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
        return false;
    };
    video_extensions.contains(&extension)
}

/// Derives the user ID from a given relative path by extracting the username and querying the database.
/// # Errors
///
/// * If the username cannot be extracted from the path.
/// * If the database query to find the user by username fails.
/// * If no user is found for the extracted username.
pub async fn user_id_from_relative_path<'c, E>(
    relative_path: &str,
    executor: E,
) -> color_eyre::Result<i32>
where
    E: Executor<'c, Database = Postgres>,
{
    let file = media_dir().join(relative_path);
    let Some(username) = username_from_path(&file) else {
        return Err(eyre!("Can't get username from path {}", relative_path));
    };
    let Ok(user_id) = user_id_from_username(&username, executor).await else {
        return Err(eyre!(
            "Error getting user from database for username = '{}'",
            username
        ));
    };
    let Some(user_id) = user_id else {
        return Err(eyre!("Can't find user id for username '{}'", username));
    };
    Ok(user_id)
}

/// Extracts the username from the first component of a file's relative path.
fn username_from_path(path: &Path) -> Option<String> {
    let relative_path = relative_path_abs(path).ok()?;
    relative_path
        .split('/')
        .next()
        .map(std::string::ToString::to_string)
}

/// Retrieves a user's ID from the database based on their username.
/// # Errors
///
/// * If the database query fails.
async fn user_id_from_username<'c, E>(
    username: &str,
    executor: E,
) -> color_eyre::Result<Option<i32>>
where
    E: Executor<'c, Database = Postgres>,
{
    let user_id = sqlx::query_scalar!("SELECT id FROM app_user WHERE name = $1", username)
        .fetch_optional(executor)
        .await?;
    Ok(user_id)
}

/// Constructs thumbnail generation options from the application settings.
#[must_use]
pub fn get_thumb_options() -> ThumbOptions {
    let thumb_gen_config = &settings().thumbnail_generation;
    ThumbOptions {
        video_options: thumb_gen_config.video_options.clone(),
        avif_options: thumb_gen_config.avif_options.clone(),
        heights: thumb_gen_config.heights.clone(),
        thumbnail_extension: thumb_gen_config.thumbnail_extension.clone(),
        photo_extensions: thumb_gen_config.photo_extensions.clone(),
        video_extensions: thumb_gen_config.video_extensions.clone(),
        skip_if_exists: true,
    }
}

/// Checks if a file has already been ingested by verifying its database record and thumbnail existence.
/// # Errors
///
/// * Can return an error from `thumbs_exist` if checking for thumbnails fails.
pub async fn file_is_ingested<'c, E>(file: &Path, executor: E) -> color_eyre::Result<bool>
where
    E: Executor<'c, Database = Postgres>,
{
    // Media item existence check:
    let Ok(relative_path_str) = relative_path_abs(file) else {
        return Ok(false);
    };
    let Ok(media_item_id) = sqlx::query_scalar!(
        "SELECT id FROM media_item WHERE relative_path = $1",
        relative_path_str
    )
    .fetch_optional(executor)
    .await
    else {
        return Ok(false);
    };
    let Some(media_item_id) = media_item_id else {
        return Ok(false);
    };
    // media item exists, check thumbnails existence
    let exist = thumbs_exist(file, &media_item_id)?;
    Ok(exist)
}

/// Verifies if all expected thumbnails for a given media file exist on disk.
/// # Errors
///
/// This function's signature returns a Result, but the current implementation does not produce any errors.
pub fn thumbs_exist(file: &Path, media_item_id: &str) -> color_eyre::Result<bool> {
    let thumb_config = get_thumb_options();
    let is_photo = is_photo_file(file);
    let is_video = is_video_file(file);

    let photo_thumb_ext = &thumb_config.thumbnail_extension;
    let video_thumb_ext = &thumb_config.video_options.extension;
    let mut should_exist: Vec<String> = vec![];

    if is_photo || is_video {
        // Both photo and video should have a thumbnail for each entry in .heights.
        for h in &thumb_config.heights {
            should_exist.push(format!("{h}p.{photo_thumb_ext}"));
        }
    }
    if is_video {
        for p in &thumb_config.video_options.percentages {
            should_exist.push(format!("{p}_percent.{photo_thumb_ext}"));
        }
        for x in &thumb_config.video_options.transcode_outputs {
            let height = x.height;
            should_exist.push(format!("{height}p.{video_thumb_ext}"));
        }
    }

    let thumb_dir = thumbnails_dir().join(media_item_id);
    for thumb_filename in should_exist {
        let thumb_file_path = thumb_dir.join(thumb_filename.clone());
        if !thumb_file_path.exists() {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Logs a warning message with an 'ALERT:' prefix.
#[macro_export]
macro_rules! alert {
    ($($arg:tt)*) => {
        warn!("ALERT: {}", format_args!($($arg)*));
    };
}
