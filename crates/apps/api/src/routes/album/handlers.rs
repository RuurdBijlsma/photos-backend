use crate::api_state::ApiContext;
use crate::auth::middlewares::optional_user::OptionalUser;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use common_services::api::album::error::AlbumError;
use common_services::api::album::interfaces::{
    AcceptInviteRequest, AddCollaboratorRequest, AddMediaToAlbumRequest, CheckInviteRequest,
    CreateAlbumRequest, GetAlbumMediaParams, ListAlbumsParam, UpdateAlbumRequest,
};
use common_services::api::album::service::{
    accept_invite, add_collaborator, add_media_to_album, create_album, generate_invite,
    get_album_ids, get_album_media, get_album_media_by_groups, get_album_ratios,
    remove_collaborator, remove_media_from_album, update_album,
};
use common_services::database::album::album::{Album, AlbumSummary, AlbumWithCount};
use common_services::database::album::album_collaborator::AlbumCollaborator;
use common_services::database::album_store::AlbumStore;
use common_services::database::app_user::User;
use common_types::pb::api::{AlbumMediaResponse, AlbumRatiosResponse, FullAlbumMediaResponse};
use tracing::{info, instrument};

/// Create a new album.
///
/// The user creating the album will be designated as the owner.
#[utoipa::path(
    post,
    path = "/album",
    tag = "Album",
    request_body = CreateAlbumRequest,
    responses(
        (status = 201, description = "Album created successfully.", body = Album),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn create_album_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateAlbumRequest>,
) -> Result<(StatusCode, Json<Album>), AlbumError> {
    info!("Create album handler {:?}", payload);
    let album = create_album(
        &context.pool,
        user.id,
        &payload.name,
        payload.description,
        payload.is_public,
        &payload.media_item_ids,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(album)))
}

