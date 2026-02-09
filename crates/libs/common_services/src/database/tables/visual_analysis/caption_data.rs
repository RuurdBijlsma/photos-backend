use common_types::ml_analysis::LlmClassification;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Corresponds to the '`caption_data`' table.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ClassificationData {
    pub caption: Option<String>,
    pub main_subject: Option<String>,
    pub contains_pets: bool,
    pub contains_vehicle: bool,
    pub contains_landmarks: bool,
    pub contains_people: bool,
    pub contains_animals: bool,
    pub contains_text: bool,
    pub is_indoor: bool,
    pub is_food: bool,
    pub is_drink: bool,
    pub is_event: bool,
    pub is_document: bool,
    pub is_landscape: bool,
    pub is_cityscape: bool,
    pub is_activity: bool,
    pub setting: String,
    pub ocr_text: Option<String>,
    pub animal_type: Option<String>,
    pub food_name: Option<String>,
    pub drink_name: Option<String>,
    pub vehicle_type: Option<String>,
    pub event_type: Option<String>,
    pub landmark_name: Option<String>,
    pub document_type: Option<String>,
    pub people_count: Option<i32>,
    pub people_mood: Option<String>,
    pub photo_type: Option<String>,
    pub activity_description: Option<String>,
}

impl From<LlmClassification> for ClassificationData {
    fn from(caption_data: LlmClassification) -> Self {
        Self {
            caption: Some(caption_data.caption),
            main_subject: Some(caption_data.main_subject),
            contains_pets: caption_data.contains_pets,
            contains_vehicle: caption_data.contains_vehicle,
            contains_landmarks: caption_data.contains_landmarks,
            contains_people: caption_data.contains_people,
            contains_animals: caption_data.contains_animals,
            contains_text: caption_data.contains_text,
            is_indoor: caption_data.is_indoor,
            is_food: caption_data.is_food,
            is_drink: caption_data.is_drink,
            is_event: caption_data.is_event,
            is_document: caption_data.is_document,
            is_landscape: caption_data.is_landscape,
            is_cityscape: caption_data.is_cityscape,
            is_activity: caption_data.is_activity,
            setting: caption_data.setting,
            ocr_text: caption_data.ocr_text,
            animal_type: caption_data.animal_type,
            food_name: caption_data.food_name,
            drink_name: caption_data.drink_name,
            vehicle_type: caption_data.vehicle_type,
            event_type: caption_data.event_type,
            landmark_name: caption_data.landmark_name,
            document_type: caption_data.document_type,
            people_count: caption_data.people_count,
            people_mood: caption_data.people_mood,
            photo_type: caption_data.photo_type,
            activity_description: caption_data.activity_name,
        }
    }
}
