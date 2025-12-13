use super::interfaces::{AcceptInviteRequest, AlbumShareClaims};
use crate::api::album::error::AlbumError;
use crate::api::timeline::interfaces::SortDirection;
use crate::database::album::album::{Album, AlbumRole, AlbumSummary};
use crate::database::album::album_collaborator::AlbumCollaborator;
use crate::database::album_store::AlbumStore;
use crate::database::jobs::JobType;
use crate::database::user_store::UserStore;
use crate::job_queue::enqueue_job;
use crate::s2s_client::{S2SClient, extract_token_claims};
use crate::utils::nice_id;
use app_state::{AppSettings, constants};
use chrono::{Duration, NaiveDate, Utc};
use color_eyre::eyre::Context;
use common_types::ImportAlbumItemPayload;
use common_types::pb::api::{
    AlbumInfo, AlbumRatiosResponse, TimelineItem, TimelineItemsResponse, TimelineMonthItems,
    TimelineMonthRatios,
};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::{Executor, PgPool, Postgres};
use std::collections::HashMap;
use tracing::instrument;

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

async fn can_edit_album(
    executor: impl Executor<'_, Database = Postgres>,
    user_id: i32,
    album_id: &str,
) -> Result<bool, AlbumError> {
    check_user_role(
        executor,
        user_id,
        album_id,
        &[AlbumRole::Owner, AlbumRole::Contributor],
    )
    .await
}

