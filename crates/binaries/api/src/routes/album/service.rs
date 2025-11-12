use super::db_model::{Album, AlbumCollaborator, AlbumRole};
use super::interfaces::{AlbumDetailsResponse, AlbumMediaItemSummary, CollaboratorSummary};
use crate::album::error::AlbumError;
use crate::album::interfaces::AcceptInviteRequest;
use crate::routes::UserRole;
use crate::routes::auth::db_model::User;
use chrono::{Duration, Utc};
use common_photos::{InviteSummaryResponse, JobType, enqueue_job, nice_id, settings};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::Client;
use serde_json::json;
use sqlx::{Executor, PgPool, Postgres};
use tracing::instrument;
use url::Url;
// --- Helper Functions for Permission Checks ---

/// Checks if a user has a specific role in an album.
#[instrument(skip(executor))]
async fn check_user_role<'c, E>(
    executor: E,
    user_id: i32,
    album_id: &str,
    required_roles: &[AlbumRole],
) -> Result<bool, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let role: Option<AlbumRole> = sqlx::query_scalar!(
        r#"
        SELECT role as "role: AlbumRole"
        FROM album_collaborator
        WHERE user_id = $1 AND album_id = $2
        "#,
        user_id,
        album_id
    )
    .fetch_optional(executor)
    .await?;

    match role {
        Some(r) if required_roles.contains(&r) => Ok(true),
        _ => Ok(false),
    }
}

/// A more specific check to see if the user is the owner of the album.
#[instrument(skip(executor))]
async fn is_album_owner<'c, E>(
    executor: E,
    user_id: i32,
    album_id: &str,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    check_user_role(executor, user_id, album_id, &[AlbumRole::Owner]).await
}

// --- Public Service Functions ---

/// Creates a new album and assigns the creator as the owner.
/// This is done in a transaction to ensure both inserts succeed or fail together.
#[instrument(skip(pool))]
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
    sqlx::query!(
        r#"
        INSERT INTO album_collaborator (album_id, user_id, role)
        VALUES ($1, $2, $3)
        "#,
        album_id,
        user_id,
        AlbumRole::Owner as AlbumRole,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(album)
}

/// Fetches all albums a user is a member of (as owner, contributor, or viewer).
#[instrument(skip(pool))]
pub async fn get_user_albums(pool: &PgPool, user_id: i32) -> Result<Vec<Album>, AlbumError> {
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
    .fetch_all(pool)
    .await?;

    Ok(albums)
}

/// Fetches detailed information for a single album, including media items and collaborators.
/// The user must be a collaborator to view the details.
#[instrument(skip(pool))]
pub async fn get_album_details(
    pool: &PgPool,
    album_id: &str,
    user_id: Option<i32>,
) -> Result<AlbumDetailsResponse, AlbumError> {
    let album = sqlx::query_as!(Album, "SELECT * FROM album WHERE id = $1", album_id)
        .fetch_one(pool)
        .await?;
    if !album.is_public {
        let Some(user_id) = user_id else {
            return Err(AlbumError::NotFound(album_id.to_string()));
        };
        // Permission Check: User must be part of the album to view it.
        let is_collaborator = check_user_role(
            pool,
            user_id,
            album_id,
            &[AlbumRole::Owner, AlbumRole::Contributor, AlbumRole::Viewer],
        )
        .await?;
        if !is_collaborator {
            return Err(AlbumError::NotFound(album_id.to_string()));
        }
    }

    // Fetch album details, media items, and collaborators in parallel
    let (media_items_res, collaborators_res) = tokio::join!(
        sqlx::query_as!(
            AlbumMediaItemSummary,
            r#"
            SELECT media_item_id as id, added_at
            FROM album_media_item
            WHERE album_id = $1
            ORDER BY added_at DESC
            "#,
            album_id
        )
        .fetch_all(pool),
        sqlx::query_as!(
            CollaboratorSummary,
            r#"
            SELECT ac.id, u.name, ac.role as "role: AlbumRole"
            FROM album_collaborator ac
            JOIN app_user u ON ac.user_id = u.id
            WHERE ac.album_id = $1
            "#,
            album_id
        )
        .fetch_all(pool)
    );

    let media_items = media_items_res?;
    let collaborators = collaborators_res?;

    Ok(AlbumDetailsResponse {
        id: album.id,
        name: album.name,
        description: album.description,
        is_public: album.is_public,
        owner_id: album.owner_id,
        created_at: album.created_at,
        media_items,
        collaborators,
    })
}

