use crate::api_state::ApiContext;
use axum::{Extension, Json, extract};
use axum_extra::protobuf::Protobuf;
use common_services::api::people::error::PeopleError;
use common_services::api::people::interfaces::UpdatePersonRequest;
use common_services::api::people::service::{get_all_people, get_person_photos, update_person};
use common_services::database::app_user::User;
use common_types::pb::api::{FullPersonMediaResponse, ListPeopleResponse};
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
    extract::State(context): extract::State<ApiContext>,
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
    extract::State(context): extract::State<ApiContext>,
    Extension(user): Extension<User>,
    extract::Path(person_id): extract::Path<i64>,
    Json(payload): Json<UpdatePersonRequest>,
) -> Result<(), PeopleError> {
    update_person(&context.pool, person_id, user.id, payload.name).await?;
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
    extract::State(context): extract::State<ApiContext>,
    Extension(user): Extension<User>,
    extract::Path(person_id): extract::Path<i64>,
) -> Result<Protobuf<FullPersonMediaResponse>, PeopleError> {
    let result = get_person_photos(&context.pool, person_id, user.id).await?;
    Ok(Protobuf(result))
}
