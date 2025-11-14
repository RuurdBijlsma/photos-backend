use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the 'weather' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Weather {
    pub temperature: Option<f32>,
    pub dew_point: Option<f32>,
    pub relative_humidity: Option<i32>,
    pub precipitation: Option<f32>,
    pub snow: Option<i32>,
    pub wind_direction: Option<i32>,
    pub wind_speed: Option<f32>,
    pub peak_wind_gust: Option<f32>,
    pub pressure: Option<f32>,
    pub sunshine_minutes: Option<i32>,
    pub condition: Option<String>,
    pub sunrise: Option<DateTime<Utc>>,
    pub sunset: Option<DateTime<Utc>>,
    pub dawn: Option<DateTime<Utc>>,
    pub dusk: Option<DateTime<Utc>>,
    pub is_daytime: Option<bool>,
}