/// Adds one or more media items to an album.
/// The user must be an owner or contributor.
#[instrument(skip(pool))]
pub async fn add_media_to_album(
    pool: &PgPool,
    album_id: &str,
    media_item_ids: &[String],
    user_id: i32,
) -> Result<(), AlbumError> {
    // Permission Check
    let has_permission = check_user_role(
        pool,
        user_id,
        album_id,
        &[AlbumRole::Owner, AlbumRole::Contributor],
    )
    .await?;
    if !has_permission {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    for media_item_id in media_item_ids {
        sqlx::query!(
            r#"
            INSERT INTO album_media_item (album_id, media_item_id, added_by_user)
            VALUES ($1, $2, $3)
            ON CONFLICT (album_id, media_item_id) DO NOTHING
            "#,
            album_id,
            media_item_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Removes a media item from an album.
/// The user must be an owner or contributor.
#[instrument(skip(pool))]
pub async fn remove_media_from_album(
    pool: &PgPool,
    album_id: &str,
    media_item_id: &str,
    user_id: i32,
) -> Result<(), AlbumError> {
    let has_permission = check_user_role(
        pool,
        user_id,
        album_id,
        &[AlbumRole::Owner, AlbumRole::Contributor],
    )
    .await?;
    if !has_permission {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    let result = sqlx::query!(
        "DELETE FROM album_media_item WHERE album_id = $1 AND media_item_id = $2",
        album_id,
        media_item_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AlbumError::NotFound(format!(
            "Media item {media_item_id} not found in album {album_id}"
        )));
    }

    Ok(())
}

/// Adds a new user as a collaborator to an album.
/// The inviting user must be the album owner.
#[instrument(skip(pool))]
pub async fn add_collaborator(
    pool: &PgPool,
    album_id: &str,
    new_user_email: &str,
    role: AlbumRole,
    inviting_user_id: i32,
) -> Result<AlbumCollaborator, AlbumError> {
    // The owner is the only one who can add collaborators.
    if !is_album_owner(pool, inviting_user_id, album_id).await? {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    // Find the user to add by their email.
    let user_to_add = sqlx::query_as!(
        User,
        r#"SELECT 
            id, email, name, media_folder, 
            created_at, updated_at,
            role as "role: UserRole"
        FROM app_user 
        WHERE email = $1"#,
        new_user_email
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AlbumError::NotFound(format!("User with email {new_user_email} not found.")))?;

    // An owner cannot be added or demoted via this function.
    if matches!(role, AlbumRole::Owner) {
        return Err(AlbumError::Internal(color_eyre::eyre::eyre!(
            "Cannot assign the owner role."
        )));
    }

    // Insert the new collaborator, or update their role if they already exist.
    let new_collaborator = sqlx::query_as!(
        AlbumCollaborator,
        r#"
        INSERT INTO album_collaborator (album_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (album_id, user_id) DO UPDATE SET role = EXCLUDED.role
        RETURNING id, album_id, user_id, remote_user_id, role as "role: AlbumRole", added_at
        "#,
        album_id,
        user_to_add.id,
        role as AlbumRole
    )
    .fetch_one(pool)
    .await?;

    Ok(new_collaborator)
}

/// Removes a collaborator from an album.
/// The user performing the action must be the album owner.
#[instrument(skip(pool))]
pub async fn remove_collaborator(
    pool: &PgPool,
    album_id: &str,
    collaborator_id_to_remove: i64,
    requesting_user_id: i32,
) -> Result<(), AlbumError> {
    // Only the album owner can remove collaborators.
    if !is_album_owner(pool, requesting_user_id, album_id).await? {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    // Get the collaborator record to check if we're trying to remove the owner.
    let collaborator_to_remove = sqlx::query_as!(
        AlbumCollaborator,
        r#"SELECT id, album_id, user_id, remote_user_id, role as "role: AlbumRole", added_at FROM album_collaborator WHERE id = $1"#,
        collaborator_id_to_remove
    )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AlbumError::NotFound("Collaborator not found.".to_string()))?;

    // Safety check: The owner cannot be removed.
    if matches!(collaborator_to_remove.role, AlbumRole::Owner) {
        return Err(AlbumError::Internal(color_eyre::eyre::eyre!(
            "The album owner cannot be removed."
        )));
    }

    // Proceed with deletion.
    sqlx::query!(
        "DELETE FROM album_collaborator WHERE id = $1",
        collaborator_id_to_remove
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Updates an album's name and/or description.
/// The user must be the album owner.
#[instrument(skip(pool))]
pub async fn update_album(
    pool: &PgPool,
    album_id: &str,
    user_id: i32,
    name: Option<String>,
    description: Option<String>,
    is_public: Option<bool>,
) -> Result<Album, AlbumError> {
    // Permission Check: Only the owner can update album details.
    if !is_album_owner(pool, user_id, album_id).await? {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    // At least one field must be provided for the update.
    if name.is_none() && description.is_none() && is_public.is_none() {
        // If no changes are requested, just return the current album data.
        return sqlx::query_as!(Album, "SELECT * FROM album WHERE id = $1", album_id)
            .fetch_one(pool)
            .await
            .map_err(Into::into);
    }

    let updated_album = sqlx::query_as!(
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
    .fetch_one(pool)
    .await?;

    Ok(updated_album)
}

#[instrument(skip(pool))]
pub async fn generate_invite(
    pool: &PgPool,
    album_id: &str,
    user_id: i32,
    user_name: &str,
) -> Result<String, AlbumError> {
    // Permission Check: Only the owner can generate an invite.
    if !is_album_owner(pool, user_id, album_id).await? {
        return Err(AlbumError::Unauthorized(
            "Only the album owner can generate an invitation.".to_string(),
        ));
    }

    let settings = settings();
    let secure_token = Alphanumeric.sample_string(&mut rand::rng(), 32);
    let expires_at = Utc::now() + Duration::minutes(settings.api.album_invitation_expiry_minutes);

    sqlx::query!(
        r#"
        INSERT INTO album_invites (album_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        album_id,
        secure_token,
        expires_at
    )
    .execute(pool)
    .await?;

    // Format: inv-{random_string}-{user.name}@{settings.public_url}
    let full_token = format!(
        "inv-{}-{}-{}",
        secure_token,
        user_name.replace(' ', "_"), // Sanitize username
        settings.api.public_url
    );

    Ok(full_token)
}

/// Parses an invite token to extract the remote server URL and the full token string.
fn parse_invite_token(token: &str) -> Result<(Url, &str), AlbumError> {
    let parts: Vec<&str> = token.split('-').collect();
    if parts.len() < 3 || parts[0] != "inv" {
        return Err(AlbumError::InvalidInviteToken(
            "Token format is incorrect.".to_string(),
        ));
    }

    // The host is the last part of the token, e.g., 'alice@photos.alice.com' or 'photos.alice.com'
    // We just need the host part.
    let host_part = parts.last().unwrap();
    let host = host_part.split('@').next_back().unwrap();

    let remote_url = Url::parse(&format!("http://{host}"))
        .or_else(|_| Url::parse(&format!("https://{host}")))
        .map_err(|_| AlbumError::InvalidInviteToken(format!("Invalid host: {host}")))?;

    Ok((remote_url, token))
}

/// Contacts the remote server to get a summary of an album invitation.
pub async fn check_invite(
    token: &str,
    http_client: &Client,
) -> Result<InviteSummaryResponse, AlbumError> {
    let (mut remote_url, full_token) = parse_invite_token(token)?;
    remote_url.set_path("/s2s/albums/invite-summary");

    let response = http_client
        .get(remote_url.clone())
        .bearer_auth(full_token)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AlbumError::RemoteServerError(format!(
            "Remote server {remote_url} returned an error: {error_text}"
        )));
    }

    let summary: InviteSummaryResponse = response.json().await?;
    Ok(summary)
}

/// Accepts an album invitation and enqueues a background job to start the import.
#[instrument(skip(pool))]
pub async fn accept_invite(
    pool: &PgPool,
    user_id: i32,
    payload: &AcceptInviteRequest,
) -> Result<(), AlbumError> {
    // We only need to parse the token to get the remote owner's identity.
    let parts: Vec<&str> = payload.token.split('-').collect();
    if parts.len() < 3 || parts[0] != "inv" {
        return Err(AlbumError::InvalidInviteToken(
            "Token format is incorrect.".to_string(),
        ));
    }
    // The remote identity is the last two parts, e.g., 'john_doe-photos.example.com'
    let remote_owner_identity = format!("{}@{}", parts[parts.len() - 2], parts[parts.len() - 1]);

    let job_payload = json!({
        "token": payload.token,
        "album_name": payload.name,
        "album_description": payload.description,
        "remote_owner_identity": remote_owner_identity,
    });

    enqueue_job(
        pool,
        JobType::ImportAlbum,
        None,
        Some(user_id),
        Some(job_payload),
    )
    .await?;

    Ok(())
}
