use crate::api::album::interfaces::{AlbumMediaItemSummary, AlbumSortField};
use crate::api::timeline::interfaces::SortDirection;
use crate::database::DbError;
use crate::database::album::album::{Album, AlbumRole, AlbumTimelineInfo};
use crate::database::album::album_collaborator::AlbumCollaborator;
use common_types::pb::api::{CollaboratorSummary, TimelineItem};
use sqlx::postgres::PgQueryResult;
use sqlx::{PgExecutor, PgTransaction};

pub struct AlbumStore;

impl AlbumStore {
    //================================================================================
    // Core Album Management
    //================================================================================

    /// Helper to get the absolute owner ID from the album table (Source of Truth).
    pub async fn get_owner_id(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<Option<i32>, DbError> {
        let rec = sqlx::query!("SELECT owner_id FROM album WHERE id = $1", album_id)
            .fetch_optional(executor)
            .await?;

        Ok(rec.map(|r| r.owner_id))
    }

    pub async fn create(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        user_id: i32,
        name: &str,
        description: Option<String>,
        thumbnail_id: Option<String>,
        is_public: bool,
    ) -> Result<Album, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
            INSERT INTO album (id, owner_id, name, description, thumbnail_id, is_public, manual_sort)
            VALUES ($1, $2, $3, $4, $5, $6, false)
            RETURNING
                id,
                owner_id,
                name,
                description,
                thumbnail_id,
                is_public,
                media_count,
                latest_media_item_timestamp,
                earliest_media_item_timestamp,
                updated_at,
                created_at,
                manual_sort
            "#,
            album_id,
            user_id,
            name,
            description,
            thumbnail_id,
            is_public,
        )
            .fetch_one(executor)
            .await?)
    }

    /// Updates the details of a specific album.
    pub async fn update(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        name: Option<String>,
        description: Option<String>,
        thumbnail_id: Option<String>,
        is_public: Option<bool>,
    ) -> Result<Album, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
            UPDATE album
            SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                thumbnail_id = COALESCE($3, thumbnail_id),
                is_public = COALESCE($4, is_public),
                updated_at = now()
            WHERE id = $5
            RETURNING
                id,
                owner_id,
                name,
                description,
                thumbnail_id,
                is_public,
                media_count,
                latest_media_item_timestamp,
                earliest_media_item_timestamp,
                updated_at,
                created_at,
                manual_sort
            "#,
            name,
            description,
            thumbnail_id,
            is_public,
            album_id
        )
        .fetch_one(executor)
        .await?)
    }

    /// Updates the details of a specific album.
    pub async fn remove_description(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<PgQueryResult, DbError> {
        Ok(sqlx::query!(
            r#"
            UPDATE album
            SET description = NULL
            WHERE id = $1
            "#,
            album_id
        )
        .execute(executor)
        .await?)
    }

    /// Retrieves a single album by its ID, including the count of media items.
    pub async fn find_by_id(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<Option<Album>, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
            SELECT
                id,
                owner_id,
                name,
                description,
                thumbnail_id,
                is_public,
                media_count,
                latest_media_item_timestamp,
                earliest_media_item_timestamp,
                updated_at,
                created_at,
                manual_sort
            FROM album
            WHERE id = $1
            "#,
            album_id
        )
        .fetch_optional(executor)
        .await?)
    }

    /// Retrieves a single album by its ID, including the first and last date taken.
    pub async fn find_timeline_info(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<Option<AlbumTimelineInfo>, DbError> {
        Ok(sqlx::query_as!(
            AlbumTimelineInfo,
            r#"
            SELECT
                a.id,
                a.owner_id,
                a.name,
                a.description,
                a.thumbnail_id,
                a.is_public,
                a.created_at,
                MIN(mi.taken_at_local) as "first_date?",
                MAX(mi.taken_at_local) as "last_date?"
            FROM album a
            LEFT JOIN album_media_item ami ON a.id = ami.album_id
            LEFT JOIN media_item mi ON ami.media_item_id = mi.id AND mi.deleted = false
            WHERE a.id = $1
            GROUP BY a.id
            "#,
            album_id
        )
        .fetch_optional(executor)
        .await?)
    }

    /// Retrieves all albums a user is a collaborator on.
    pub async fn list_by_user_id(
        executor: impl PgExecutor<'_>,
        user_id: i32,
    ) -> Result<Vec<Album>, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
            SELECT
                a.id,
                owner_id,
                name,
                description,
                thumbnail_id,
                is_public,
                media_count,
                latest_media_item_timestamp,
                earliest_media_item_timestamp,
                updated_at,
                created_at,
                manual_sort
            FROM album a
            JOIN album_collaborator ac ON a.id = ac.album_id
            WHERE ac.user_id = $1
            ORDER BY a.updated_at DESC
            "#,
            user_id
        )
        .fetch_all(executor)
        .await?)
    }

    /// Retrieves all albums a user is a collaborator on with sorting.
    pub async fn list_with_count_by_user_id(
        executor: impl PgExecutor<'_>,
        user_id: i32,
        sort_field: AlbumSortField,
        sort_dir: SortDirection,
    ) -> Result<Vec<Album>, DbError> {
        Ok(sqlx::query_as!(
        Album,
        r#"
            SELECT
                a.id,
                a.owner_id,
                a.name,
                a.description,
                a.thumbnail_id,
                a.is_public,
                a.manual_sort,
                a.created_at,
                a.updated_at,
                a.media_count,
                a.latest_media_item_timestamp,
                a.earliest_media_item_timestamp
            FROM album a
            LEFT JOIN album_collaborator ac ON a.id = ac.album_id AND ac.user_id = $1
            WHERE a.owner_id = $1 OR ac.user_id = $1
            ORDER BY
                CASE WHEN $2 = 'name' AND $3 = 'ASC' THEN a.name END ASC,
                CASE WHEN $2 = 'name' AND $3 = 'DESC' THEN a.name END DESC,
                CASE WHEN $2 = 'updated_at' AND $3 = 'ASC' THEN a.updated_at END ASC,
                CASE WHEN $2 = 'updated_at' AND $3 = 'DESC' THEN a.updated_at END DESC,
                CASE WHEN $2 = 'latest_photo' AND $3 = 'ASC' THEN a.latest_media_item_timestamp END ASC NULLS LAST,
                CASE WHEN $2 = 'latest_photo' AND $3 = 'DESC' THEN a.latest_media_item_timestamp END DESC NULLS LAST,
                a.id
        "#,
        user_id,
        sort_field.as_str(),
        sort_dir.as_sql()
    )
            .fetch_all(executor)
            .await?)
    }

    //================================================================================
    // Album Media Item Management
    //================================================================================

    /// Adds multiple media items.
    /// Ranks are assigned by taking the current max rank and adding increments
    /// based on the `sort_timestamp` of the new items.
    pub async fn add_media_items(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        media_item_ids: &[String],
        added_by_user_id: i32,
    ) -> Result<PgQueryResult, DbError> {
        Ok(sqlx::query!(
            r#"
            INSERT INTO album_media_item (album_id, media_item_id, rank, added_by_user)
            SELECT
                $1::TEXT,
                items.id,
                COALESCE((SELECT MAX(rank) FROM album_media_item WHERE album_id = $1::TEXT), 0.0)
                    + (ROW_NUMBER() OVER (ORDER BY mi.sort_timestamp ASC) * 1000.0),
                $2
            FROM UNNEST($3::TEXT[]) AS items(id)
            JOIN media_item mi ON mi.id = items.id
            ON CONFLICT (album_id, media_item_id) DO NOTHING
            "#,
            album_id,
            added_by_user_id,
            media_item_ids
        )
        .execute(executor)
        .await?)
    }

    /// Resets the ranks of all items in an album based on their chronological `sort_timestamp`.
    pub async fn reorder_by_date(
        tx: &mut PgTransaction<'_>,
        album_id: &str,
    ) -> Result<(), DbError> {
        // Re-rank everything
        sqlx::query!(
            r#"
            UPDATE album_media_item ami
            SET rank = sub.new_rank
            FROM (
                SELECT
                    ami2.album_id,
                    ami2.media_item_id,
                    ROW_NUMBER() OVER (ORDER BY mi.sort_timestamp ASC, mi.created_at ASC) * 1000.0 as new_rank
                FROM album_media_item ami2
                JOIN media_item mi ON ami2.media_item_id = mi.id
                WHERE ami2.album_id = $1
            ) sub
            WHERE ami.album_id = sub.album_id
              AND ami.media_item_id = sub.media_item_id;
            "#,
            album_id
        )
            .execute(&mut **tx)
            .await?;

        // Reset the manual_sort flag
        sqlx::query!(
            "UPDATE album SET manual_sort = false WHERE id = $1",
            album_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn remove_media_items_by_id(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        media_item_ids: &[String],
    ) -> Result<PgQueryResult, DbError> {
        Ok(sqlx::query!(
            r#"
            DELETE FROM album_media_item
            WHERE album_id = $1 AND media_item_id = ANY($2::TEXT[])
            "#,
            album_id,
            media_item_ids
        )
        .execute(executor)
        .await?)
    }

    /// Retrieves all media items associated with an album, joined with their metadata.
    pub async fn list_media_items(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<Vec<AlbumMediaItemSummary>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mi.id,
                mi.is_video,
                mi.has_thumbnails,
                mi.duration_ms::INT as duration_ms,
                mi.taken_at_local::TEXT as "timestamp!",
                ami.added_at
            FROM album_media_item ami
            JOIN media_item mi ON ami.media_item_id = mi.id
            WHERE ami.album_id = $1
            AND mi.deleted = false
            ORDER BY ami.added_at DESC
            "#,
            album_id
        )
        .fetch_all(executor)
        .await?;

        // Map the flat DB rows to the nested AlbumMediaItemSummary struct
        let summaries = rows
            .into_iter()
            .map(|r| AlbumMediaItemSummary {
                media_item: TimelineItem {
                    id: r.id,
                    is_video: r.is_video,
                    has_thumbnails: r.has_thumbnails,
                    duration_ms: r.duration_ms,
                    timestamp: r.timestamp,
                },
                added_at: r.added_at,
            })
            .collect();

        Ok(summaries)
    }

    /// Checks if a specific media item exists in an album and is not deleted.
    pub async fn has_media_item(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        media_item_id: &str,
    ) -> Result<bool, DbError> {
        let result = sqlx::query!(
            r#"
            SELECT 1 as "one!"
            FROM album_media_item ami
            JOIN media_item mi ON ami.media_item_id = mi.id
            WHERE ami.album_id = $1
              AND ami.media_item_id = $2
              AND mi.deleted = false
            LIMIT 1
            "#,
            album_id,
            media_item_id
        )
        .fetch_optional(executor)
        .await?;

        Ok(result.is_some())
    }

    /// Finds the media item ID located at the middle of the album.
    /// Useful for generating a representative thumbnail.
    pub async fn find_middle_media_item_id(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<Option<String>, DbError> {
        struct Row {
            id: String,
        }

        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT media_item_id as id
            FROM (
                SELECT
                    media_item_id,
                    ROW_NUMBER() OVER (ORDER BY rank ASC) as rn,
                    COUNT(*) OVER () as total_count
                FROM album_media_item
                WHERE album_id = $1
            ) t
            WHERE rn = (total_count / 2) + 1
            LIMIT 1
            "#,
            album_id
        )
        .fetch_optional(executor)
        .await?;

        Ok(result.map(|r| r.id))
    }

    //================================================================================
    // Album Collaborator Management
    //================================================================================

    /// Adds a collaborator to an album or updates their role if they already exist.
    pub async fn upsert_collaborator(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        user_id: i32,
        role: AlbumRole,
    ) -> Result<AlbumCollaborator, DbError> {
        Ok(sqlx::query_as!(
            AlbumCollaborator,
            r#"
            INSERT INTO album_collaborator (album_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (album_id, user_id) DO UPDATE SET role = EXCLUDED.role
            RETURNING id, album_id, user_id, role as "role: AlbumRole", added_at
            "#,
            album_id,
            user_id,
            role as AlbumRole
        )
        .fetch_one(executor)
        .await?)
    }

    /// Removes a collaborator from an album by their collaborator ID.
    pub async fn remove_collaborator_by_id(
        executor: impl PgExecutor<'_>,
        collaborator_id: i64,
    ) -> Result<PgQueryResult, DbError> {
        Ok(sqlx::query!(
            "DELETE FROM album_collaborator WHERE id = $1",
            collaborator_id
        )
        .execute(executor)
        .await?)
    }

    /// Retrieves a collaborator by their ID.
    pub async fn find_collaborator_by_id(
        executor: impl PgExecutor<'_>,
        collaborator_id: i64,
    ) -> Result<Option<AlbumCollaborator>, DbError> {
        Ok(sqlx::query_as!(
            AlbumCollaborator,
            r#"
            SELECT id, album_id, user_id, role as "role: AlbumRole", added_at
            FROM album_collaborator
            WHERE id = $1
            "#,
            collaborator_id
        )
        .fetch_optional(executor)
        .await?)
    }

    /// Retrieves all collaborators for a given album.
    pub async fn list_collaborators(
        executor: impl PgExecutor<'_>,
        album_id: &str,
    ) -> Result<Vec<CollaboratorSummary>, DbError> {
        Ok(sqlx::query_as!(
            CollaboratorSummary,
            r#"
            SELECT
                ac.id,
                ac.user_id,
                u.name,
                ac.role as "role: String"
            FROM album_collaborator ac
            JOIN app_user u ON ac.user_id = u.id
            WHERE ac.album_id = $1
            "#,
            album_id
        )
        .fetch_all(executor)
        .await?)
    }

    /// Gets the role of a user for a specific album.
    pub async fn find_user_role(
        executor: impl PgExecutor<'_>,
        album_id: &str,
        user_id: i32,
    ) -> Result<Option<AlbumRole>, DbError> {
        Ok(sqlx::query_scalar!(
            r#"
            SELECT role as "role: AlbumRole"
            FROM album_collaborator
            WHERE user_id = $1 AND album_id = $2
            "#,
            user_id,
            album_id
        )
        .fetch_optional(executor)
        .await?)
    }
}
