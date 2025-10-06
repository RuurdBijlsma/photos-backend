use photos_core::{
    enqueue_file_ingest, enqueue_file_remove, get_relative_path_str, get_thumbnail_options,
    get_thumbnails_dir,
};
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;
use walkdir::WalkDir;

fn has_allowed_ext(path: &Path, allowed: &HashSet<&str>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| allowed.contains(ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

async fn get_media_files(folder: &Path, allowed_exts: HashSet<&str>) -> color_eyre::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() && has_allowed_ext(entry.path(), &allowed_exts) {
            files.push(entry.into_path());
        }
    }
    Ok::<_, color_eyre::Report>(files)
}

async fn get_thumbnail_folders() -> color_eyre::Result<HashSet<String>> {
    let mut set = HashSet::new();
    let mut entries = fs::read_dir(get_thumbnails_dir()).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir()
            && let Some(name) = entry.file_name().to_str() {
                set.insert(name.to_owned());
            }
    }
    Ok(set)
}

async fn remove_unused_thumbnails(pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    let job_count: i64 = sqlx::query_scalar("SELECT count(id) FROM job_queue")
        .fetch_one(pool)
        .await?;
    if job_count > 0 {
        return Ok(()); // skip if ingest jobs are pending
    }

    let (thumb_ids, db_ids) = tokio::try_join!(get_thumbnail_folders(), async {
        let rows: Vec<String> = sqlx::query_scalar("SELECT id FROM media_item")
            .fetch_all(pool)
            .await?;
        Ok::<HashSet<String>, color_eyre::Report>(rows.into_iter().collect())
    })?;

    let to_delete: Vec<_> = thumb_ids.difference(&db_ids).cloned().collect();
    let base = get_thumbnails_dir();
    for id in to_delete {
        fs::remove_dir_all(base.join(id)).await?;
    }

    Ok(())
}

pub async fn sync_files_to_db(media_dir: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    let cfg = get_thumbnail_options();
    let allowed: HashSet<_> = cfg
        .photo_extensions
        .iter()
        .chain(cfg.video_extensions.iter())
        .map(|s| s.as_str())
        .collect();

    let all_files = get_media_files(media_dir, allowed).await?;
    let fs_paths: HashSet<String> = all_files
        .into_iter()
        .flat_map(|p| get_relative_path_str(&p))
        .collect();

    let db_paths: HashSet<String> = sqlx::query_scalar("SELECT relative_path FROM media_item")
        .fetch_all(pool)
        .await?
        .into_iter()
        .collect();

    let to_ingest: Vec<_> = fs_paths.difference(&db_paths).cloned().collect();
    let to_remove: Vec<_> = db_paths.difference(&fs_paths).cloned().collect();

    for path in to_ingest {
        enqueue_file_ingest(&media_dir.join(&path), pool).await?;
    }
    for path in to_remove {
        enqueue_file_remove(&media_dir.join(&path), pool).await?;
    }

    remove_unused_thumbnails(pool).await?;
    info!("Sync complete");
    Ok(())
}
