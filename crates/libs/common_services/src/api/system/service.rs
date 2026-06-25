use crate::api::app_error::AppError;
use crate::api::system::interfaces::{DiskStats, SystemStats};
use crate::api::system::storage_helpers::{
    ARE_SAME_DRIVE, are_on_same_drive, get_single_disk_info,
};
use app_state::{IngestSettings, constants};
use sqlx::PgPool;
use tokio::task;

pub async fn get_system_stats(
    pool: &PgPool,
    settings: &IngestSettings,
    user_id: i32,
) -> Result<SystemStats, AppError> {
    let db_task = sqlx::query!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM person WHERE user_id = $1) AS "has_people!",
            EXISTS(SELECT 1 FROM photo_cluster WHERE user_id = $1) AS "has_photo_clusters!"
        "#,
        user_id
    )
    .fetch_one(pool);

    let thumb_folder = settings.thumbnail_root.clone();
    let media_folder = settings.media_root.clone();

    let fs_task = task::spawn_blocking(move || {
        let are_same_drive =
            *ARE_SAME_DRIVE.get_or_init(|| are_on_same_drive(&thumb_folder, &media_folder));

        let thumbnail_drive = get_single_disk_info(&thumb_folder)?;
        let media_drive = if are_same_drive {
            thumbnail_drive.clone()
        } else {
            get_single_disk_info(&media_folder)?
        };

        Ok::<_, AppError>(DiskStats {
            thumbnail_drive,
            media_drive,
            are_same_drive,
        })
    });

    let (db_res, fs_res) =
        tokio::try_join!(async { db_task.await.map_err(AppError::from) }, async {
            fs_task.await.map_err(AppError::from)?
        })?;

    Ok(SystemStats {
        has_clustered_people: db_res.has_people,
        has_clustered_photos: db_res.has_photo_clusters,
        allow_file_deletion: constants().allow_file_deletion,
        allow_file_modifications: constants().allow_file_modifications,
        disk: fs_res,
    })
}
