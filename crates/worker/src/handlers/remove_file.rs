use common_photos::{Job, media_dir, thumbnails_dir};
use sqlx::{Executor, Postgres};

pub async fn remove_file<'c, E>(executor: E, job: &Job) -> color_eyre::Result<()>
where
    E: Executor<'c, Database = Postgres>,
{
    let file = media_dir().join(&job.relative_path);
    // 1. Delete from main media items table (cascades should handle the rest)
    let deleted_id: String = sqlx::query_scalar!(
        "DELETE FROM media_item WHERE relative_path = $1 RETURNING id",
        &job.relative_path
    )
    .fetch_one(executor)
    .await?;

    // 2. Delete thumbnails from the filesystem
    let thumb_dir = thumbnails_dir();
    let thumb_file_dir = thumb_dir.join(deleted_id);
    if thumb_file_dir.exists() {
        tokio::fs::remove_dir_all(thumb_file_dir).await?;
    }

    if file.exists() {
        tokio::fs::remove_file(file).await?;
    }

    Ok(())
}
