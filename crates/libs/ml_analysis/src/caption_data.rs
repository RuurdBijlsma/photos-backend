use color_eyre::eyre::Result;
use common_types::ml_analysis::LlmCategorizationData;
use language_model::LlamaClient;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
pub struct RequiredOutput {
    pub caption: String,
    pub main_subject: String,
    pub setting: String,
    pub contains_pets: bool,
    pub contains_vehicle: bool,
    pub contains_landmarks: bool,
    pub contains_people: bool,
    pub contains_animals: bool,
    pub contains_text: bool,
    pub is_indoor: bool,
    pub is_food_or_drink: bool,
    pub is_event: bool,
    pub is_document: bool,
    pub is_landscape: bool,
    pub is_cityscape: bool,
    pub is_activity: bool,
}

pub async fn get_caption_data(
    llm: &LlamaClient,
    file: &Path,
) -> Result<Option<LlmCategorizationData>> {
    let stage1_schema = json!({
        "type": "object",
        "properties": {
            "caption": { "type": "string" },
            "main_subject": { "type": "string" },
            "setting": { "type": "string" },
            "contains_pets": { "type": "boolean", "description": "Is there a prominent pet shown in the photo? examples: [cat, dog, bird, etc.]" },
            "contains_animals": { "type": "boolean", "description": "Is an animal prominently shown in the photo? Examples: [lion, dog, deer, ant, spider]" },
            "contains_vehicle": { "type": "boolean", "description": "Is a vehicle shown prominently in the photo? Examples: [car, boat, bicycle, scooter]" },
            "contains_landmarks": { "type": "boolean", "description": "Is a landmark shown prominently in the photo? Examples: [Eiffel tower, notre dame, parthenon, statue of liberty, colosseum, golden gate bridge]" },
            "contains_people": { "type": "boolean", "description": "Is this a document, like a \
            passport, receipt, ticket, book, magazine, notes, payment card, id card, menu, \
            or recipe?"},
            "is_indoor": { "type": "boolean" },
            "is_food_or_drink": { "type": "boolean", "description": "Is this a photo of a food or drink item? Example: [spaghetti, pizza, cheeseburger, lasagna, cola, coffee, tea]" },
            "is_event": { "type": "boolean", "description": "Is this a specific event (e.g., \
            birthday party, wedding, concert, holiday)?" },
            "is_document": { "type": "boolean", "description": "Is this a document? Examples: [passport, receipt, ticket, book, magazine, notes, payment card, id card, menu, or recipe?]" },
            "is_activity": { "type": "boolean", "description": "Is a clear, intentional physical \
            action being performed in this photo (e.g., walking, cooking, exercising), not \
            including passive states such as sitting, standing, resting, posing, or watching?" },
            "contains_text": { "type": "boolean", "description": "Is there legible text visible in \
            this photo?" },
            "is_landscape": { "type": "boolean" },
            "is_cityscape": { "type": "boolean" },
        },
        "required": [
            "contains_pets", "contains_animals", "contains_vehicle", "contains_landmarks",
            "contains_people", "is_indoor", "is_food_or_drink", "is_event",
            "is_document", "is_activity", "contains_text", "is_landscape",
            "is_cityscape", "setting", "main_subject", "caption"
        ]
    });

    let s1_response = llm
        .chat(
            r"
Analyze this image and identify which categories it belongs to.
caption:
    You are an image captioning assistant. Describe the main
    content of this image in a short, factual caption suitable for search. Include the main
    objects, people, animals, and the setting or scene, but do not add opinions or creative
    interpretations. Make it concise and clear, suitable for full-text search. Answer in one
    to two paragraphs.

subject:
    Concisely name the single main subject of the photo. Answer in one word.

setting:
    Identify the main scene or environment of this photo. Focus only on the type of
    place or setting (e.g., kitchen, classroom, street, park, courtyard, office). Ignore
    people, animals, objects, and activities. Answer with a word or short phrase suitable for
    categorizing or grouping photos.
",
        )
        .images(&[file])
        .schema(stage1_schema)
        .call()
        .await?;
    let required: RequiredOutput = serde_json::from_str(&s1_response)?;

    let mut s2_props = json!({});
    let mut s2_required = vec![];

    // Conditionally add fields to the schema based on Stage 1
    if required.contains_pets {
        s2_props["pet_type"] = json!({ "type": "string", "description": "Kind of pet, answer broadly and concisely in one word. Examples: [cat, dog, mouse, guinea pig, snake, rat, goldfish]." });
        s2_required.push("pet_type".to_string());
    }
    if required.contains_animals {
        s2_props["animal_type"] = json!({ "type": "string", "description": "What animal is shown, give a singular term broadly naming the animal. Examples: [lion, giraffe, goose, fox, dog, ant]" });
        s2_required.push("animal_type".to_string());
    }
    if required.contains_vehicle {
        s2_props["vehicle_type"] = json!({ "type": "string", "description": "Name the singular main vehicle shown in the photo (keep to a single word, examples: [car, boat, kayak, bicycle, scooter, etc.]" });
        s2_required.push("vehicle_type".to_string());
    }
    if required.contains_landmarks {
        s2_props["landmark_name"] =
            json!({ "type": "string", "description": "Name of the famous place or landmark." });
        s2_required.push("landmark_name".to_string());
    }
    if required.contains_people {
        s2_props["people_count"] = json!({ "type": "integer" });
        s2_props["people_mood"] = json!({
            "type": "string",
            "enum": ["Happy", "Content", "Relaxed", "Calm", "Excited", "Playful", "Energetic", "Focused", "Thoughtful", "Neutral", "Serious", "Tired", "Bored", "Sad", "Melancholic", "Anxious", "Stressed", "Frustrated", "Angry", "Confident"]
        });
        s2_props["photo_type"] = json!({
            "type": "string",
            "enum": ["selfie", "group photo", "crowd", "portrait", "action", "candid", "other"]
        });
        s2_required.extend(vec![
            "people_count".into(),
            "people_mood".into(),
            "photo_type".into(),
        ]);
    }
    if required.is_food_or_drink {
        s2_props["food_or_drink_type"] = json!({ "type": "string", "description": "The main type of food or drink shown, examples: [cola, pizza, spaghetti carbonara, cheeseburger, tea, coffee]" });
        s2_required.push("food_or_drink_type".to_string());
    }
    if required.is_event {
        s2_props["event_type"] = json!({ "type": "string", "description": "birthday party, wedding, concert, holiday, etc." });
        s2_required.push("event_type".to_string());
    }
    if required.is_document {
        s2_props["document_type"] = json!({ "type": "string", "description": "Choose one of \
        [passport, receipt, ticket, book, magazine, notes, payment card, price card, id card, \
        menu, or recipe] or an otherwise fitting term. Answer with the document type only." });
        s2_required.push("document_type".to_string());
    }
    if required.is_activity {
        s2_props["activity_name"] = json!({ "type": "string", "description": "The main, singular physical action being performed, examples: [football, running, rowing, swimming]." });
        s2_required.push("activity_name".to_string());
    }
    if required.contains_text {
        s2_props["ocr_text"] = json!({ "type": "string", "description": "Extract all legible text exactly as it appears." });
        s2_required.push("ocr_text".to_string());
    }

    let s2: Option<Value> = if s2_required.is_empty() {
        None
    } else {
        let stage2_schema = json!({
            "type": "object",
            "properties": s2_props,
            "required": s2_required,
            "additionalProperties": false
        });
        let prompt = "Provide detailed analysis for the requested fields only. \
                  If 'ocr_text' is requested, ignore design and output only the readable text.";
        let s2_response = llm
            .chat(prompt)
            .images(&[file])
            .schema(stage2_schema)
            .call()
            .await?;
        serde_json::from_str(&s2_response).ok()
    };

    let data = if let Some(s2) = s2 {
        LlmCategorizationData {
            caption: required.caption,
            main_subject: required.main_subject,
            setting: required.setting,
            contains_pets: required.contains_pets,
            contains_vehicle: required.contains_vehicle,
            contains_landmarks: required.contains_landmarks,
            contains_people: required.contains_people,
            contains_animals: required.contains_animals,
            is_indoor: required.is_indoor,
            is_food_or_drink: required.is_food_or_drink,
            is_event: required.is_event,
            is_document: required.is_document,
            is_landscape: required.is_landscape,
            is_cityscape: required.is_cityscape,
            is_activity: required.is_activity,
            contains_text: required.contains_text,

            pet_type: s2["pet_type"].as_str().map(String::from),
            animal_type: s2["animal_type"].as_str().map(String::from),
            food_or_drink_type: s2["food_or_drink_type"].as_str().map(String::from),
            vehicle_type: s2["vehicle_type"].as_str().map(String::from),
            event_type: s2["event_type"].as_str().map(String::from),
            landmark_name: s2["landmark_name"].as_str().map(String::from),
            document_type: s2["document_type"].as_str().map(String::from),
            people_count: s2["people_count"].as_i64().map(|v| v as i32),
            people_mood: s2["people_mood"].as_str().map(String::from),
            photo_type: s2["photo_type"].as_str().map(String::from),
            activity_name: s2["activity_name"].as_str().map(String::from),
            ocr_text: s2["ocr_text"].as_str().map(String::from),
        }
    } else {
        LlmCategorizationData {
            caption: required.caption,
            main_subject: required.main_subject,
            contains_pets: required.contains_pets,
            contains_vehicle: required.contains_vehicle,
            contains_landmarks: required.contains_landmarks,
            contains_people: required.contains_people,
            contains_animals: required.contains_animals,
            is_indoor: required.is_indoor,
            is_food_or_drink: required.is_food_or_drink,
            is_event: required.is_event,
            is_document: required.is_document,
            is_landscape: required.is_landscape,
            is_cityscape: required.is_cityscape,
            is_activity: required.is_activity,
            setting: required.setting,
            contains_text: required.contains_text,

            pet_type: None,
            animal_type: None,
            food_or_drink_type: None,
            vehicle_type: None,
            event_type: None,
            landmark_name: None,
            document_type: None,
            people_count: None,
            people_mood: None,
            photo_type: None,
            activity_name: None,
            ocr_text: None,
        }
    };

    Ok(Some(data))
}
