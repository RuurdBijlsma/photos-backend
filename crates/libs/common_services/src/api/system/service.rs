use crate::api::system::interfaces::SystemStats;
use crate::api::user::error::UserError;
use sqlx::PgPool;

pub async fn get_system_stats(pool: &PgPool, user_id: i32) -> Result<SystemStats, UserError> {
    let stats = sqlx::query!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM person WHERE user_id = $1) AS "has_people!",
            EXISTS(SELECT 1 FROM photo_cluster WHERE user_id = $1) AS "has_photo_clusters!"
        "#,
        user_id
    )
        .fetch_one(pool)
        .await?;

    Ok(SystemStats {
        has_clustered_people: stats.has_people,
        has_clustered_photos: stats.has_photo_clusters,
    })
}