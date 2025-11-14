use crate::api::album::interfaces::CollaboratorSummary;
use crate::database::album::album::AlbumRole;
use crate::database::error::DbError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::postgres::PgQueryResult;
use sqlx::{Executor, FromRow, Postgres};
use utoipa::ToSchema;

pub async fn remove_album_collaborator(
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

pub async fn insert_album_collaborator(
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

pub async fn get_album_collaborator(
    executor: impl Executor<'_, Database = Postgres>,
    collaborator_id: i64,
) -> Result<Option<AlbumCollaborator>, DbError> {
    Ok( sqlx::query_as!(
        AlbumCollaborator,
        r#"SELECT id, album_id, user_id, remote_user_id, role as "role: AlbumRole", added_at FROM album_collaborator WHERE id = $1"#,
        collaborator_id
    )
        .fetch_optional(executor)
        .await?)
}

pub async fn get_album_collaborators(
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

/// Represents a user's role in an album (a collaborator).
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCollaborator {
    pub id: i64,
    pub album_id: String,
    pub user_id: Option<i32>,
    pub remote_user_id: Option<String>,
    pub role: AlbumRole,
    pub added_at: DateTime<Utc>,
}
