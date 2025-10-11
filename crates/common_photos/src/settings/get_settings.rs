use crate::settings::structs::AppSettings;
use std::fs::canonicalize;
use std::path::{absolute, Path, PathBuf};
use std::sync::LazyLock;

/// Load the app settings from YAML + environment variables
pub fn load_app_settings() -> color_eyre::Result<AppSettings> {
    let config_path =  Path::new("config/settings.yaml").canonicalize()?;

    let builder = config::Config::builder()
        .add_source(config::File::from(config_path))
        .add_source(
            config::Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );
    Ok(builder.build()?.try_deserialize::<AppSettings>()?)
}

/// Immutable global settings, initialized on first access.
pub static SETTINGS: LazyLock<AppSettings> = LazyLock::new(|| {
    load_app_settings().expect("Failed to load app settings")
});

/// Leaked global paths, derived from SETTINGS.
pub static MEDIA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    absolute(&SETTINGS.directories.media_folder)
        .expect("Invalid media dir")

});

pub static CANON_MEDIA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    canonicalize(&*MEDIA_DIR).expect("Cannot canonicalize media dir")
});

pub static THUMBNAILS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    absolute(&SETTINGS.directories.thumbnails_folder)
        .expect("Invalid thumbnails dir")

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

#[cfg(test)]
mod tests {
    // Note: The test functions don't need to change at all, because the
    // public API (the helper functions) remains the same.
    use crate::settings::get_settings::{media_dir, canon_media_dir, thumbnails_dir, load_app_settings};

    #[test]
    fn test_new_ok() -> color_eyre::Result<()> {
        // This first call will initialize all the LazyLock statics
        load_app_settings()?;

        // Subsequent calls access the cached data
        println!("MEDIA_DIR: {:?}", media_dir());
        println!("CANON_MEDIA_DIR: {:?}", canon_media_dir());
        println!("THUMBNAILS_DIR: {:?}", thumbnails_dir());

        Ok(())
    }
}