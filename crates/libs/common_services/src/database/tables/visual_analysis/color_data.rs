use common_types::ml_analysis::PyColorData;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`color_data`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct ColorData {
    pub themes: Vec<Value>,
    pub prominent_colors: Vec<String>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
    pub histogram: Value,
}

impl From<PyColorData> for ColorData {
    fn from(color_data: PyColorData) -> Self {
        Self {
            themes: color_data.themes,
            prominent_colors: color_data.prominent_colors,
            average_hue: color_data.average_hue,
            average_saturation: color_data.average_saturation,
            average_lightness: color_data.average_lightness,
            // Serialize the nested histogram struct to a JSON Value
            histogram: json!(color_data.histogram),
        }
    }
}
