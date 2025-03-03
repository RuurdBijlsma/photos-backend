pub use super::_entities::weather::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::WeatherData;
use crate::common::image_utils::parse_iso_datetime;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type Weather = Entity;

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
    pub async fn create_weather_from_analysis<C>(
        db: &C,
        weather: WeatherData,
        image_id: String,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let recorded_at = weather
            .weather_recorded_at
            .as_ref()
            .and_then(|ts| parse_iso_datetime(ts).ok());
        let weather = ActiveModel {
            weather_recorded_at: Set(recorded_at),
            weather_temperature: Set(weather.weather_temperature),
            weather_dewpoint: Set(weather.weather_dewpoint),
            weather_relative_humidity: Set(weather.weather_relative_humidity),
            weather_precipitation: Set(weather.weather_precipitation),
            weather_wind_gust: Set(weather.weather_wind_gust),
            weather_pressure: Set(weather.weather_pressure),
            weather_sun_hours: Set(weather.weather_sun_hours),
            weather_condition: Set(weather.weather_condition.map(|c| c.to_string())),
            image_id: Set(image_id.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(weather)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
