pub use super::_entities::locations::{ActiveModel, Entity, Model};
use crate::models::_entities::locations::Column;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type Locations = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {
    pub async fn find_or_create_location<C>(
        db: &C,
        country: String,
        province: Option<String>,
        city: String,
        latitude: f32,
        longitude: f32,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        // Try to find the location first
        let location = Entity::find()
            .filter(Column::Country.eq(&country))
            .filter(Column::City.eq(&city))
            .one(db)
            .await?;

        match location {
            Some(location) => Ok(location), // If found, return it
            None => {
                // If not found, create a new location
                let new_location = ActiveModel {
                    country: Set(country),
                    province: Set(province),
                    city: Set(city),
                    latitude: Set(latitude),
                    longitude: Set(longitude),
                    ..Default::default()
                };

                let result = new_location.insert(db).await?;
                Ok(result)
            }
        }
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
