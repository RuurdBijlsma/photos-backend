use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyCardResponse {
    pub id: i32,
    pub user_id: i32,
    pub card_date: Option<NaiveDate>,
    pub card_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub thumbnail_media_item_id: Option<String>,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub shown: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DailyCardsQueryParams {
    pub date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidateMediaRequest {
    pub media_item_ids: Vec<String>,
}
