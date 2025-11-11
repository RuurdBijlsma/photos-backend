use crate::auth::db_model::User;
use crate::auth::middleware::OptionalUser;
use crate::routes::albums::db_model::{Album, AlbumCollaborator};
use crate::routes::albums::error::AlbumsError;
use crate::routes::albums::interfaces::{
    AddCollaboratorRequest, AddMediaToAlbumRequest, AlbumDetailsResponse, CreateAlbumRequest,
    UpdateAlbumRequest,
};
use crate::routes::albums::service;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::PgPool;
use common_photos::InviteSummaryResponse;
use crate::albums::interfaces::{AcceptInviteRequest, CheckInviteRequest};

/// Create a new album.
///
/// The user creating the album will be designated as the owner.
#[utoipa::path(
    post,
    path = "/albums",
    tag = "Albums",
    request_body = CreateAlbumRequest,
    responses(
        (status = 201, description = "Album created successfully.", body = Album),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_album_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateAlbumRequest>,
) -> Result<(StatusCode, Json<Album>), AlbumsError> {
    let album = service::create_album(
        &pool,
        user.id,
        &payload.name,
        payload.description.as_deref(),
        payload.is_public,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(album)))
}

/// List all albums for the current user.
///
/// Returns all albums where the user is a collaborator (owner, contributor, or viewer).
#[utoipa::path(
    get,
    path = "/albums",
    tag = "Albums",
    responses(
        (status = 200, description = "A list of the user's albums.", body = Vec<Album>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user_albums_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<Album>>, AlbumsError> {
    let albums = service::get_user_albums(&pool, user.id).await?;
    Ok(Json(albums))
}

/// Get details for a specific album.
///
/// The user must be a collaborator on the album to view its details.
#[utoipa::path(
    get,
    path = "/albums/{album_id}",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album.")
    ),
    responses(
        (status = 200, description = "Detailed information about the album.", body = AlbumDetailsResponse),
        (status = 404, description = "Album not found or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_album_details_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<OptionalUser>,
    Path(album_id): Path<String>,
) -> Result<Json<AlbumDetailsResponse>, AlbumsError> {
    let details = service::get_album_details(&pool, &album_id, user.0.map(|u| u.id)).await?;
    Ok(Json(details))
}

/// Update an album's details.
///
/// Allows updating the name and/or description. The user must be the album owner.
#[utoipa::path(
    put,
    path = "/albums/{album_id}",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album to update.")
    ),
    request_body = UpdateAlbumRequest,
    responses(
        (status = 200, description = "Album updated successfully.", body = Album),
        (status = 404, description = "Album not found or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_album_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
    Json(payload): Json<UpdateAlbumRequest>,
) -> Result<Json<Album>, AlbumsError> {
    let album = service::update_album(
        &pool,
        &album_id,
        user.id,
        payload.name,
        payload.description,
        payload.is_public,
    )
    .await?;
    Ok(Json(album))
}

/// Add media items to an album.
///
/// The user must be an owner or contributor of the album.
#[utoipa::path(
    post,
    path = "/albums/{album_id}/media",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album.")
    ),
    request_body = AddMediaToAlbumRequest,
    responses(
        (status = 200, description = "Media items added successfully."),
        (status = 404, description = "Album not found or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_media_to_album_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
    Json(payload): Json<AddMediaToAlbumRequest>,
) -> Result<StatusCode, AlbumsError> {
    service::add_media_to_album(&pool, &album_id, &payload.media_item_ids, user.id).await?;
    Ok(StatusCode::OK)
}

/// Remove a media item from an album.
///
/// The user must be an owner or contributor of the album.
#[utoipa::path(
    delete,
    path = "/albums/{album_id}/media/{media_item_id}",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album."),
        ("media_item_id" = String, Path, description = "The ID of the media item to remove.")
    ),
    responses(
        (status = 204, description = "Media item removed successfully."),
        (status = 404, description = "Album or media item not found, or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn remove_media_from_album_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Path((album_id, media_item_id)): Path<(String, String)>,
) -> Result<StatusCode, AlbumsError> {
    service::remove_media_from_album(&pool, &album_id, &media_item_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add a collaborator to an album.
///
/// The inviting user must be the album owner.
#[utoipa::path(
    post,
    path = "/albums/{album_id}/collaborators",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album.")
    ),
    request_body = AddCollaboratorRequest,
    responses(
        (status = 200, description = "Collaborator added successfully.", body = AlbumCollaborator),
        (status = 404, description = "Album or user not found, or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_collaborator_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
    Json(payload): Json<AddCollaboratorRequest>,
) -> Result<Json<AlbumCollaborator>, AlbumsError> {
    let collaborator =
        service::add_collaborator(&pool, &album_id, &payload.user_email, payload.role, user.id)
            .await?;
    Ok(Json(collaborator))
}

/// Remove a collaborator from an album.
///
/// The user performing the action must be the album owner. The owner cannot be removed.
#[utoipa::path(
    delete,
    path = "/albums/{album_id}/collaborators/{collaborator_id}",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album."),
        ("collaborator_id" = i64, Path, description = "The numeric ID of the collaborator record.")
    ),
    responses(
        (status = 204, description = "Collaborator removed successfully."),
        (status = 404, description = "Album or collaborator not found, or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn remove_collaborator_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Path((album_id, collaborator_id)): Path<(String, i64)>,
) -> Result<StatusCode, AlbumsError> {
    service::remove_collaborator(&pool, &album_id, collaborator_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Generate a cross-server invitation link for an album.
///
/// The inviting user must be the album owner. The generated token has a configurable expiry time.
#[utoipa::path(
    get,
    path = "/albums/{album_id}/generate-invite",
    tag = "Albums",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album to share.")
    ),
    responses(
        (status = 200, description = "Invitation token generated successfully.", body = String),
        (status = 404, description = "Album not found."),
        (status = 401, description = "User is not the album owner."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn generate_invite_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
) -> Result<Json<String>, AlbumsError> {
    let token = service::generate_invite(&pool, &album_id, user.id, &user.name).await?;
    Ok(Json(token))
}

#[utoipa::path(
    post,
    path = "/albums/invite/check",
    tag = "Albums",
    request_body = CheckInviteRequest,
    responses(
        (status = 200, description = "Invitation summary retrieved successfully.", body = InviteSummaryResponse),
        (status = 400, description = "The invitation token is malformed."),
        (status = 502, description = "The remote server could not be reached or returned an error."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn check_invite_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CheckInviteRequest>,
) -> Result<Json<InviteSummaryResponse>, AlbumsError> {
    let summary = service::check_invite(&pool, &payload.token).await?;
    Ok(Json(summary))
}

/// Accept an album invitation.
///
/// This will enqueue a background job to begin the process of importing the album
/// from the remote server.
#[utoipa::path(
    post,
    path = "/albums/invite/accept",
    tag = "Albums",
    request_body = AcceptInviteRequest,
    responses(
        (status = 202, description = "Album import process has been started."),
        (status = 400, description = "The invitation token is malformed."),
        (status = 500, description = "A database error occurred while creating the job."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_invite_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<AcceptInviteRequest>,
) -> Result<StatusCode, AlbumsError> {
    service::accept_invite(&pool, user.id, &payload).await?;
    Ok(StatusCode::ACCEPTED)
}