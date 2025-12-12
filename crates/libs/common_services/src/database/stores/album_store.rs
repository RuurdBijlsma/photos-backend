use crate::api::album::interfaces::{AlbumMediaItemSummary, AlbumSortField, CollaboratorSummary};
use crate::api::timeline::interfaces::SortDirection;
use crate::database::album::album::{Album, AlbumRole, AlbumWithCount};
use crate::database::album::album_collaborator::AlbumCollaborator;
use crate::database::DbError;
use common_types::pb::api::TimelineItem;
use sqlx::postgres::PgQueryResult;
use sqlx::{Executor, Postgres};

pub struct AlbumStore;

impl AlbumStore {
    //================================================================================
    // Core Album Management
    //================================================================================

    /// Creates a new album and assigns the user as the owner.
    pub async fn create(
        executor: impl Executor<'_, Database = Postgres>,
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
            INSERT INTO album (id, owner_id, name, description, thumbnail_id, is_public)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
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
        executor: impl Executor<'_, Database = Postgres>,
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
            RETURNING *
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

    /// Retrieves a single album by its ID.
    pub async fn find_by_id(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
    ) -> Result<Option<Album>, DbError> {
        Ok(
            sqlx::query_as!(Album, "SELECT * FROM album WHERE id = $1", album_id)
                .fetch_optional(executor)
                .await?,
        )
    }

    /// Retrieves all albums a user is a collaborator on.
    pub async fn list_by_user_id(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: i32,
    ) -> Result<Vec<Album>, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
            SELECT a.*
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

    /// Retrieves all albums a user is a collaborator on.
    pub async fn list_with_count_by_user_id(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: i32,
        sort_field: AlbumSortField,
        sort_dir: SortDirection,
    ) -> Result<Vec<AlbumWithCount>, DbError> {
        // todo: benchmark
        Ok(sqlx::query_as!(
        AlbumWithCount,
        r#"
            SELECT
                a.id,
                a.owner_id,
                a.name,
                a.description,
                a.thumbnail_id,
                a.is_public,
                a.created_at,
                a.updated_at,
                COUNT(mi.id)::INT AS "media_count!"
            FROM album a
            -- Check collaboration status
            LEFT JOIN album_collaborator ac ON a.id = ac.album_id AND ac.user_id = $1
            -- Join for media items
            LEFT JOIN album_media_item ami ON a.id = ami.album_id
            -- Join media items to filter deleted ones and get timestamps
            LEFT JOIN media_item mi ON ami.media_item_id = mi.id AND mi.deleted = false
            WHERE
                a.owner_id = $1      -- User is the owner
                OR
                ac.user_id = $1      -- OR User is a collaborator
            GROUP BY
                a.id,
                a.owner_id,
                a.name,
                a.description,
                a.thumbnail_id,
                a.is_public,
                a.created_at,
                a.updated_at
            ORDER BY
                -- 1. Sort by Name
                CASE WHEN $2 = 'name' AND $3 = 'ASC' THEN a.name END,
                CASE WHEN $2 = 'name' AND $3 = 'DESC' THEN a.name END DESC,

                -- 2. Sort by Updated At
                CASE WHEN $2 = 'updated_at' AND $3 = 'ASC' THEN a.updated_at END,
                CASE WHEN $2 = 'updated_at' AND $3 = 'DESC' THEN a.updated_at END DESC,

                -- 3. Sort by Latest Photo (MAX sort_timestamp)
                -- NULLS LAST ensures empty albums appear at the bottom
                CASE WHEN $2 = 'latest_photo' AND $3 = 'ASC' THEN MAX(mi.sort_timestamp) END NULLS LAST,
                CASE WHEN $2 = 'latest_photo' AND $3 = 'DESC' THEN MAX(mi.sort_timestamp) END DESC NULLS LAST,

                -- Secondary sort to ensure stable pagination/ordering
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

    /// Adds multiple media items to an album.
    /// Ignores duplicates if a media item is already in the album.
    pub async fn add_media_items(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
        media_item_ids: &[String],
        added_by_user_id: i32,
    ) -> Result<PgQueryResult, DbError> {
        Ok(sqlx::query!(
            r#"
            INSERT INTO album_media_item (album_id, media_item_id, added_by_user)
            SELECT $1, item_id, $2
            FROM UNNEST($3::TEXT[]) as item_id
            ON CONFLICT (album_id, media_item_id) DO NOTHING
            "#,
            album_id,
            added_by_user_id,
            media_item_ids
        )
        .execute(executor)
        .await?)
    }

    /// Removes multiple media items from an album by their IDs.
    pub async fn remove_media_items_by_id(
        executor: impl Executor<'_, Database = Postgres>,
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
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
    ) -> Result<Vec<AlbumMediaItemSummary>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mi.id,
                mi.is_video,
                mi.use_panorama_viewer as is_panorama,
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
                    is_panorama: r.is_panorama,
                    duration_ms: r.duration_ms,
                    timestamp: r.timestamp,
                },
                added_at: r.added_at,
            })
            .collect();

        Ok(summaries)
    }

    //================================================================================
    // Album Collaborator Management
    //================================================================================

    /// Adds a collaborator to an album or updates their role if they already exist.
    pub async fn upsert_collaborator(
        executor: impl Executor<'_, Database = Postgres>,
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
        executor: impl Executor<'_, Database = Postgres>,
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
        executor: impl Executor<'_, Database = Postgres>,
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
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
    ) -> Result<Vec<CollaboratorSummary>, DbError> {
        Ok(sqlx::query_as!(
            CollaboratorSummary,
            r#"
            SELECT ac.id, u.name, ac.role as "role: AlbumRole"
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
        executor: impl Executor<'_, Database = Postgres>,
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
