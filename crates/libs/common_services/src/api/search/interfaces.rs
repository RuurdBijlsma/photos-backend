use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilterRanges {
    pub available_months: Vec<NaiveDate>,
    pub people: Vec<(String, String)>,
    pub countries: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct SearchMediaConfig {
    pub embedder_model_id: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub semantic_score_threshold: f64,
    pub semantic_weight: f64,
    pub text_weight: f64,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub media_type: SearchMediaType,
    pub sort_by: SearchSortBy,
    pub negative_query: Option<String>,
    pub country_codes: Vec<String>,
    pub face_names: Vec<String>,
    pub all_faces_required: bool,
}

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub media_type: SearchMediaType,
    #[serde(default)]
    pub sort_by: SearchSortBy,
    pub negative_query: Option<String>,
    pub country_codes: Option<String>,    // comma separated
    pub face_names: Option<String>,       // comma separated
    pub all_faces_required: Option<bool>, // all persons must be in the photo/video if true, otherwise just one of them
}

#[derive(Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchMediaType {
    #[default]
    All,
    Photo,
    Video,
}

#[derive(Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchSortBy {
    #[default]
    Relevancy,
    Date,
}
