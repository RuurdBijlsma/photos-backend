use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`color_data`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct ColorData {
    pub themes: Vec<Value>,
    pub prominent_colors: Option<Vec<String>>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
    pub histogram: Option<Value>,
}
