use color_eyre::eyre::eyre;
use common_photos::{
    enqueue_full_ingest, enqueue_remove_job, media_dir, relative_path_abs, settings, thumbnails_dir,
};
use sqlx::{PgPool, Pool, Postgres};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tracing::{error, info};
use walkdir::WalkDir;

/// Checks if a file path has an extension present in a given set of allowed extensions.
fn has_allowed_ext(path: &Path, allowed: &HashSet<&str>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| allowed.contains(ext.to_lowercase().as_str()))
}

/// Recursively finds all media files in a folder that have an allowed extension.
///
/// # Errors
///
/// * Can return an I/O error if the specified folder cannot be read.
fn get_media_files(folder: &Path, allowed_exts: &HashSet<&str>) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && has_allowed_ext(entry.path(), allowed_exts) {
            files.push(entry.into_path());
        }
    }
    files
}

/// Reads the thumbnails directory and returns a set of all subdirectory names (media item IDs).
///
/// # Errors
///
/// * Returns an I/O error if the thumbnails directory or its entries cannot be read.
async fn get_thumbnail_folders() -> color_eyre::Result<HashSet<String>> {
    let mut set = HashSet::new();
    let mut entries = fs::read_dir(thumbnails_dir()).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            set.insert(name.to_owned());
        }
    }
    Ok(set)
}

/// Synchronizes thumbnail folders with the database, deleting orphans and re-ingesting items with missing thumbnails.
///
/// # Errors
///
/// * Returns an error for database query failures or file system I/O errors during deletion.
async fn sync_thumbnails(pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    let Some(job_count) =
        sqlx::query_scalar!("SELECT count(id) FROM jobs WHERE status IN ('running', 'queued')")
            .fetch_one(pool)
            .await?
    else {
        return Err(eyre!("Can't get job count"));
    };
    if job_count > 0 {
        return Ok(()); // skip if ingest jobs are pending
    }

    let (thumb_ids, db_ids) = tokio::try_join!(get_thumbnail_folders(), async {
        let rows: Vec<String> = sqlx::query_scalar!("SELECT id FROM media_item")
            .fetch_all(pool)
            .await?;
        Ok::<HashSet<String>, color_eyre::Report>(rows.into_iter().collect())
    })?;

    let to_delete: Vec<_> = thumb_ids.difference(&db_ids).cloned().collect();
    let base = thumbnails_dir();
    for id in to_delete {
        info!("Deleting thumbnail folder {}", id);
        fs::remove_dir_all(base.join(id)).await?;
    }

    let db_items_missing_thumbnails = db_ids.difference(&thumb_ids).cloned().collect::<Vec<_>>();
    let media_dir = media_dir();
    for id in db_items_missing_thumbnails {
        let relative_path: String =
            sqlx::query_scalar!("SELECT relative_path FROM media_item WHERE id = $1", id)
                .fetch_one(pool)
                .await?;
        let file = media_dir.join(&relative_path);
        if file.exists() {
            info!("Media item has no thumbnail, re-ingesting now. {:?}", file);
            // Re-ingest files with missing thumbnails, as long as the fs file exists.
            enqueue_full_ingest(pool, &relative_path).await?;
        }
    }

    Ok(())
}

/// Synchronizes the filesystem state with the database by enqueuing new files for ingest and old files for removal.
///
/// # Errors
///
/// * Returns an error if file system scanning, database queries, or job enqueuing fails.
pub async fn sync_files_to_db(media_dir: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    let thumb_options = &settings().thumbnail_generation;
    let allowed: HashSet<_> = thumb_options
        .photo_extensions
        .iter()
        .chain(thumb_options.video_extensions.iter())
        .map(String::as_str)
        .collect();

    let all_files = get_media_files(media_dir, &allowed);
    let fs_paths: HashSet<String> = all_files
        .into_iter()
        .flat_map(|p| relative_path_abs(&p))
        .collect();

    let db_paths: HashSet<String> = sqlx::query_scalar!("SELECT relative_path FROM media_item")
        .fetch_all(pool)
        .await?
        .into_iter()
        .collect();

    let to_ingest: Vec<_> = fs_paths.difference(&db_paths).cloned().collect();
    let to_remove: Vec<_> = db_paths.difference(&fs_paths).cloned().collect();

    for rel_path in to_ingest {
        if let Err(e) = enqueue_full_ingest(pool, &rel_path).await {
            error!("Error enqueueing file ingest: {:?}", e.to_string());
        }
    }
    for rel_path in to_remove {
        if let Err(e) = enqueue_remove_job(pool, &rel_path).await {
            error!("Error enqueueing file remove: {:?}", e.to_string());
        }
    }

    sync_thumbnails(pool).await?;
    Ok(())
}

/// Run the indexing scan.
///
/// # Errors
///
/// Error if creating thumbnails dir doesn't work out
pub async fn run_scan(pool: &PgPool) -> color_eyre::Result<()> {
    let media_dir = media_dir();
    info!("Scanning \"{}\" ...", &media_dir.display());
    sync_files_to_db(media_dir, pool).await?;
    info!("Scan complete");

    Ok(())
}
