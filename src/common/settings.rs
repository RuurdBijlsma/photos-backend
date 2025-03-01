use loco_rs::app::AppContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Settings {
    pub media_dir: String,
    pub thumbnails_dir: String,
    pub processing_api_url: String,
}

impl Settings {
    pub fn from_context(ctx: &AppContext) -> Self {
        let settings_value = ctx
            .config
            .settings
            .clone()
            .expect("No settings found in config.");
        serde_json::from_value(settings_value).expect("Error deserializing settings.")
    }
}
