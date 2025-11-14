use super::interfaces::{AcceptInviteRequest, AlbumDetailsResponse, AlbumShareClaims};
use crate::api::album::error::AlbumError;
use crate::database::album::album::{Album, AlbumRole, AlbumSummary};
use crate::database::album::album_collaborator::AlbumCollaborator;
use crate::database::album_store::AlbumStore;
use crate::database::app_user::get_user_by_email;
use crate::database::jobs::JobType;
use crate::get_settings::settings;
use crate::job_queue::enqueue_job;
use crate::utils::nice_id;
use chrono::{Duration, Utc};
use common_types::ImportAlbumPayload;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use sqlx::{Executor, PgPool, Postgres};
use tracing::instrument;
use url::Url;

/// Checks if a user has a specific role in an album.
#[instrument(skip(executor))]
async fn check_user_role(
    executor: impl Executor<'_, Database = Postgres>,
    user_id: i32,
    album_id: &str,
    required_roles: &[AlbumRole],
) -> Result<bool, AlbumError> {
    let role = AlbumStore::find_user_role(executor, album_id, user_id).await?;

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
) -> Result<bool, AlbumError>
where
    E: Executor<'c, Database = Postgres>,
{
    check_user_role(executor, user_id, album_id, &[AlbumRole::Owner]).await
}

/// Fetches detailed information for a single album, including media items and collaborators.
/// The user must be a collaborator to view the details.
#[instrument(skip(pool))]
pub async fn get_album_details(
    pool: &PgPool,
    album_id: &str,
    user_id: Option<i32>,
) -> Result<AlbumDetailsResponse, AlbumError> {
    let album = AlbumStore::find_by_id(pool, album_id).await?;
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
        AlbumStore::list_media_items(pool, album_id),
        AlbumStore::list_collaborators(pool, album_id),
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

    let album =
        AlbumStore::create(&mut *tx, &album_id, user_id, name, description, is_public).await?;
    AlbumStore::upsert_collaborator(&mut *tx, &album.id, user_id, AlbumRole::Owner).await?;

    tx.commit().await?;

    Ok(album)
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

    AlbumStore::add_media_items(&mut *tx, album_id, media_item_ids, user_id).await?;

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

    let result = AlbumStore::remove_media_items_by_id(pool, album_id, &[media_item_id.to_owned()]).await?;

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
    let user_to_add = get_user_by_email(pool, new_user_email)
        .await?
        .ok_or_else(|| {
            AlbumError::NotFound(format!("User with email {new_user_email} not found."))
        })?;

    // An owner cannot be added or demoted via this function.
    if matches!(role, AlbumRole::Owner) {
        return Err(AlbumError::Internal(color_eyre::eyre::eyre!(
            "Cannot assign the owner role."
        )));
    }

    // Insert the new collaborator, or update their role if they already exist.
    let new_collaborator =
        AlbumStore::upsert_collaborator(pool, album_id, user_to_add.id, role).await?;

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
    let collaborator_to_remove = AlbumStore::find_collaborator_by_id(pool, collaborator_id_to_remove)
        .await?
        .ok_or_else(|| AlbumError::NotFound("Collaborator not found.".to_string()))?;

    // Safety check: The owner cannot be removed.
    if matches!(collaborator_to_remove.role, AlbumRole::Owner) {
        return Err(AlbumError::Internal(color_eyre::eyre::eyre!(
            "The album owner cannot be removed."
        )));
    }

    // Proceed with deletion.
    AlbumStore::remove_collaborator_by_id(pool, collaborator_id_to_remove).await?;

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
        return Ok(AlbumStore::find_by_id(pool, album_id).await?);
    }

    let updated_album = AlbumStore::update(pool, album_id, name, description, is_public).await?;

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
    let expires_at =
        (Utc::now() + Duration::minutes(settings.auth.album_invitation_expiry_minutes)).timestamp();

    let claims = AlbumShareClaims {
        iss: settings.api.public_url.clone(),
        sub: album_id.to_owned(),
        exp: expires_at,
        sharer_username: user_name.to_owned(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings.auth.jwt_secret.as_ref()),
    )?;

    Ok(token)
}

/// Parses an invite token to extract the remote server URL.
fn extract_token_claims(token: &str) -> Result<AlbumShareClaims, AlbumError> {
    decode::<AlbumShareClaims>(
        token,
        &DecodingKey::from_secret(settings().auth.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|p| p.claims)
    .map_err(|_| AlbumError::Unauthorized("Invalid token.".to_string()))
}

/// Contacts the remote server to get a summary of an album invitation.
pub async fn check_invite(token: &str, http_client: &Client) -> Result<AlbumSummary, AlbumError> {
    let claims = extract_token_claims(token)?;
    let mut remote_url: Url = claims.iss.parse()?;
    remote_url.set_path("/s2s/albums/invite-summary");

    let response = http_client
        .get(remote_url.clone())
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AlbumError::RemoteServerError(format!(
            "Remote server {remote_url} returned an error: {error_text}"
        )));
    }

    let summary: AlbumSummary = response.json().await?;
    Ok(summary)
}

/// Accepts an album invitation and enqueues a background job to start the import.
#[instrument(skip(pool))]
pub async fn accept_invite(
    pool: &PgPool,
    user_id: i32,
    payload: &AcceptInviteRequest,
) -> Result<(), AlbumError> {
    let claims = extract_token_claims(&payload.token)?;

    let payload = ImportAlbumPayload {
        album_name: payload.name.clone(),
        album_description: payload.description.clone(),
        token: payload.token.clone(),
        remote_username: claims.sharer_username,
        remote_url: claims.iss.parse()?,
    };

    enqueue_job(pool, JobType::ImportAlbum)
        .payload(&payload)
        .user_id(user_id)
        .call()
        .await?;

    Ok(())
}
