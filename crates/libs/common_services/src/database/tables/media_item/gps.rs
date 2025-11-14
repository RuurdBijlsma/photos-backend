use crate::database::media_item::location::Location;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// A composite struct representing data from the 'gps' table, with its associated 'location' data nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Gps {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub image_direction: Option<f64>,
    pub location: Option<Location>,
}