/// List all albums for the current user.
///
/// Returns all albums where the user is a collaborator (owner, contributor, or viewer).
#[utoipa::path(
    get,
    path = "/album",
    tag = "Album",
    responses(
        (status = 200, description = "A list of the user's albums.", body = Vec<AlbumWithCount>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn get_user_albums_handler(
    State(context): State<ApiContext>,
    Query(query): Query<ListAlbumsParam>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<AlbumWithCount>>, AlbumError> {
    let albums = AlbumStore::list_with_count_by_user_id(
        &context.pool,
        user.id,
        query.sort_field,
        query.sort_direction,
    )
    .await?;
    Ok(Json(albums))
}

/// Update an album's details.
///
/// Allows updating the name and/or description. The user must be the album owner.
#[utoipa::path(
    put,
    path = "/album/{album_id}",
    tag = "Album",
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
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
    Json(payload): Json<UpdateAlbumRequest>,
) -> Result<Json<Album>, AlbumError> {
    let album = update_album(
        &context.pool,
        &album_id,
        user.id,
        payload.name,
        payload.description,
        payload.thumbnail_id,
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
    path = "/album/{album_id}/media",
    tag = "Album",
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
#[instrument(skip(context, user), err(Debug))]
pub async fn add_media_to_album_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
    Json(payload): Json<AddMediaToAlbumRequest>,
) -> Result<StatusCode, AlbumError> {
    add_media_to_album(&context.pool, &album_id, &payload.media_item_ids, user.id).await?;
    Ok(StatusCode::OK)
}

/// Remove a media item from an album.
///
/// The user must be an owner or contributor of the album.
#[utoipa::path(
    delete,
    path = "/album/{album_id}/media/{media_item_id}",
    tag = "Album",
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
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path((album_id, media_item_id)): Path<(String, String)>,
) -> Result<StatusCode, AlbumError> {
    remove_media_from_album(&context.pool, &album_id, &media_item_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add a collaborator to an album.
///
/// The inviting user must be the album owner.
#[utoipa::path(
    post,
    path = "/album/{album_id}/collaborators",
    tag = "Album",
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
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
    Json(payload): Json<AddCollaboratorRequest>,
) -> Result<Json<AlbumCollaborator>, AlbumError> {
    let collaborator = add_collaborator(
        &context.pool,
        &album_id,
        &payload.user_email,
        payload.role,
        user.id,
    )
    .await?;
    Ok(Json(collaborator))
}

/// Remove a collaborator from an album.
///
/// The user performing the action must be the album owner. The owner cannot be removed.
#[utoipa::path(
    delete,
    path = "/album/{album_id}/collaborators/{collaborator_id}",
    tag = "Album",
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
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path((album_id, collaborator_id)): Path<(String, i64)>,
) -> Result<StatusCode, AlbumError> {
    remove_collaborator(&context.pool, &album_id, collaborator_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Generate a cross-server invitation link for an album.
///
/// The inviting user must be the album owner. The generated token has a configurable expiry time.
#[utoipa::path(
    get,
    path = "/album/{album_id}/invite",
    tag = "Album",
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
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(album_id): Path<String>,
) -> Result<Json<String>, AlbumError> {
    let token = generate_invite(
        &context.pool,
        context.settings.api.public_url,
        context.settings.secrets.jwt,
        &album_id,
        user.id,
        &user.name,
    )
    .await?;
    Ok(Json(token))
}

#[utoipa::path(
    post,
    path = "/album/invite/check",
    tag = "Album",
    request_body = CheckInviteRequest,
    responses(
        (status = 200, description = "Invitation summary retrieved successfully.", body = AlbumSummary),
        (status = 400, description = "The invitation token is malformed."),
        (status = 502, description = "The remote server could not be reached or returned an error."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn check_invite_handler(
    State(context): State<ApiContext>,
    Json(payload): Json<CheckInviteRequest>,
) -> Result<Json<AlbumSummary>, AlbumError> {
    let summary = context
        .s2s_client
        .get_album_invite_summary(&payload.token, &context.settings.secrets.jwt)
        .await?;
    Ok(Json(summary))
}

/// Accept an album invitation.
///
/// This will enqueue a background job to begin the process of importing the album
/// from the remote server.
#[utoipa::path(
    post,
    path = "/album/invite/accept",
    tag = "Album",
    request_body = AcceptInviteRequest,
    responses(
        (status = 202, description = "Album import process has been started."),
        (status = 400, description = "The invitation token is malformed."),
        (status = 500, description = "A database error occurred while creating the job."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_invite_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Json(payload): Json<AcceptInviteRequest>,
) -> Result<Json<Album>, AlbumError> {
    let album = accept_invite(
        &context.pool,
        &context.settings,
        &context.s2s_client,
        user.id,
        payload,
    )
    .await?;
    Ok(Json(album))
}

// ====================================== //
// === --- === ALBUM TIMELINE === --- === //
// ====================================== //

/// Get album metadata and timeline ratios (grouped by rank).
/// Replaces the need for a heavy initial load.
#[utoipa::path(
    get,
    path = "/album/{album_id}/ratios",
    tag = "Album",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album.")
    ),
    responses(
        (status = 200, description = "Album metadata and media ratios.", body = AlbumRatiosResponse),
        (status = 404, description = "Album not found or permission denied."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = [])) // Optional auth handled inside service
)]
pub async fn get_album_ratios_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<OptionalUser>,
    Path(album_id): Path<String>,
) -> Result<Protobuf<AlbumRatiosResponse>, AlbumError> {
    let response = get_album_ratios(&context.pool, &album_id, user.0.map(|u| u.id)).await?;
    Ok(Protobuf(response))
}

/// Get all media IDs for an album.
#[utoipa::path(
    get,
    path = "/album/{album_id}/ids",
    tag = "Album",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album.")
    ),
    responses(
        (status = 200, description = "List of media IDs in the album.", body = Vec<String>),
        (status = 404, description = "Album not found."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_album_ids_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<OptionalUser>,
    Path(album_id): Path<String>,
) -> Result<Json<Vec<String>>, AlbumError> {
    let ids = get_album_ids(&context.pool, &album_id, user.0.map(|u| u.id)).await?;
    Ok(Json(ids))
}

/// Get media items for specific rank groups within an album.
#[utoipa::path(
    get,
    path = "/album/{album_id}/by-groups",
    tag = "Album",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album."),
        GetAlbumMediaParams
    ),
    responses(
        (status = 200, description = "Media items for the requested groups.", body = AlbumMediaResponse),
        (status = 500, description = "Internal Error."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_album_media_by_groups_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<OptionalUser>,
    Path(album_id): Path<String>,
    Query(params): Query<GetAlbumMediaParams>,
) -> Result<Protobuf<AlbumMediaResponse>, AlbumError> {
    let group_ids = params
        .groups
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<f64>().map_err(|_| {
                AlbumError::BadRequest(format!(
                    "Invalid rank format '{s}' in 'groups' parameter. Must be float."
                ))
            })
        })
        .collect::<Result<Vec<f64>, _>>()?;

    let response =
        get_album_media_by_groups(&context.pool, &album_id, user.0.map(|u| u.id), &group_ids)
            .await?;
    Ok(Protobuf(response))
}

/// Get media items for specific rank groups within an album.
#[utoipa::path(
    get,
    path = "/album/{album_id}/media",
    tag = "Album",
    params(
        ("album_id" = String, Path, description = "The unique ID of the album."),
    ),
    responses(
        (status = 200, description = "Media items for the requested groups.", body = AlbumMediaResponse),
        (status = 500, description = "Internal Error."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_album_media_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<OptionalUser>,
    Path(album_id): Path<String>,
) -> Result<Protobuf<FullAlbumMediaResponse>, AlbumError> {
    let response = get_album_media(&context.pool, &album_id, user.0.map(|u| u.id)).await?;
    Ok(Protobuf(response))
}
