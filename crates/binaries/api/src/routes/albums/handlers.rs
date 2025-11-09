use crate::auth::db_model::User;
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
use uuid::Uuid;

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
    let album = service::create_album(&pool, user.id, &payload.name, payload.description.as_deref()).await?;
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
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
) -> Result<Json<AlbumDetailsResponse>, AlbumsError> {
    let album_uuid = Uuid::parse_str(&album_id)?;
    let details = service::get_album_details(&pool, album_uuid, user.id).await?;
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
    let album_uuid = Uuid::parse_str(&album_id)?;
    let album = service::update_album(&pool, album_uuid, user.id, payload.name, payload.description).await?;
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
    let album_uuid = Uuid::parse_str(&album_id)?;
    service::add_media_to_album(&pool, album_uuid, &payload.media_item_ids, user.id).await?;
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
    let album_uuid = Uuid::parse_str(&album_id)?;
    service::remove_media_from_album(&pool, album_uuid, &media_item_id, user.id).await?;
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
    let album_uuid = Uuid::parse_str(&album_id)?;
    let collaborator = service::add_collaborator(&pool, album_uuid, &payload.user_email, payload.role, user.id).await?;
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
    let album_uuid = Uuid::parse_str(&album_id)?;
    service::remove_collaborator(&pool, album_uuid, collaborator_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}