use crate::settings::structs::AppSettings;
use chrono_tz::Tz;
use std::fs;
use std::fs::canonicalize;
use std::path::{absolute, Path, PathBuf};
use std::sync::LazyLock;

pub fn load_app_settings() -> color_eyre::Result<AppSettings> {
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
    Ok(builder.build()?.try_deserialize::<AppSettings>()?)
}

pub static SETTINGS: LazyLock<AppSettings> =
    LazyLock::new(|| load_app_settings().expect("Failed to load app settings"));

pub static MEDIA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| absolute(&SETTINGS.directories.media_folder).expect("Invalid media dir"));

pub static CANON_MEDIA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| canonicalize(&*MEDIA_DIR).expect("Cannot canonicalize media dir"));

pub static THUMBNAILS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = absolute(&SETTINGS.directories.thumbnails_folder).expect("Invalid thumbnails dir");
    fs::create_dir_all(&dir).expect("Cannot create thumbnails folder");
    dir
});

pub static FALLBACK_TIMEZONE: LazyLock<Option<Tz>> = LazyLock::new(|| {
    let tz_string = &SETTINGS.analyzer.fallback_timezone;
    if tz_string.is_empty() {
        return None;
    }
    let parsed_tz = settings().analyzer.fallback_timezone.parse::<Tz>();
    parsed_tz.ok()
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

#[must_use]
pub fn fallback_timezone() -> &'static Option<Tz> {
    &FALLBACK_TIMEZONE
}
