use common_photos::settings;
use ruurd_photos_thumbnail_generation::ThumbOptions;

pub fn get_thumb_options() -> ThumbOptions {
    let thumb_gen_config = &settings().thumbnail_generation;
    ThumbOptions {
        video_options: thumb_gen_config.video_options.clone(),
        avif_options: thumb_gen_config.avif_options.clone(),
        heights: thumb_gen_config.heights.clone(),
        thumbnail_extension: thumb_gen_config.thumbnail_extension.clone(),
        photo_extensions: thumb_gen_config.photo_extensions.clone(),
        video_extensions: thumb_gen_config.video_extensions.clone(),
        skip_if_exists: true,
    }
}
