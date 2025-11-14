use crate::api::album::error::AlbumError;
use crate::database::album::album_collaborator::insert_album_collaborator;
use crate::database::error::DbError;
use crate::get_settings::settings;
use crate::utils::nice_id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, PgPool, Postgres};
use utoipa::ToSchema;

pub async fn get_album(
    executor: impl Executor<'_, Database = Postgres>,
    album_id: &str,
) -> Result<Album, AlbumError> {
    let album = sqlx::query_as!(Album, "SELECT * FROM album WHERE id = $1", album_id)
        .fetch_one(executor)
        .await?;

    Ok(album)
}

pub async fn get_user_albums(
    executor: impl Executor<'_, Database = Postgres>,
    user_id: i32,
) -> Result<Vec<Album>, AlbumError> {
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

pub async fn create_album(
    pool: &PgPool,
    user_id: i32,
    name: &str,
    description: Option<&str>,
    is_public: bool,
) -> Result<Album, AlbumError> {
    let mut tx = pool.begin().await?;
    let album_id = nice_id(settings().database.media_item_id_length);

    // Step 1: Create the album record
    let album = sqlx::query_as!(
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
    .fetch_one(&mut *tx)
    .await?;

    // Step 2: Add the creator as the 'owner' in the collaborators table
    insert_album_collaborator(&mut *tx, &album.id, user_id, AlbumRole::Owner).await?;

    tx.commit().await?;

    Ok(album)
}

pub async fn get_user_album_role(
    executor: impl Executor<'_, Database = Postgres>,
    user_id: i32,
    album_id: &str,
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

// Custom types to match the ENUMs in your SQL schema.
// It's good practice to define these in a shared library if they are used elsewhere,
// but defining them here is fine for this module.
#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "album_role", rename_all = "lowercase")]
#[derive(PartialEq, Eq)]
pub enum AlbumRole {
    Owner,
    Contributor,
    Viewer,
}

/// Represents a single album in the database.
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub owner_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AlbumSummary {
    pub name: String,
    pub description: Option<String>,
    pub media_item_ids: Vec<String>,
}
