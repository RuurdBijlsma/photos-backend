use app_state::{AppSettings, MakeRelativePath};
use common_services::database::app_user::user_from_relative_path;
use common_services::database::jobs::JobType;
use common_services::job_queue::{enqueue_full_ingest, enqueue_job};
use sqlx::PgPool;
use std::path::Path;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Handles a create event from the watcher.
pub async fn handle_create(
    pool: &PgPool,
    settings: &AppSettings,
    path: &Path,
) -> color_eyre::Result<()> {
    if path.is_file() {
        enqueue_file_job(pool, settings, path, JobType::Ingest).await?;
    } else {
        info!("Directory created: {:?}. Scanning for new files.", path);
        for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                enqueue_file_job(pool, settings, entry.path(), JobType::Ingest).await?;
            }
        }
    }
    Ok(())
}

/// Handles a remove event from the watcher.
pub async fn handle_remove(
    pool: &PgPool,
    settings: &AppSettings,
    path: &Path,
) -> color_eyre::Result<()> {
    // This logic is preserved as per your request to differentiate file from folder deletes.
    if is_path_in_db(pool, settings, path).await? {
        enqueue_file_job(pool, settings, path, JobType::Remove).await?;
    } else {
        info!(
            "Directory removed: {:?}. Removing all media items within.",
            path
        );
        handle_remove_folder(pool, settings, path).await?;
    }
    Ok(())
}

/// A helper function to enqueue a job for a given file path.
async fn enqueue_file_job(
    pool: &PgPool,
    settings: &AppSettings,
    path: &Path,
    job_type: JobType,
) -> color_eyre::Result<()> {
    let relative_path = &path.make_relative(&settings.ingest.media_folder)?;
    let Some(user) = user_from_relative_path(relative_path, pool).await? else {
        warn!(
            "Could not find user for path: {}. Cannot enqueue job.",
            relative_path
        );
        return Ok(());
    };

    match job_type {
        JobType::Ingest => enqueue_full_ingest(pool, settings, relative_path, user.id).await?,
        JobType::Remove => {
            enqueue_job::<()>(pool, settings, JobType::Remove)
                .relative_path(relative_path)
                .user_id(user.id)
                .call()
                .await?;
        }
        _ => warn!("Unsupported job type for watcher: {:?}", job_type),
    }

    Ok(())
}

/// Handles removing a folder by finding all its items in the DB and enqueuing their removal.
async fn handle_remove_folder(
    pool: &PgPool,
    settings: &AppSettings,
    folder: &Path,
) -> color_eyre::Result<()> {
    let relative_dir =folder.make_relative(&settings.ingest.media_folder)?;
    let pattern = format!("{relative_dir}%");

    let relative_paths = sqlx::query_scalar!(
        r"SELECT relative_path FROM media_item WHERE relative_path LIKE $1",
        pattern
    )
    .fetch_all(pool)
    .await?;

    if relative_paths.is_empty() {
        debug!(
            "No media items found in DB for removed directory: {:?}",
            folder
        );
        return Ok(());
    }

    for path in relative_paths {
        let absolute_path = settings.ingest.media_folder.join(path);
        enqueue_file_job(pool, settings, &absolute_path, JobType::Remove).await?;
    }

    Ok(())
}

/// Checks if a given path exists in either the `media_item` or `jobs` table.
async fn is_path_in_db(
    pool: &PgPool,
    settings: &AppSettings,
    path: &Path,
) -> color_eyre::Result<bool> {
    let relative_path =path.make_relative(&settings.ingest.media_folder)?;
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM media_item WHERE relative_path = $1
            UNION ALL
            SELECT 1 FROM jobs WHERE relative_path = $1
        )
        "#,
        relative_path
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    Ok(exists)
}
