pub use super::_entities::gps::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::GPSData;
use crate::models::locations;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type Gps = Entity;

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
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
    /// Create gps and location based on `MediaAnalyzerOutput`, and store it in db.
    ///
    /// # Errors
    /// * If an INSERT fails.
    /// * If a query in find location fails.
    pub async fn create_from_analysis<C>(
        db: &C,
        gps_data: GPSData,
        image_id: String,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let location = locations::Model::find_or_create_location(
            db,
            gps_data.location.country,
            gps_data.location.province,
            gps_data.location.city,
            gps_data.latitude,
            gps_data.longitude,
        )
        .await?;

        let gps = Self {
            latitude: Set(gps_data.latitude),
            longitude: Set(gps_data.longitude),
            altitude: Set(gps_data.altitude),
            location_id: Set(location.id),
            image_id: Set(image_id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(gps)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
