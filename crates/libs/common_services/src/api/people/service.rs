use super::error::PeopleError;
use crate::database::person_store::PersonStore;
use common_types::pb::api::{FullPersonMediaResponse, ListPeopleResponse, PersonInfo};
use sqlx::PgPool;
use tracing::instrument;

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
            thumbnail_id: p.thumbnail_media_item_id,
        })
        .collect();

    Ok(ListPeopleResponse { people: people_pb })
}

#[instrument(skip(pool))]
pub async fn update_person(
    pool: &PgPool,
    person_id: i64,
    user_id: i32,
    name: Option<String>,
) -> Result<(), PeopleError> {
    let rows = PersonStore::update_name(pool, person_id, user_id, name).await?;
    if rows == 0 {
        return Err(PeopleError::NotFound(person_id));
    }
    Ok(())
}

#[instrument(skip(pool))]
pub async fn get_person_photos(
    pool: &PgPool,
    person_id: i64,
    user_id: i32,
) -> Result<FullPersonMediaResponse, PeopleError> {
    let person = PersonStore::find_by_id(pool, person_id, user_id)
        .await?
        .ok_or(PeopleError::NotFound(person_id))?;

    let items = PersonStore::get_person_media_items(pool, person_id, user_id).await?;

    Ok(FullPersonMediaResponse {
        person: Some(PersonInfo {
            id: person.id,
            name: person.name,
            photo_count: person.photo_count,
            thumbnail_id: person.thumbnail_media_item_id,
        }),
        items,
    })
}
