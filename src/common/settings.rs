use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Settings {
    pub media_folder: String,
    pub processing_api_url: String,
}

impl Settings {
    /// Loads and deserializes the `settings` section from the Loco app's configuration
    /// file into a strongly-typed `Settings` struct.
    ///
    /// The `settings` section is defined in the configuration file (e.g., `config/development.yaml`)
    /// and is accessible via `ctx.config.settings` as a `serde_json::Value`. This function
    /// converts that `serde_json::Value` into a `Settings` struct.
    ///
    /// # Arguments
    /// * `value` - A reference to a `serde_json::Value` representing the `settings` section
    ///             from the configuration file.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The `value` cannot be deserialized into the `Settings` struct.
    /// - The `settings` section in the configuration file does not match the expected structure.
    ///
    /// # Notes
    /// - This function is specifically designed to work with the `settings` section of the
    ///   Loco app's configuration file.
    /// - The `settings` section is optional in the configuration file. If it is missing,
    ///   `ctx.config.settings` will be `None`.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }
}
