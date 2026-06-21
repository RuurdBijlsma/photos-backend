use crate::api::system::interfaces::{DiskInfo, DiskStats, SystemStats};
use app_state::IngestSettings;
use fs2::statvfs;
use sqlx::PgPool;
use std::path::Path;
use std::sync::OnceLock;
use tokio::task;
use crate::api::app_error::AppError;

/// Identifies if the media folder and the thumbnail folder reside on the same drive.
#[must_use]
fn are_on_same_drive(p1: &Path, p2: &Path) -> bool {
    let p1_canon = std::fs::canonicalize(p1).unwrap_or_else(|_| p1.to_path_buf());
    let p2_canon = std::fs::canonicalize(p2).unwrap_or_else(|_| p2.to_path_buf());

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(m1), Ok(m2)) = (std::fs::metadata(&p1_canon), std::fs::metadata(&p2_canon)) {
            return m1.dev() == m2.dev();
        }
    }

    p1_canon.components().next() == p2_canon.components().next()
}

pub fn get_single_disk_info(folder: &Path) -> Result<DiskInfo, AppError> {
    let fs_stats = statvfs(folder)?;
    let available = fs_stats.available_space();
    let total = fs_stats.total_space();

    Ok(DiskInfo {
        disk_available: available,
        disk_total: total,
        disk_used: total.saturating_sub(available),
    })
}

static ARE_SAME_DRIVE: OnceLock<bool> = OnceLock::new();

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
        let are_same_drive = *ARE_SAME_DRIVE.get_or_init(|| {
            are_on_same_drive(&thumb_folder, &media_folder)
        });

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

    let (db_res, fs_res) = tokio::try_join!(
        async { db_task.await.map_err(AppError::from) },
        async { fs_task.await.map_err(AppError::from)? }
    )?;

    Ok(SystemStats {
        has_clustered_people: db_res.has_people,
        has_clustered_photos: db_res.has_photo_clusters,
        disk: fs_res,
    })
}
