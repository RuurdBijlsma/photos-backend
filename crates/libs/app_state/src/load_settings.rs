use crate::{AppConstants, AppSettings, RawSettings};
use color_eyre::eyre::Result;
use config::{Config, File};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

pub fn load_settings_from_path(path: &Path, env_path: Option<&Path>) -> Result<AppSettings> {
    // Need to load from dotenv to get it to overwrite the secrets from env.
    if let Some(env_path) = env_path {
        dotenv::from_path(env_path).ok();
    }

    let builder = {
        let mut builder = Config::builder().add_source(File::from(path));
        if env_path.is_some() {
            builder = builder.add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            );
        }
        builder
    };

    let raw_settings = builder.build()?.try_deserialize::<RawSettings>()?;
    let settings: AppSettings = raw_settings.into();

    fs::create_dir_all(&settings.ingest.thumbnail_root).expect("Cannot create thumbnails folder");

    Ok(settings)
}

pub fn load_constants_from_path(path: &Path) -> Result<AppConstants> {
    let builder = Config::builder().add_source(File::from(path));
    let raw_constants = builder.build()?.try_deserialize::<RawSettings>()?;
    let app_constants: AppConstants = raw_constants.into();

    Ok(app_constants)
}

pub fn load_app_settings() -> Result<AppSettings> {
    let config_path = Path::new("config/settings.yaml").canonicalize()?;
    load_settings_from_path(&config_path, Some(Path::new(".env")))
}

fn load_app_constants() -> Result<AppConstants> {
    let config_path = Path::new("config/settings.yaml").canonicalize()?;
    load_constants_from_path(&config_path)
}

pub static CONSTANTS: OnceLock<AppConstants> = OnceLock::new();

#[must_use]
pub fn constants() -> &'static AppConstants {
    CONSTANTS.get_or_init(|| load_app_constants().expect("Cannot load app settings."))
}
