use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the 'location' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Location {
    pub name: Option<String>,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub country_code: String,
    pub country_name: String,
}
