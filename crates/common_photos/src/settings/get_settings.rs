use crate::settings::structs::AppSettings;
use std::fs::canonicalize;
use std::path::{absolute, Path, PathBuf};
use std::sync::LazyLock;

/// Load the app settings from YAML + environment variables
pub fn load_app_settings() -> color_eyre::Result<AppSettings> {
    let config_path = Path::new("config/settings.yaml").canonicalize()?;

    let builder = config::Config::builder()
        .add_source(config::File::from(config_path))
        .add_source(
            config::Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );
    Ok(builder.build()?.try_deserialize::<AppSettings>()?)
}

pub static SETTINGS: LazyLock<AppSettings> =
    LazyLock::new(|| load_app_settings().expect("Failed to load app settings"));

pub static MEDIA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| absolute(&SETTINGS.directories.media_folder).expect("Invalid media dir"));

pub static CANON_MEDIA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| canonicalize(&*MEDIA_DIR).expect("Cannot canonicalize media dir"));

pub static THUMBNAILS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    absolute(&SETTINGS.directories.thumbnails_folder).expect("Invalid thumbnails dir")
});

#[must_use]
pub fn settings() -> &'static AppSettings {
    &SETTINGS
}

#[must_use]
pub fn media_dir() -> &'static Path {
    &MEDIA_DIR
}

#[must_use]
pub fn canon_media_dir() -> &'static Path {
    &CANON_MEDIA_DIR
}

#[must_use]
pub fn thumbnails_dir() -> &'static Path {
    &THUMBNAILS_DIR
}
