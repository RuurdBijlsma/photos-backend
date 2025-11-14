use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`time_details`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct TimeDetails {
    pub timezone_name: Option<String>,
    pub timezone_offset_seconds: Option<i32>,
    pub source: Option<String>,
    pub source_details: Option<String>,
    pub source_confidence: Option<String>,
}
