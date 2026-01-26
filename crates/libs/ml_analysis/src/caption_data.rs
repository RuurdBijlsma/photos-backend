use color_eyre::eyre::Result;
use common_types::ml_analysis::LlmCaptionData;
use language_model::ChatSession;
use std::path::Path;

const CAPTION_PROMPT: &str = "Caption this image in one paragraph.";

async fn ask(session: &mut ChatSession, file: &Path, question: &str) -> Result<String> {
    let answer = session.chat(question).images(&[file]).call().await?;
    session.reset();
    Ok(answer)
}

#[allow(clippy::too_many_lines)]
pub async fn get_caption_data(session: &mut ChatSession, file: &Path) -> Result<LlmCaptionData> {
    let default_caption = ask(session, file, CAPTION_PROMPT).await?;

    let main_subject = ask(
        session,
        file,
        "What is the single main subject of this photo?",
    ).await?;
    let in_or_outdoors_raw = ask(
        session,
        file,
        "Is this photo taken indoors or outdoors?",
    ).await?;

    let is_outdoor = in_or_outdoors_raw.to_lowercase().contains("outdoor");
    let is_indoor = in_or_outdoors_raw.to_lowercase().contains("indoor");

    let mut is_landscape = false;
    let mut is_cityscape = false;
    if is_outdoor {
        let is_landscape_raw = ask(
            session,
            file,
            "Is this a landscape featuring natural scenery such as mountains, dunes, forests, lakes, etc.? yes or no.",
        ).await?;
        if is_landscape_raw.to_lowercase().contains("yes") {
            is_landscape = true;
        }
        let is_cityscape_raw = ask(
            session,
            file,
            "Is this a cityscape showing urban buildings, streets, skylines, etc.? yes or no.",
        ).await?;
        if is_cityscape_raw.to_lowercase().contains("yes") {
            is_cityscape = true;
        }
    }

    let has_pet_raw = ask(
        session,
        file,
        "Does this photo contain one or more pets? yes or no.",
    ).await?;
    let contains_pets = has_pet_raw.to_lowercase().contains("yes");
    let pet_type = if contains_pets {
        Some(ask(
            session,
            file,
            "What kind of pet is shown in this photo?",
        ).await?)
    } else {
        None
    };

    let has_animals_raw = ask(
        session,
        file,
        "Does this photo contain one or more live animals? yes or no.",
    ).await?;
    let contains_animals = has_animals_raw.to_lowercase().contains("yes");
    let animal_type = if contains_animals {
        Some(ask(
            session,
            file,
            "What animal is shown in this photo?",
        ).await?)
    } else {
        None
    };

    let is_food_raw = ask(
        session,
        file,
        "Is this a photo of food or drink? yes or no.",
    ).await?;
    let is_food_or_drink = is_food_raw.to_lowercase().contains("yes");
    let food_or_drink_type = if is_food_or_drink {
        Some(ask(
            session,
            file,
            "What kind of food is this?",
        ).await?)
    } else {
        None
    };

    let has_vehicle_raw = ask(
        session,
        file,
        "Is there a vehicle shown prominently in this photo? yes or no.",
    ).await?;
    let contains_vehicle = has_vehicle_raw.to_lowercase().contains("yes");
    let vehicle_type = if contains_vehicle {
        Some(ask(
            session,
            file,
            "What type of vehicle is in this image (e.g., car, boat, bicycle).await?",
        ).await?)
    } else {
        None
    };

    let setting = ask(
        session,
        file,
        "What is the setting of this photo?",
    ).await?;

    let is_event_raw = ask(
        session,
        file,
        "Does this photo appear to be from a specific event (e.g., birthday party, wedding, concert, holiday).await? Answer yes or no.",
    ).await?;
    let is_event = is_event_raw.to_lowercase().contains("yes");
    let event_type = if is_event {
        Some(ask(
            session,
            file,
            "What event is depicted in this photo?",
        ).await?)
    } else {
        None
    };

    let has_landmarks_raw = ask(
        session,
        file,
        "Are there any recognizable landmarks or famous places in this photo? Answer yes or no.",
    ).await?;
    let contains_landmarks = has_landmarks_raw.to_lowercase().contains("yes");
    let landmark_name = if contains_landmarks {
        Some(ask(
            session,
            file,
            "What landmark or famous place is shown in this photo?",
        ).await?)
    } else {
        None
    };

    let is_document_raw = ask(
        session,
        file,
        "Is this a photo of a document, like a passport, receipt, ticket, book, magazine, notes, payment card, id card, menu, or recipe? Answer yes or no.",
    ).await?;
    let is_document = is_document_raw.to_lowercase().contains("yes");
    let document_type = if is_document {
        Some(ask(
            session,
            file,
            "What kind of document is this?",
        ).await?)
    } else {
        None
    };

    let has_people_raw = ask(
        session,
        file,
        "Does this photo contain one or more people? Answer yes or no.",
    ).await?;
    let contains_people = has_people_raw.to_lowercase().contains("yes");

    let mut people_count = None;
    let mut people_mood = None;
    #[allow(clippy::useless_let_if_seq)]
    let mut photo_type = None;

    if contains_people {
        let people_count_raw = ask(
            session,
            file,
            "How many people are in this photo? Answer with a number.",
        ).await?;
        people_count = people_count_raw
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>()
            .parse::<i32>()
            .ok();

        people_mood = Some(ask(
            session,
            file,
            "What is the overall mood of the people in this photo? Are they happy, sad, serious, or neutral?",
        ).await?);
        photo_type = Some(ask(
            session,
            file,
            "What kind of photo is this, choose one of: (selfie, group photo, crowd, portrait, other).",
        ).await?);
    }

    let is_activity_raw = ask(
        session,
        file,
        "Is an activity being performed in this photo? Answer yes or no.",
    ).await?;
    let is_activity = is_activity_raw.to_lowercase().contains("yes");
    let activity_description = if is_activity {
        Some(ask(
            session,
            file,
            "What activity is being performed in this photo?",
        ).await?)
    } else {
        None
    };

    Ok(LlmCaptionData {
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
    })
}
