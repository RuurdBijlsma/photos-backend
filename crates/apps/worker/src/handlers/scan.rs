use crate::context::WorkerContext;
use crate::handlers::JobResult;
use app_state::{AppSettings, MakeRelativePath};
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use common_services::alert;
use common_services::database::jobs::{Job, JobType};
use common_services::database::user_store::UserStore;
use common_services::job_queue::{bulk_enqueue_full_ingest, enqueue_full_ingest, enqueue_job};
use sqlx::PgPool;
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tracing::warn;
use tracing::{error, info};
use walkdir::WalkDir;

/// Checks if a file path has an extension present in a given set of allowed extensions.
fn has_allowed_ext(path: &Path, allowed: &HashSet<&str>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| allowed.contains(ext.to_lowercase().as_str()))
}

/// Recursively finds all media files in a folder that have an allowed extension.
fn get_media_files(folder: &Path, allowed_exts: &HashSet<&str>) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && has_allowed_ext(entry.path(), allowed_exts) {
            files.push(entry.into_path());
        }
    }
    files
}

/// Synchronizes the filesystem state with the database by enqueuing new files for ingest and old files for removal.
pub async fn sync_user_files_to_db(
    pool: &PgPool,
    settings: &AppSettings,
    user_folder: &Path,
    user_id: i32,
) -> Result<()> {
    let detection = &settings.ingest.file_detection;
    let allowed: HashSet<_> = detection
        .photo_extensions
        .iter()
        .chain(detection.video_extensions.iter())
        .map(String::as_str)
        .collect();

    let all_files = get_media_files(user_folder, &allowed);
    let fs_paths: HashSet<String> = all_files
        .into_iter()
        .flat_map(|p| p.make_relative(&settings.ingest.media_root))
        .collect();

    let db_paths: HashSet<String> = sqlx::query_scalar!(
        "SELECT relative_path FROM media_item WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect();

    let to_ingest: Vec<_> = fs_paths.difference(&db_paths).cloned().collect();
    let to_remove: Vec<_> = db_paths.difference(&fs_paths).cloned().collect();

    bulk_enqueue_full_ingest(pool, &settings.ingest, &to_ingest, user_id).await?;

    // for rel_path in to_ingest {
    //     if let Err(e) = enqueue_full_ingest(pool, settings, &rel_path, user_id).await {
    //         error!("Error enqueueing file ingest: {:?}", e.to_string());
    //     }
    // }
    for rel_path in to_remove {
        if let Err(e) = enqueue_job::<()>(pool, settings, JobType::Remove)
            .relative_path(rel_path)
            .user_id(user_id)
            .call()
            .await
        {
            error!("Error enqueueing file remove: {:?}", e.to_string());
        }
    }
    Ok(())
}

/// Reads the thumbnails directory and returns a set of all subdirectory names (media item IDs).
async fn get_thumbnail_folders(thumbnail_folder: &Path) -> Result<HashSet<String>> {
    let mut set = HashSet::new();
    let mut entries = fs::read_dir(thumbnail_folder).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            set.insert(name.to_owned());
        }
    }
    Ok(set)
}

/// Synchronises thumbnail folders with the database, deleting orphans and re-ingesting items with missing thumbnails.
async fn sync_thumbnails(pool: &PgPool, settings: &AppSettings) -> Result<()> {
    let Some(job_count) = sqlx::query_scalar!(
        "SELECT count(id) FROM jobs WHERE status IN ('running', 'queued') AND job_type in ('ingest_thumbnails', 'remove')"
    )
        .fetch_one(pool)
        .await?
    else {
        return Err(eyre!("Can't get job count"));
    };
    if job_count > 0 {
        return Ok(()); // skip if ingest jobs are pending
    }

    let thumbnails_root = &settings.ingest.thumbnail_root;
    let media_root = &settings.ingest.media_root;

    let (thumb_ids, db_ids) = tokio::try_join!(get_thumbnail_folders(thumbnails_root), async {
        let rows: Vec<String> = sqlx::query_scalar!("SELECT id FROM media_item")
            .fetch_all(pool)
            .await?;
        Ok::<HashSet<String>, color_eyre::Report>(rows.into_iter().collect())
    })?;

    let to_delete: Vec<_> = thumb_ids.difference(&db_ids).cloned().collect();
    for id in to_delete {
        info!("Deleting thumbnail folder {}", id);
        fs::remove_dir_all(thumbnails_root.join(id)).await?;
    }

    let db_items_missing_thumbnails = db_ids.difference(&thumb_ids).cloned().collect::<Vec<_>>();
    for id in db_items_missing_thumbnails {
        let relative_path: String =
            sqlx::query_scalar!("SELECT relative_path FROM media_item WHERE id = $1", id)
                .fetch_one(pool)
                .await?;
        let file = media_root.join(&relative_path);
        if file.exists() {
            info!("Media item has no thumbnail, re-ingesting now. {:?}", file);
            // Re-ingest files with missing thumbnails, as long as the fs file exists.

            if let Some(user) = UserStore::find_user_by_relative_path(pool, &relative_path).await? {
                enqueue_full_ingest(pool, settings, &relative_path, user.id).await?;
            } else {
                alert!("[Sync - Thumbnail scan] Cannot find user from relative path.");
            }
        }
    }

    Ok(())
}

/// Run the indexing scan.
pub async fn run_scan(pool: &PgPool, settings: &AppSettings) -> Result<()> {
    let users = UserStore::list_users_with_media_folders(pool).await?;
    let media_root = &settings.ingest.media_root;
    info!("Scanning \"{}\" ...", &media_root.display());
    for user in users {
        let Some(media_folder) = user.media_folder else {
            continue;
        };
        sync_user_files_to_db(pool, settings, &media_root.join(media_folder), user.id).await?;
    }
    info!("User scan complete");
    sync_thumbnails(pool, settings).await?;
    info!("Thumbnail sync complete.");

    Ok(())
}

/// Triggers a full scan to synchronise the filesystem and database.
pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    run_scan(&context.pool, &context.settings).await?;

    Ok(JobResult::Done)
}
