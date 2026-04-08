use crate::context::WorkerContext;
use crate::handlers::JobResult;
use app_state::{AppSettings, MakeRelativePath};
use color_eyre::eyre::Result;
use common_services::database::jobs::{Job, JobType};
use common_services::database::user_store::UserStore;
use common_services::job_queue::{bulk_enqueue_full_ingest, enqueue_job};
use sqlx::PgPool;
use std::collections::HashSet;
use std::path::Path;
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

    tokio::try_join!(
        enqueue_job::<()>(pool, settings, JobType::UpdateGlobalCentroid).call(),
        enqueue_job::<()>(pool, settings, JobType::ClusterFaces).call(),
        enqueue_job::<()>(pool, settings, JobType::ClusterPhotos).call()
    )?;

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

    Ok(())
}

/// Triggers a full scan to synchronise the filesystem and database.
pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    run_scan(&context.pool, &context.settings).await?;

    Ok(JobResult::Done)
}
