use crate::api_state::ApiContext;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use common_services::api::people::error::PeopleError;
use common_services::api::people::interfaces::{MergePersonRequest, UpdatePersonRequest};
use common_services::api::people::service::{
    get_all_people, get_person_photos, merge_person, unmerge_person, update_person,
};
use common_services::database::app_user::User;
use common_types::constants::FACE_CLUSTERS_FOLDER;
use common_types::pb::api::{FullPersonMediaResponse, ListPeopleResponse};
use http::header::CACHE_CONTROL;
use tracing::instrument;

#[utoipa::path(
    get,
    path = "/people",
    tag = "People",
    responses(
        (status = 200, description = "List all identified people clusters.", body = ListPeopleResponse),
        (status = 500, description = "Internal server error."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn list_people_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Protobuf<ListPeopleResponse>, PeopleError> {
    let result = get_all_people(&context.pool, user.id).await?;
    Ok(Protobuf(result))
}

#[utoipa::path(
    patch,
    path = "/people/{id}",
    tag = "People",
    params(
        ("id" = i64, Path, description = "Person ID")
    ),
    responses(
        (status = 200, description = "Person label updated successfully."),
        (status = 404, description = "Person not found."),
        (status = 500, description = "Internal server error."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn update_person_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(person_id): Path<String>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<(), PeopleError> {
    update_person(&context.pool, &person_id, user.id, &payload).await?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/people/{id}/merge",
    tag = "People",
    params(
        ("id" = String, Path, description = "Person ID to keep")
    ),
    request_body = MergePersonRequest,
    responses(
        (status = 200, description = "People merged successfully."),
        (status = 404, description = "Person not found."),
        (status = 500, description = "Internal server error."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn merge_person_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(person_id): Path<String>,
    Json(payload): Json<MergePersonRequest>,
) -> Result<(), PeopleError> {
    merge_person(&context.pool, &person_id, user.id, &payload).await?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/people/{id}/unmerge",
    tag = "People",
    params(
        ("id" = String, Path, description = "Person ID to split")
    ),
    responses(
        (status = 200, description = "Person split successfully."),
        (status = 404, description = "Person not found."),
        (status = 500, description = "Internal server error."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn unmerge_person_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(person_id): Path<String>,
) -> Result<(), PeopleError> {
    unmerge_person(&context.pool, &person_id, user.id).await?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/people/{id}/photos",
    tag = "People",
    params(
        ("id" = i64, Path, description = "Person ID")
    ),
    responses(
        (status = 200, description = "Get all photos of a person.", body = FullPersonMediaResponse),
        (status = 404, description = "Person not found."),
        (status = 500, description = "Internal server error."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn get_person_photos_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(person_id): Path<String>,
) -> Result<Protobuf<FullPersonMediaResponse>, PeopleError> {
    let result = get_person_photos(&context.pool, &person_id, user.id).await?;
    Ok(Protobuf(result))
}

pub async fn get_person_thumbnail_redirect_handler(
    State(context): State<ApiContext>,
    Path(person_id): Path<String>,
) -> Result<impl IntoResponse, PeopleError> {
    let cluster_id = if let Some(db_face) =
        sqlx::query_scalar!("SELECT face_thumb_id FROM person WHERE id = $1", person_id)
            .fetch_one(&context.pool)
            .await?
    {
        db_face
    } else {
        sqlx::query_scalar!(
            "SELECT id FROM face_cluster WHERE person_id = $1",
            person_id
        )
        .fetch_one(&context.pool)
        .await?
    };

    let target_url = format!("/thumbnails/{FACE_CLUSTERS_FOLDER}/{cluster_id}.webp");
    let headers = [(CACHE_CONTROL, "public, max-age=300")];
    Ok((headers, Redirect::temporary(&target_url)))
}

#[instrument(skip(context, user), err(Debug))]
pub async fn get_person_media_item_id(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(person_id): Path<String>,
) -> Result<Json<String>, PeopleError> {
    let Some(result) = sqlx::query_scalar!(
        "SELECT thumb_media_item_id FROM face_cluster WHERE person_id = $1 AND user_id = $2 AND thumb_media_item_id IS NOT NULL",
        person_id, user.id
    )
        .fetch_one(&context.pool)
        .await? else {
        return Err(PeopleError::NotFound(person_id));
    };
    Ok(Json(result))
}
