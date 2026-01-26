use color_eyre::eyre::Result;
use common_types::ml_analysis::LlmCategorizationData;
use language_model::LlamaClient;
use std::path::Path;

const CAPTION_PROMPT: &str = "You are an image captioning assistant. Describe the main \
content of this image in a short, factual caption suitable for search. Include the main \
objects, people, animals, and the setting or scene, but do not add opinions or creative \
interpretations. Make it concise and clear, suitable for full-text search. Answer in one \
to two paragraphs.";

async fn ask(client: &LlamaClient, file: &Path, question: &str) -> Result<String> {
    let response = client.chat(question).images(&[file]).call().await?;
    println!("Q: '{question}': \nA: \t{response}\n");
    Ok(response)
}

#[allow(clippy::too_many_lines)]
pub async fn get_caption_data(llm: &LlamaClient, file: &Path) -> Result<LlmCategorizationData> {
    let default_caption = ask(llm, file, CAPTION_PROMPT).await?;

    let main_subject = ask(
        llm,
        file,
        "What is the single main subject of this photo? Direct answer only.",
    )
    .await?;
    let in_or_outdoors_raw = ask(
        llm,
        file,
        "Is this photo taken indoors or outdoors? Answer in one word.",
    )
    .await?;

    let is_outdoor = in_or_outdoors_raw.to_lowercase().contains("outdoor");
    let is_indoor = in_or_outdoors_raw.to_lowercase().contains("indoor");

    let mut is_landscape = false;
    let mut is_cityscape = false;
    if is_outdoor {
        let is_landscape_raw = ask(
            llm,
            file,
            "Is this a landscape featuring natural scenery such as mountains, dunes, forests, lakes, etc.? Answer yes or no only.",
        ).await?;
        if is_landscape_raw.to_lowercase().contains("yes") {
            is_landscape = true;
        }
        let is_cityscape_raw = ask(
            llm,
            file,
            "Is this a cityscape showing urban buildings, streets, skylines, etc.? Answer yes or no only.",
        )
            .await?;
        if is_cityscape_raw.to_lowercase().contains("yes") {
            is_cityscape = true;
        }
    }

    let has_pet_raw = ask(
        llm,
        file,
        "Does this photo contain one or more pets? Answer yes or no only.",
    )
    .await?;
    let contains_pets = has_pet_raw.to_lowercase().contains("yes");
    let pet_type = if contains_pets {
        Some(
            ask(
                llm,
                file,
                "What kind of pet is shown in this photo? Answer with the animal only.",
            )
            .await?,
        )
    } else {
        None
    };

    let has_animals_raw = ask(
        llm,
        file,
        "Does this photo contain one or more live animals? Answer yes or no only.",
    )
    .await?;
    let contains_animals = has_animals_raw.to_lowercase().contains("yes");
    let animal_type = if contains_animals {
        Some(
            ask(
                llm,
                file,
                "What animal is shown in this photo? Answer with the animal only.",
            )
            .await?,
        )
    } else {
        None
    };

    let is_food_raw = ask(
        llm,
        file,
        "Is this a photo of food or drink? Answer yes or no only.",
    )
    .await?;
    let is_food_or_drink = is_food_raw.to_lowercase().contains("yes");
    let food_or_drink_type = if is_food_or_drink {
        Some(
            ask(
                llm,
                file,
                "What kind of food is this? Answer with the food/drink name only.",
            )
            .await?,
        )
    } else {
        None
    };

    let has_vehicle_raw = ask(
        llm,
        file,
        "Is there a vehicle shown prominently in this photo? Answer yes or no only.",
    )
    .await?;
    let contains_vehicle = has_vehicle_raw.to_lowercase().contains("yes");
    let vehicle_type = if contains_vehicle {
        Some(
            ask(
                llm,
                file,
                "What type of vehicle is in this image (e.g., car, boat, bicycle)? Answer with only the vehicle.",
            )
            .await?,
        )
    } else {
        None
    };

    let setting = ask(
        llm,
        file,
        "Identify the main scene or environment of this photo. Focus only on the type of
        place or setting (e.g., kitchen, classroom, street, park, courtyard, office). Ignore
        people, animals, objects, and activities. Answer with a word or short phrase suitable for
        categorizing or grouping photos.",
    )
    .await?;

    let is_event_raw = ask(
        llm,
        file,
        "Does this photo appear to be from a specific event (e.g., birthday party, \
        wedding, concert, holiday)? Answer yes or no only.",
    )
    .await?;
    let is_event = is_event_raw.to_lowercase().contains("yes");
    let event_type = if is_event {
        Some(
            ask(
                llm,
                file,
                "What event is depicted in this photo? (e.g., \
        birthday party, wedding, concert, holiday, etc.)? Answer with the event only.",
            )
            .await?,
        )
    } else {
        None
    };

    let has_landmarks_raw = ask(
        llm,
        file,
        "Are there any recognizable landmarks or famous places in this photo? \
        Answer yes or no only.",
    )
    .await?;
    let contains_landmarks = has_landmarks_raw.to_lowercase().contains("yes");
    let landmark_name = if contains_landmarks {
        Some(
            ask(
                llm,
                file,
                "What landmark or famous place is shown in this photo? Answer with only \
                the landmark name. Do not add any notes or explanations, just respond with the landmark or place name.",
            )
            .await?,
        )
    } else {
        None
    };

    let is_document_raw = ask(
        llm,
        file,
        "Is this a photo of a document, like a passport, receipt, ticket, book, magazine, \
        notes, payment card, id card, menu, or recipe? Answer yes or no only.",
    )
    .await?;
    let is_document = is_document_raw.to_lowercase().contains("yes");
    let document_type = if is_document {
        Some(
            ask(
                llm,
                file,
                "What kind of document is this? Choose one of [passport, receipt, ticket, \
                book, magazine, notes, payment card, price card, id card, menu, or recipe] or an otherwise \
                fitting term. Answer with the document type only.",
            )
                .await?,
        )
    } else {
        None
    };

    let has_people_raw = ask(
        llm,
        file,
        "Does this photo contain one or more people? Answer yes or no only.",
    )
    .await?;
    let contains_people = has_people_raw.to_lowercase().contains("yes");

    let mut people_count = None;
    let mut people_mood = None;
    #[allow(clippy::useless_let_if_seq)]
    let mut photo_type = None;

    if contains_people {
        let people_count_raw = ask(
            llm,
            file,
            "How many people are in this photo? Answer only with a number.",
        )
        .await?;
        people_count = people_count_raw
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>()
            .parse::<i32>()
            .ok();

        people_mood = Some(
            ask(
                llm,
                file,
                r"
What is the overall mood of the people in this photo? Choose from the following
```
Happy
Content
Relaxed
Calm
Excited
Playful
Energetic
Focused
Thoughtful
Neutral
Serious
Tired
Bored
Sad
Melancholic
Anxious
Stressed
Frustrated
Angry
Confident
```
Answer with the overall mood only.
",
            )
            .await?,
        );
        photo_type = Some(ask(
            llm,
            file,
            "What kind of photo is this, choose one of: (selfie, group photo, crowd, portrait, action, candid, other).",
        ).await?);
    }

    let is_activity_raw = ask(
        llm,
        file,
        "Is a clear, intentional physical action being performed in this photo \
        (e.g., walking, cooking, exercising), not including passive states such as sitting, \
        standing, resting, posing, or watching? Answer yes or no only.",
    )
    .await?;
    let is_activity = is_activity_raw.to_lowercase().contains("yes");
    let activity_description = if is_activity {
        Some(
            ask(
                llm,
                file,
                "What activity is being performed in this photo? Answer with the activity only, no formatting.",
            )
                .await?,
        )
    } else {
        None
    };

    let has_legible_text_raw = ask(
        llm,
        file,
        "Is there any legible text in this photo? Answer yes or no only.",
    )
    .await?;
    let contains_text = has_legible_text_raw.to_lowercase().contains("yes");
    let ocr_text = if contains_text {
        Some(
            ask(
                llm,
                file,
                "You are an OCR assistant. Extract all text from the provided image. \
                The text could be in any format: a logo, sign, receipt, recipe, handwritten note, \
                printed document, etc. Ignore the visual design, formatting, or colors, and output \
                only the readable text as plain text. Do not add explanations or interpretations, \
                just return the text exactly as it appears in the image.",
            )
            .await?,
        )
    } else {
        None
    };

    Ok(LlmCategorizationData {
        default_caption,
        main_subject,
        contains_pets,
        contains_vehicle,
        contains_landmarks,
        contains_people,
        contains_animals,
        is_indoor,
        is_food_or_drink,
        is_event,
        is_document,
        is_landscape,
        is_cityscape,
        is_activity,
        setting,
        pet_type,
        animal_type,
        food_or_drink_type,
        vehicle_type,
        event_type,
        landmark_name,
        document_type,
        people_count,
        people_mood,
        photo_type,
        activity_description,
        contains_text,
        ocr_text,
    })
}
