use crate::api::people::interfaces::PersonSummary;
use crate::database::DbError;
use common_types::pb::api::SimpleTimelineItem;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

pub struct PersonStore;

impl PersonStore {
    #[instrument(skip(executor))]
    pub async fn list_by_user_id(
        executor: impl PgExecutor<'_>,
        user_id: i32,
    ) -> Result<Vec<PersonSummary>, DbError> {
        let people = sqlx::query_as!(
            PersonSummary,
            r#"
            SELECT 
                p.id, 
                p.name, 
                p.thumbnail_media_item_id,
                COUNT(DISTINCT va.media_item_id)::INT as "photo_count!"
            FROM person p
            LEFT JOIN face f ON f.person_id = p.id
            LEFT JOIN visual_analysis va ON f.visual_analysis_id = va.id
            WHERE p.user_id = $1
            GROUP BY p.id
            ORDER BY "photo_count!" DESC, p.id ASC
            "#,
            user_id
        )
        .fetch_all(executor)
        .await?;

        Ok(people)
    }

    #[instrument(skip(executor))]
    pub async fn find_by_id(
        executor: impl PgExecutor<'_>,
        person_id: i64,
        user_id: i32,
    ) -> Result<Option<PersonSummary>, DbError> {
        let person = sqlx::query_as!(
            PersonSummary,
            r#"
            SELECT 
                p.id, 
                p.name, 
                p.thumbnail_media_item_id,
                COUNT(DISTINCT va.media_item_id)::INT as "photo_count!"
            FROM person p
            LEFT JOIN face f ON f.person_id = p.id
            LEFT JOIN visual_analysis va ON f.visual_analysis_id = va.id
            WHERE p.id = $1 AND p.user_id = $2
            GROUP BY p.id
            "#,
            person_id,
            user_id
        )
        .fetch_optional(executor)
        .await?;

        Ok(person)
    }

    #[instrument(skip(executor))]
    pub async fn update_name(
        executor: impl PgExecutor<'_>,
        person_id: i64,
        user_id: i32,
        name: Option<String>,
    ) -> Result<u64, DbError> {
        let result = sqlx::query!(
            r#"
            UPDATE person 
            SET name = $1, updated_at = now()
            WHERE id = $2 AND user_id = $3
            "#,
            name,
            person_id,
            user_id
        )
        .execute(executor)
        .await?;

        Ok(result.rows_affected())
    }

    #[instrument(skip(pool))]
    pub async fn get_person_media_items(
        pool: &PgPool,
        person_id: i64,
        user_id: i32,
    ) -> Result<Vec<SimpleTimelineItem>, DbError> {
        let items = sqlx::query_as!(
            SimpleTimelineItem,
            r#"
            SELECT 
                id,
                is_video,
                has_thumbnails,
                duration_ms::INT as duration_ms,
                ratio as "ratio!"
            FROM (
                SELECT DISTINCT ON (mi.id)
                    mi.id,
                    mi.is_video,
                    mi.has_thumbnails,
                    mi.duration_ms,
                    (mi.width::real / mi.height::real) as ratio,
                    mi.sort_timestamp
                FROM face f
                JOIN visual_analysis va ON f.visual_analysis_id = va.id
                JOIN media_item mi ON va.media_item_id = mi.id
                WHERE f.person_id = $1 AND mi.user_id = $2 AND mi.deleted = false
                ORDER BY mi.id
            ) sub
            ORDER BY sub.sort_timestamp DESC
            "#,
            person_id,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(items)
    }
}
