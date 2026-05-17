use super::error::PeopleError;
use crate::database::person_store::PersonStore;
use common_types::pb::api::{FullPersonMediaResponse, ListPeopleResponse, PersonInfo};
use sqlx::PgPool;
use tracing::instrument;
use crate::api::people::interfaces::UpdatePersonRequest;

#[instrument(skip(pool))]
pub async fn get_all_people(
    pool: &PgPool,
    user_id: i32,
) -> Result<ListPeopleResponse, PeopleError> {
    let people = PersonStore::list_by_user_id(pool, user_id).await?;
    let people_pb = people
        .into_iter()
        .map(|p| PersonInfo {
            id: p.id,
            name: p.name,
            photo_count: p.photo_count,
            face_thumb_id: p.face_thumb_id,
            face_cluster_ids: p.face_cluster_ids,
        })
        .collect();

    Ok(ListPeopleResponse { people: people_pb })
}

#[instrument(skip(pool))]
pub async fn update_person(
    pool: &PgPool,
    person_id: &str,
    user_id: i32,
    payload: &UpdatePersonRequest,
) -> Result<(), PeopleError> {
    let rows = PersonStore::update(pool, person_id, user_id, payload).await?;
    if rows == 0 {
        return Err(PeopleError::NotFound(person_id.to_string()));
    }
    Ok(())
}

#[instrument(skip(pool))]
pub async fn get_person_photos(
    pool: &PgPool,
    person_id: &str,
    user_id: i32,
) -> Result<FullPersonMediaResponse, PeopleError> {
    let person = PersonStore::find_by_id(pool, person_id, user_id)
        .await?
        .ok_or_else(|| PeopleError::NotFound(person_id.to_string()))?;

    let items = PersonStore::get_person_media_items(pool, person_id, user_id).await?;

    Ok(FullPersonMediaResponse {
        person: Some(PersonInfo {
            id: person.id,
            name: person.name,
            photo_count: person.photo_count,
            face_thumb_id: person.face_thumb_id,
            face_cluster_ids: person.face_cluster_ids,
        }),
        items,
    })
}
