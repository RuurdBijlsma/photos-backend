use crate::queue_logic::Job;
use common_photos::get_thumbnails_dir;
use sqlx::PgTransaction;
use std::path::Path;

pub async fn remove_file(
    job: &Job,
    file: &Path,
    tx: &mut PgTransaction<'_>,
) -> color_eyre::Result<()> {
    // 1. Delete from main media items table (cascades should handle the rest)
    let deleted_id: String = sqlx::query_scalar!(
        "DELETE FROM media_item WHERE relative_path = $1 RETURNING id",
        &job.relative_path
    )
    .fetch_one(&mut **tx)
    .await?;

    // 2. Delete thumbnails from the filesystem
    let thumb_dir = get_thumbnails_dir();
    let thumb_file_dir = thumb_dir.join(deleted_id);
    if thumb_file_dir.exists() {
        tokio::fs::remove_dir_all(thumb_file_dir).await?;
    }

    if file.exists() {
        tokio::fs::remove_file(file).await?;
    }

    Ok(())
}
