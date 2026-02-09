use color_eyre::eyre::Result;
use common_types::ml_analysis::LlmCategorizationData;
use language_model::LlamaClient;
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

// todo:
// Deze is te dom voor document type om een of andere reden, ik denk dat ocr_text er ook beter uit kan
// Verder werkt new-caption wel goed lijkt t.
// Test nog even de andere fields maar t lijkt wel geod

#[derive(Deserialize)]
pub struct RequiredStrings {
    pub caption: String,
    pub main_subject: String,
    pub setting: String,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
pub struct RequiredBools {
    pub contains_pets: bool,
    pub contains_vehicle: bool,
    pub famous_landmarks: bool,
    pub contains_people: bool,
    pub contains_animals: bool,
    pub contains_legible_text: bool,
    pub is_indoor: bool,
    pub depicts_food: bool,
    pub depicts_drink: bool,
    pub depicts_event: bool,
    pub contains_document: bool,
    pub depicts_landscape: bool,
    pub depicts_cityscape: bool,
    pub depicts_activity: bool,
}

#[derive(Deserialize)]
pub struct OptionalOutput {
    pub animal_type: Option<String>,
    pub food_name: Option<String>,
    pub drink_name: Option<String>,
    pub vehicle_type: Option<String>,
    pub event_type: Option<String>,
    pub landmark_name: Option<String>,
    pub activity_name: Option<String>,
    pub people_mood: Option<String>,
    pub photo_type: Option<String>,
    pub people_count: Option<i32>,
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
        },
        "required": ["setting", "main_subject", "caption"]
    });

    let s1_response = llm
        .chat(
            r"
Analyze this image and fill the schema fields.
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
    let required: RequiredStrings = serde_json::from_str(&s1_response)?;

    let categories_schema = json!({
        "type": "object",
        "properties": {
            "contains_pets": { "type": "boolean", "description": "Is there a prominent pet shown \
            in the photo? examples: [cat, dog, bird, etc.]" },

            "contains_animals": { "type": "boolean", "description": "Is an animal prominently \
            shown in the photo? Examples: [lion, dog, deer, ant, spider]" },

            "contains_vehicle": { "type": "boolean", "description": "Is a vehicle shown \
            prominently in the photo? Examples: [car, boat, bicycle, scooter]" },

            "famous_landmarks": { "type": "boolean", "description": "Does this photo clearly \
            show a *well-known, globally recognized landmark*? Examples: Eiffel Tower, Statue of \
            Liberty, Colosseum, Golden Gate Bridge, Parthenon. Do NOT mark typical buildings, \
            generic churches, or streetscapes as landmarks. Do NOT mark generic recognizable \
            natural features as landmarks. Mark as true only if a viewer could confidently name \
            the landmark without guessing." },

            "contains_people": { "type": "boolean", "description": "Does this photo contain at \
            least one person? Making sure the person is recognizable enough in the photo for you \
            to count the person(s) and detect their mood"},

            "is_indoor": { "type": "boolean" },

            "depicts_food": { "type": "boolean", "description": "Is this a photo food? Example: \
            [spaghetti, pizza, cheeseburger, lasagna]" },

            "depicts_drink": { "type": "boolean", "description": "Is this a photo a drink? Example: \
            [beer, coffee, cola, tea]" },

            "depicts_event": { "type": "boolean", "description": "Is this a specific event (e.g., \
            birthday party, wedding, concert, holiday)?" },

            "contains_document": { "type": "boolean", "description": "Is this a photo of one of the \
            following: passport, id_card, driver_license, certificate, receipt, invoice, bill, \
            bank_statement, tax_document, payslip, payment_card, ticket, boarding_pass, \
            reservation, itinerary, contract, license, warranty, manual, notes, exam_paper, \
            assignment, diploma, presentation_slide, book, magazine, newspaper, article, menu, \
            recipe, price_card, product_label, medical, letter, brochure, screenshot, diagram, \
            handwritten, sketch, generic_document." },

            "depicts_activity": { "type": "boolean", "description": "Is a clear, intentional physical \
            action being performed in this photo (e.g., walking, cooking, exercising), not \
            including passive states such as sitting, standing, resting, posing, or watching?" },

            "contains_legible_text": { "type": "boolean", "description": "Is there legible text visible in \
            this photo?" },

            "depicts_landscape": { "type": "boolean", "description": "Is this a landscape featuring \
            natural scenery such as mountains, dunes, forests, lakes, etc.?" },

            "depicts_cityscape": { "type": "boolean" },
        },
        "required": [
            "contains_pets", "contains_animals", "contains_vehicle", "famous_landmarks",
            "contains_people", "is_indoor", "depicts_food", "depicts_drink", "depicts_event",
            "contains_document", "depicts_activity", "contains_legible_text", "depicts_landscape",
            "depicts_cityscape"
        ]
    });

    let categories_response = llm
        .chat(
            r"You are an image categorization bot, analyze this image and identify
        which categories it belongs to.",
        )
        .images(&[file])
        .schema(categories_schema)
        .call()
        .await?;
    let categories: RequiredBools = serde_json::from_str(&categories_response)?;

    let mut s2_props = json!({});
    let mut s2_required = vec![];
    if categories.contains_animals || categories.contains_pets {
        s2_props["animal_type"] = json!({ "type": "string", "description": "Best guess as to what \
        animal is shown, give a singular term broadly naming the animal. Examples: \
        [lion, giraffe, goose, fox, dog, ant]. Answer concisely in at most two words. \
        Do not explain your choice." });
        s2_required.push("animal_type".to_string());
    }
    if categories.contains_vehicle {
        s2_props["vehicle_type"] = json!({ "type": "string", "enum": [
            "car", "motorcycle", "bicycle", "scooter", "moped", "truck", "van", "bus", "tram",
            "train", "subway", "trolleybus", "rickshaw", "electric_scooter", "segway", "boat",
            "ferry", "yacht", "sailboat", "canoe", "kayak", "jet_ski", "rowboat", "speedboat",
            "ship", "airplane", "helicopter", "glider", "hot_air_balloon", "drone", "tramcar",
            "cable_car", "funicular", "monorail", "submarine", "horse_cart", "animal_cart",
            "camper", "other_vehicle"
        ]});
        s2_required.push("vehicle_type".to_string());
    }
    if categories.famous_landmarks {
        s2_props["landmark_name"] =
            json!({ "type": "string", "description": "Name of the famous place or landmark." });
        s2_required.push("landmark_name".to_string());
    }
    if categories.contains_people {
        s2_props["people_count"] = json!({ "type": "integer" });
        s2_props["people_mood"] = json!({
            "type": "string",
            "enum": ["happy", "content", "relaxed", "calm", "excited", "playful", "energetic",
                "focused", "thoughtful", "neutral", "serious", "tired", "bored", "sad",
                "melancholic", "anxious", "stressed", "frustrated", "angry", "confident"]
        });
        s2_props["photo_type"] = json!({
            "type": "string",
            "enum": ["selfie", "group photo", "crowd", "portrait", "action", "candid", "people_not_main_subject", "other"]
        });
        s2_required.extend(vec![
            "people_count".into(),
            "people_mood".into(),
            "photo_type".into(),
        ]);
    }
    if categories.depicts_food {
        s2_props["food_name"] = json!({ "type": "string", "description": "What food is shown in this photo? Answer concisely in at most two words. Do not explain your choice." });
        s2_required.push("food_name".to_string());
    }
    if categories.depicts_drink {
        s2_props["drink_name"] = json!({ "type": "string", "description": "The main type \
        of drink shown, examples: [coffee, tea, beer, wine, cola, water]" });
        s2_required.push("drink_name".to_string());
    }
    if categories.depicts_event {
        s2_props["event_type"] = json!({ "type": "string", "enum": [
          "holiday", "wedding", "birthday_party", "new_years_eve", "christmas", "easter",
            "anniversary", "graduation", "party", "baby_shower", "funeral", "concert",
            "festival", "sports_event", "parade", "theater_performance", "exhibition",
            "conference", "workshop", "trade_show", "family_gathering", "reunion", "picnic",
            "barbecue", "camping_trip", "protest", "political_rally", "charity_event", "other_event"
        ]});
        s2_required.push("event_type".to_string());
    }
    if categories.depicts_activity {
        s2_props["activity_name"] = json!({ "type": "string", "description": "The main, singular \
        physical action being performed, examples: [football, running, rowing, swimming]." });
        s2_required.push("activity_name".to_string());
    }

    let optional_output = if s2_required.is_empty() {
        None
    } else {
        let stage2_schema = json!({
            "type": "object",
            "properties": s2_props,
            "required": s2_required,
            "additionalProperties": false
        });
        let prompt = "You are an image classification bot. Analyze the provided photo to \
        fill the requested schema fields. Keep your answers as concise as possible, don't explain \
        your choices.";
        let s2_response = llm
            .chat(prompt)
            .images(&[file])
            .schema(stage2_schema)
            .call()
            .await?;
        println!("optional_fields:\n{}", &s2_response);
        Some(serde_json::from_str::<OptionalOutput>(&s2_response)?)
    };

    let ocr_text = if categories.contains_legible_text {
        Some(
            llm.chat(
                "You are an OCR bot, transcribe the legible text in this image exactly as \
                is. Only OCR the pieces of text that are fully in view and clearly readable.",
            )
            .images(&[file])
            .call()
            .await?,
        )
    } else {
        None
    };

    let document_type = if categories.contains_document {
        #[derive(Deserialize)]
        struct DocTypeOutput {
            pub document_type: String,
        }
        let doc_type_schema = json!({
            "type": "object",
            "properties": {
                "document_type": {
                    "type": "string",
                    "description": "This photo has been identified as a document, classify it \
                    into a specific subtype.",
                    "enum": [
                        "passport", "id_card", "driver_license", "certificate", "receipt", "invoice", "bill",
                        "bank_statement", "tax_document", "payslip", "payment_card", "ticket", "boarding_pass",
                        "reservation", "itinerary", "contract", "license", "warranty", "manual", "notes",
                        "exam_paper", "assignment", "diploma", "presentation_slide", "book", "magazine",
                        "newspaper", "article", "menu", "recipe", "price_card", "product_label", "medical",
                        "letter", "brochure", "screenshot", "diagram", "handwritten", "sketch",
                        "generic_document"
                    ]
                }
            },
            "required": [
                "document_type"
            ],
            "additionalProperties": false
        });
        let prompt = format!(
            r"Analyze the provided photo in detail to fill the requested schema fields.

        Image caption:
        ```
        {}
        ```
        ",
            &required.caption
        );
        let doc_out_string = llm
            .chat(&prompt)
            .images(&[file])
            .schema(doc_type_schema)
            .call()
            .await?;
        println!("document_type{}", &doc_out_string);
        Some(serde_json::from_str::<DocTypeOutput>(&doc_out_string)?)
    } else {
        None
    };

    let data = LlmCategorizationData {
        caption: required.caption,
        main_subject: required.main_subject.to_lowercase(),
        setting: required.setting.to_lowercase(),
        contains_pets: categories.contains_pets,
        contains_vehicle: categories.contains_vehicle,
        contains_landmarks: categories.famous_landmarks,
        contains_animals: categories.contains_animals,
        is_indoor: categories.is_indoor,
        is_food: categories.depicts_food,
        is_drink: categories.depicts_drink,
        is_event: categories.depicts_event,
        is_document: categories.contains_document,
        is_landscape: categories.depicts_landscape,
        is_cityscape: categories.depicts_cityscape,
        is_activity: categories.depicts_activity,
        contains_text: categories.contains_legible_text,

        ocr_text,
        document_type: document_type.map(|d| d.document_type),
        animal_type: optional_output.as_ref().and_then(|o| o.animal_type.clone()),
        food_name: optional_output.as_ref().and_then(|o| o.food_name.clone()),
        drink_name: optional_output.as_ref().and_then(|o| o.drink_name.clone()),
        vehicle_type: optional_output
            .as_ref()
            .and_then(|o| o.vehicle_type.clone()),
        event_type: optional_output.as_ref().and_then(|o| o.event_type.clone()),
        landmark_name: optional_output
            .as_ref()
            .and_then(|o| o.landmark_name.clone()),
        activity_name: optional_output
            .as_ref()
            .and_then(|o| o.activity_name.clone()),
        people_mood: optional_output.as_ref().and_then(|o| o.people_mood.clone()),
        photo_type: optional_output.as_ref().and_then(|o| o.photo_type.clone()),
        people_count: optional_output.as_ref().and_then(|o| o.people_count),
        contains_people: if let Some(ref opt) = optional_output {
            categories.contains_people && opt.people_count.unwrap_or(0) > 0
        } else {
            categories.contains_people
        },
    };

    Ok(Some(data))
}