async fn can_view_album(
    executor: impl Executor<'_, Database = Postgres>,
    user_id: i32,
    album_id: &str,
) -> Result<bool, AlbumError> {
    check_user_role(
        executor,
        user_id,
        album_id,
        &[AlbumRole::Owner, AlbumRole::Contributor, AlbumRole::Viewer],
    )
    .await
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

#[instrument(skip(pool))]
pub async fn create_album(
    pool: &PgPool,
    user_id: i32,
    name: &str,
    description: Option<String>,
    is_public: bool,
    media_item_ids: &[String],
) -> Result<Album, AlbumError> {
    let mut tx = pool.begin().await?;
    let album_id = nice_id(constants().database.media_item_id_length);

    let album = AlbumStore::create(
        &mut *tx,
        &album_id,
        user_id,
        name,
        description,
        None,
        is_public,
    )
    .await?;
    AlbumStore::upsert_collaborator(&mut *tx, &album.id, user_id, AlbumRole::Owner).await?;
    if !media_item_ids.is_empty() {
        AlbumStore::add_media_items(&mut *tx, &album_id, media_item_ids, user_id).await?;
        if let Some(thumb) = AlbumStore::find_middle_media_item_id(&mut *tx, &album_id).await? {
            AlbumStore::update(&mut *tx, &album_id, None, None, Some(thumb), None).await?;
        }
    }

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
    // Permission Check: Owner OR Contributor
    if !can_edit_album(pool, user_id, album_id).await? {
        return Err(AlbumError::NotFound("Album not found.".to_string()));
    }

    let mut tx = pool.begin().await?;

    let Some(album_before) = AlbumStore::find_by_id(&mut *tx, album_id).await? else {
        return Err(AlbumError::NotFound("Album not found.".to_string()));
    };
    AlbumStore::add_media_items(&mut *tx, album_id, media_item_ids, user_id).await?;
    if album_before.thumbnail_id.is_none() {
        let thumbnail_id = AlbumStore::find_middle_media_item_id(&mut *tx, album_id).await?;
        if let Some(thumbnail_id) = thumbnail_id {
            AlbumStore::update(&mut *tx, album_id, None, None, Some(thumbnail_id), None).await?;
        }
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
    // Permission Check: Owner OR Contributor
    if !can_edit_album(pool, user_id, album_id).await? {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;
    let result =
        AlbumStore::remove_media_items_by_id(&mut *tx, album_id, &[media_item_id.to_owned()])
            .await?;

    if result.rows_affected() == 0 {
        return Err(AlbumError::NotFound(format!(
            "Media item {media_item_id} not found in album {album_id}"
        )));
    }

    // Fix thumbnail id if it was removed
    if let Some(album) = AlbumStore::find_by_id(&mut *tx, album_id).await? {
        // Check if removed item was the thumbnail
        if Some(media_item_id.to_owned()) == album.thumbnail_id && album.media_count > 0 {
            let thumbnail_id = AlbumStore::find_middle_media_item_id(&mut *tx, album_id).await?;
            if let Some(thumbnail_id) = thumbnail_id {
                AlbumStore::update(&mut *tx, album_id, None, None, Some(thumbnail_id), None)
                    .await?;
            }
        }
    }
    tx.commit().await?;

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
    let user_to_add = UserStore::find_by_email(pool, new_user_email)
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
    let collaborator_to_remove =
        AlbumStore::find_collaborator_by_id(pool, collaborator_id_to_remove)
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
    thumbnail_id: Option<String>,
    is_public: Option<bool>,
) -> Result<Album, AlbumError> {
    // Permission Check: Only the owner can update album details.
    if !is_album_owner(pool, user_id, album_id).await? {
        return Err(AlbumError::NotFound(
            "Album not found or permission denied.".to_string(),
        ));
    }

    // At least one field must be provided for the update.
    if name.is_none() && description.is_none() && thumbnail_id.is_none() && is_public.is_none() {
        let album = AlbumStore::find_by_id(pool, album_id)
            .await?
            .ok_or_else(|| AlbumError::NotFound(album_id.to_owned()))?;
        return Ok(album.into());
    }

    if let Some(thumbnail_id) = &thumbnail_id {
        let exists = AlbumStore::has_media_item(pool, album_id, thumbnail_id).await?;
        if !exists {
            return Err(AlbumError::BadRequest(
                "thumbnail_id is not in the album".to_owned(),
            ));
        }
    }

    let updated_album =
        AlbumStore::update(pool, album_id, name, description, thumbnail_id, is_public).await?;
    Ok(updated_album)
}

#[instrument(skip(pool))]
pub async fn generate_invite(
    pool: &PgPool,
    public_url: String,
    jwt_secret: String,
    album_id: &str,
    user_id: i32,
    user_name: &str,
) -> Result<String, AlbumError> {
    // Permission Check: Only the owner can generate an invite.
    if !is_album_owner(pool, user_id, album_id).await? {
        return Err(AlbumError::Forbidden(
            "Only the album owner can generate an invitation.".to_string(),
        ));
    }

    let expires_at = (Utc::now()
        + Duration::minutes(constants().auth.album_invitation_expiry_minutes))
    .timestamp();

    let claims = AlbumShareClaims {
        iss: public_url.clone(),
        sub: album_id.to_owned(),
        exp: expires_at,
        sharer_username: user_name.to_owned(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )?;

    Ok(token)
}

/// Accepts an album invitation and enqueues a background job to start the import.
pub async fn accept_invite(
    pool: &PgPool,
    settings: &AppSettings,
    s2s_client: &S2SClient,
    user_id: i32,
    payload: AcceptInviteRequest,
) -> Result<Album, AlbumError> {
    let jwt_secret = &settings.secrets.jwt;
    let claims = extract_token_claims(&payload.token, jwt_secret)
        .map_err(|_| AlbumError::Forbidden("Invalid token.".to_string()))?;

    let summary: AlbumSummary = s2s_client
        .get_album_invite_summary(&payload.token, jwt_secret)
        .await
        .wrap_err("Failed to get album invite summary from remote server")?;

    // 2. Create the new album locally
    let album_id = nice_id(constants().database.album_id_length);
    let mut tx = pool.begin().await?;
    let album = AlbumStore::create(
        &mut *tx,
        &album_id,
        user_id,
        &payload.name,
        payload.description,
        None,
        false,
    )
    .await?;
    AlbumStore::upsert_collaborator(&mut *tx, &album_id, user_id, AlbumRole::Owner).await?;
    tx.commit().await?;

    // 3. For each media item, enqueue a download & import job
    for relative_path in summary.relative_paths {
        let item_payload = ImportAlbumItemPayload {
            remote_relative_path: relative_path,
            local_album_id: album_id.clone(),
            remote_username: claims.sharer_username.clone(),
            remote_url: claims.iss.parse()?,
            token: payload.token.clone(),
        };

        enqueue_job(pool, settings, JobType::ImportAlbumItem)
            .user_id(user_id)
            .payload(&item_payload)
            .call()
            .await?;
    }

    Ok(album)
}

// ====================================== //
// === --- === ALBUM TIMELINE === --- === //
// ====================================== //

/// Fetches lightweight album metadata and the timeline ratios.
#[instrument(skip(pool))]
pub async fn get_album_ratios(
    pool: &PgPool,
    album_id: &str,
    user_id: Option<i32>,
    sort_direction: SortDirection,
) -> Result<AlbumRatiosResponse, AlbumError> {
    // 1. Permission Check
    let Some(album) = AlbumStore::find_by_id(pool, album_id).await? else {
        return Err(AlbumError::NotFound(album_id.to_owned()));
    };

    let mut user_role_str = None;

    if !album.is_public {
        let Some(uid) = user_id else {
            return Err(AlbumError::NotFound(album_id.to_string()));
        };
        // Check role directly so we can return it in metadata
        let role = AlbumStore::find_user_role(pool, album_id, uid).await?;
        if role.is_none() {
            return Err(AlbumError::NotFound(album_id.to_string()));
        }
        user_role_str = role.map(|r| r.to_string());
    } else if let Some(uid) = user_id {
        // Even if public, try to find role for UI purposes (e.g. show edit buttons)
        let role = AlbumStore::find_user_role(pool, album_id, uid).await?;
        user_role_str = role.map(|r| r.to_string());
    }

    // 2. Fetch Ratios (Mirrors timeline service but with JOIN)
    let sql = format!(
        r"
        SELECT
            m.month_id::TEXT as month_id,
            COUNT(*)::INT AS count,
            array_agg(m.width::real / m.height::real ORDER BY m.sort_timestamp {0}) AS ratios
        FROM media_item m
        JOIN album_media_item ami ON m.id = ami.media_item_id
        WHERE ami.album_id = $1
          AND m.deleted = false
        GROUP BY m.month_id
        ORDER BY m.month_id {0}
        ",
        sort_direction.as_sql()
    );

    let months = sqlx::query_as::<_, TimelineMonthRatios>(&sql)
        .bind(album_id)
        .fetch_all(pool)
        .await?;

    // 3. Construct Response
    let album_info = AlbumInfo {
        id: album.id,
        name: album.name,
        description: album.description,
        is_public: album.is_public,
        owner_id: album.owner_id,
        created_at: album.created_at.to_rfc3339(),
        thumbnail_id: album.thumbnail_id,
        user_role: user_role_str,
    };

    Ok(AlbumRatiosResponse {
        album: Some(album_info),
        months,
    })
}

/// Fetches media item IDs for an album (Timeline style).
#[instrument(skip(pool))]
pub async fn get_album_ids(
    pool: &PgPool,
    album_id: &str,
    user_id: Option<i32>,
    sort_direction: SortDirection,
) -> Result<Vec<String>, AlbumError> {
    // Permission Check
    let Some(album) = AlbumStore::find_by_id(pool, album_id).await? else {
        return Err(AlbumError::NotFound(album_id.to_owned()));
    };
    if !album.is_public {
        let Some(uid) = user_id else {
            return Err(AlbumError::NotFound(album_id.to_string()));
        };
        if !can_view_album(pool, uid, album_id).await? {
            return Err(AlbumError::NotFound(album_id.to_string()));
        }
    }

    let sql = format!(
        r"
        SELECT COALESCE(array_agg(m.id ORDER BY m.sort_timestamp {}), '{{}}')
        FROM media_item m
        JOIN album_media_item ami ON m.id = ami.media_item_id
        WHERE ami.album_id = $1 AND m.deleted = false
        ",
        sort_direction.as_sql()
    );

    let ids = sqlx::query_scalar::<_, Vec<String>>(&sql)
        .bind(album_id)
        .fetch_one(pool)
        .await?;

    Ok(ids)
}

/// Fetches media items for an album by month (Timeline style).
#[instrument(skip(pool))]
pub async fn get_album_photos_by_month(
    pool: &PgPool,
    album_id: &str,
    user_id: Option<i32>,
    month_ids: &[NaiveDate],
    sort_direction: SortDirection,
) -> Result<TimelineItemsResponse, AlbumError> {
    // Permission Check
    let Some(album) = AlbumStore::find_by_id(pool, album_id).await? else {
        return Err(AlbumError::NotFound(album_id.to_owned()));
    };
    if !album.is_public {
        let Some(uid) = user_id else {
            return Err(AlbumError::NotFound(album_id.to_string()));
        };
        if !can_view_album(pool, uid, album_id).await? {
            return Err(AlbumError::NotFound(album_id.to_string()));
        }
    }

    let sql = format!(
        r"
        SELECT
            m.id,
            m.is_video,
            m.use_panorama_viewer as is_panorama,
            m.duration_ms::INT as duration_ms,
            m.taken_at_local::TEXT as timestamp
        FROM
            media_item m
        JOIN album_media_item ami ON m.id = ami.media_item_id
        WHERE
            ami.album_id = $1
            AND m.deleted = false
            AND m.month_id = ANY($2)
        ORDER BY
            m.sort_timestamp {}
        ",
        sort_direction.as_sql()
    );

    let items = sqlx::query_as::<_, TimelineItem>(&sql)
        .bind(album_id)
        .bind(month_ids)
        .fetch_all(pool)
        .await?;

    // Grouping logic (Same as timeline service)
    let mut months_map: HashMap<String, Vec<TimelineItem>> = HashMap::new();
    for item in items {
        if item.timestamp.len() >= 7 {
            let month_id = format!("{}-01", &item.timestamp[0..7]);
            months_map.entry(month_id).or_default().push(item);
        }
    }

    let months = months_map
        .into_iter()
        .map(|(month_id, items)| TimelineMonthItems { month_id, items })
        .collect();

    Ok(TimelineItemsResponse { months })
}
