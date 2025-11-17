use crate::{AppConstants, AppSettings, RawSettings};
use color_eyre::eyre::Result;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

pub fn load_app_settings() -> Result<AppSettings> {
    // Need to load from dotenv to get it to overwrite the db url from env.
    dotenv::from_path(".env").ok();
    let config_path = Path::new("config/settings.yaml").canonicalize()?;

    let builder = config::Config::builder()
        .add_source(config::File::from(config_path))
        .add_source(
            config::Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

    let raw_settings = builder.build()?.try_deserialize::<RawSettings>()?;
    let settings: AppSettings = raw_settings.into();

    fs::create_dir_all(&settings.ingest.thumbnail_root).expect("Cannot create thumbnails folder");

    Ok(settings)
}

fn load_app_constants() -> Result<AppConstants> {
    let config_path = Path::new("config/settings.yaml").canonicalize()?;
    let builder = config::Config::builder().add_source(config::File::from(config_path));
    let raw_constants = builder.build()?.try_deserialize::<RawSettings>()?;
    let app_constants: AppConstants = raw_constants.into();

    Ok(app_constants)
}

pub static CONSTANTS: LazyLock<AppConstants> =
    LazyLock::new(|| load_app_constants().expect("Cannot load app settings."));

#[must_use]
pub fn constants() -> &'static AppConstants {
    &CONSTANTS
}
