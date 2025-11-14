use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`quality_data`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct QualityData {
    pub blurriness: f64,
    pub noisiness: f64,
    pub exposure: f64,
    pub quality_score: f64,
}