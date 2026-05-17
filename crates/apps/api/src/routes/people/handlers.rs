use crate::api_state::ApiContext;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use common_services::api::people::error::PeopleError;
use common_services::api::people::interfaces::UpdatePersonRequest;
use common_services::api::people::service::{get_all_people, get_person_photos, update_person};
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
    update_person(&context.pool, &person_id, user.id, payload.name).await?;
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
    let cluster_id = sqlx::query_scalar!(
        "SELECT id FROM face_cluster WHERE person_id = $1",
        person_id
    )
        .fetch_one(&context.pool)
        .await?;

    let target_url = format!("/thumbnails/{FACE_CLUSTERS_FOLDER}/{cluster_id}.webp");
    let headers = [(CACHE_CONTROL, "public, max-age=86400")];
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
