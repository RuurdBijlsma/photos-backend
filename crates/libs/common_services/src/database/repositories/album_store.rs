use crate::api::album::interfaces::{AlbumMediaItemSummary, CollaboratorSummary};
use crate::database::album::album::{Album, AlbumRole};
use crate::database::album::album_collaborator::AlbumCollaborator;
use crate::database::DbError;
use sqlx::postgres::PgQueryResult;
use sqlx::{Executor, Postgres};

pub struct AlbumStore;

impl AlbumStore {
    pub async fn create(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
        user_id: i32,
        name: &str,
        description: Option<&str>,
        is_public: bool,
    ) -> Result<Album, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
        INSERT INTO album (id, owner_id, name, description, is_public)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
            album_id,
            user_id,
            name,
            description,
            is_public,
        )
        .fetch_one(executor)
        .await?)
    }

    pub async fn get(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
    ) -> Result<Album, DbError> {
        let album = sqlx::query_as!(Album, "SELECT * FROM album WHERE id = $1", album_id)
            .fetch_one(executor)
            .await?;

        Ok(album)
    }

    pub async fn update(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
        name: Option<String>,
        description: Option<String>,
        is_public: Option<bool>,
    ) -> Result<Album, DbError> {
        Ok(sqlx::query_as!(
            Album,
            r#"
        UPDATE album
        SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            is_public = COALESCE($3, is_public),
            updated_at = now()
        WHERE id = $4
        RETURNING *
        "#,
            name,
            description,
            is_public,
            album_id
        )
        .fetch_one(executor)
        .await?)
    }

    pub async fn get_user_albums(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: i32,
    ) -> Result<Vec<Album>, DbError> {
        let albums = sqlx::query_as!(
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
        .await?;

        Ok(albums)
    }

    pub async fn get_user_role(
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

    pub async fn get_media_items(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
    ) -> Result<Vec<AlbumMediaItemSummary>, DbError> {
        let items = sqlx::query_as!(
            AlbumMediaItemSummary,
            r#"
            SELECT media_item_id as id, added_at
            FROM album_media_item
            WHERE album_id = $1
            ORDER BY added_at DESC
            "#,
            album_id
        )
        .fetch_all(executor)
        .await?;

        Ok(items)
    }

    pub async fn insert_media_items(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
        media_item_id: &str,
        user_id: i32,
    ) -> Result<PgQueryResult, DbError> {
        // todo: make this accept multiple media item ids
        Ok(sqlx::query!(
            r#"
            INSERT INTO album_media_item (album_id, media_item_id, added_by_user)
            VALUES ($1, $2, $3)
            ON CONFLICT (album_id, media_item_id) DO NOTHING
            "#,
            album_id,
            media_item_id,
            user_id
        )
        .execute(executor)
        .await?)
    }

    pub async fn remove_media_items(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
        media_item_id: &str,
    ) -> Result<PgQueryResult, DbError> {
        Ok(sqlx::query!(
            "DELETE FROM album_media_item WHERE album_id = $1 AND media_item_id = $2",
            album_id,
            media_item_id
        )
        .execute(executor)
        .await?)
    }

    pub async fn remove_collaborator(
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

    pub async fn insert_collaborator(
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
        RETURNING id, album_id, user_id, remote_user_id, role as "role: AlbumRole", added_at
        "#,
            album_id,
            user_id,
            role as AlbumRole
        )
        .fetch_one(executor)
        .await?)
    }

    pub async fn get_collaborator(
        executor: impl Executor<'_, Database = Postgres>,
        collaborator_id: i64,
    ) -> Result<Option<AlbumCollaborator>, DbError> {
        Ok(sqlx::query_as!(
        AlbumCollaborator,
        r#"SELECT id, album_id, user_id, remote_user_id, role as "role: AlbumRole", added_at FROM album_collaborator WHERE id = $1"#,
        collaborator_id
    )
            .fetch_optional(executor)
            .await?)
    }

    pub async fn get_collaborators(
        executor: impl Executor<'_, Database = Postgres>,
        album_id: &str,
    ) -> Result<Vec<CollaboratorSummary>, DbError> {
        let collaborators = sqlx::query_as!(
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
        .await?;

        Ok(collaborators)
    }
}
