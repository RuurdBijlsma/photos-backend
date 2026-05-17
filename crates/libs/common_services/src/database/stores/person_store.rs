use crate::api::people::interfaces::{PersonSummary, UpdatePersonRequest};
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
                p.face_thumb_id,
                COUNT(DISTINCT va.media_item_id)::INT as "photo_count!",
                COALESCE(array_agg(DISTINCT fc.id) FILTER (WHERE fc.id IS NOT NULL), '{}') as "face_cluster_ids!"
            FROM person p
            LEFT JOIN face_cluster fc ON fc.person_id = p.id
            LEFT JOIN face f ON f.face_cluster_id = fc.id
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
        person_id: &str,
        user_id: i32,
    ) -> Result<Option<PersonSummary>, DbError> {
        let person = sqlx::query_as!(
            PersonSummary,
            r#"
            SELECT
                p.id,
                p.name,
                p.face_thumb_id,
                COUNT(DISTINCT va.media_item_id)::INT as "photo_count!",
                COALESCE(array_agg(DISTINCT fc.id) FILTER (WHERE fc.id IS NOT NULL), '{}') as "face_cluster_ids!"
            FROM person p
            LEFT JOIN face_cluster fc ON fc.person_id = p.id
            LEFT JOIN face f ON f.face_cluster_id = fc.id
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
    pub async fn update(
        executor: impl PgExecutor<'_>,
        person_id: &str,
        user_id: i32,
        payload: &UpdatePersonRequest,
    ) -> Result<u64, DbError> {
        let result = sqlx::query!(
            r#"
            UPDATE person
            SET
                name = CASE WHEN $3::boolean THEN name ELSE $4 END,
                face_thumb_id = CASE WHEN $5::boolean THEN face_thumb_id ELSE $6 END,
                updated_at = now()
            WHERE id = $1 AND user_id = $2
            "#,
            person_id,
            user_id,
            payload.name.is_ignore(),
            payload.name.clone().value(),
            payload.face_thumb_id.is_ignore(),
            payload.face_thumb_id.clone().value(),
        )
        .execute(executor)
        .await?;

        Ok(result.rows_affected())
    }

    #[instrument(skip(pool))]
    pub async fn get_person_media_items(
        pool: &PgPool,
        person_id: &str, // Changed from i64
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
                FROM person p
                JOIN face_cluster fc ON fc.person_id = p.id
                JOIN face f ON f.face_cluster_id = fc.id
                JOIN visual_analysis va ON f.visual_analysis_id = va.id
                JOIN media_item mi ON va.media_item_id = mi.id
                WHERE p.id = $1 AND mi.user_id = $2 AND mi.deleted = false
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
