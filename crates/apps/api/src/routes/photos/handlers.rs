use app_state::IngestSettings;
use axum::{Extension, Json};
use common_services::api::photos::interfaces::{
    ColorThemeParams, DownloadMediaParams, GetMediaItemParams, RandomPhotoResponse,
};
use common_services::api::photos::service::{download_media_file, random_photo};
use common_services::database::app_user::User;
use common_services::database::media_item::media_item::FullMediaItem;
use image::ImageDecoder;
use ml_analysis::get_color_theme;

use crate::api_state::ApiContext;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use color_eyre::eyre::eyre;
use common_services::api::photos::error::PhotosError;
use common_services::api::photos::interfaces::PhotoThumbnailParams;
use common_services::database::media_item_store::MediaItemStore;
use exif::{In, Tag, Value};
use fast_image_resize as fr;
use image::ImageReader;
use std::io::Cursor;
use std::time::Instant;
use tokio::{fs, task};
use tracing::instrument;

/// Get a full media item
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/item",
    tag = "Photos",
    params(
        GetMediaItemParams
    ),
    responses(
        (status = 200, description = "Get random photo and associated themes.", body = FullMediaItem),
        (status = 404, description = "Item not found."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn get_full_item_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaItemParams>,
) -> Result<Json<FullMediaItem>, PhotosError> {
    let item = MediaItemStore::find_by_id(&context.pool, &params.id).await?;
    if let Some(item) = item
        && item.user_id == user.id
    {
        Ok(Json(item))
    } else {
        Err(PhotosError::MediaNotFound(params.id))
    }
}

/// Get a random photo and its associated theme.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/random",
    tag = "Photos",
    responses(
        (status = 200, description = "Get random photo and associated themes.", body = RandomPhotoResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_random_photo(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Json<Option<RandomPhotoResponse>>, PhotosError> {
    let result = random_photo(&user, &context.pool).await?;
    Ok(Json(result))
}

/// Get a random photo and its associated theme.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/theme",
    tag = "Photos",
    params(
        ColorThemeParams
    ),
    responses(
        (status = 200, description = "Get theme object from a color.", body = Value),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_color_theme_handler(
    State(ingestion): State<IngestSettings>,
    Query(params): Query<ColorThemeParams>,
) -> Result<Json<serde_json::Value>, PhotosError> {
    let variant = &ingestion.analyzer.theme_generation.variant;
    let contrast_level = ingestion.analyzer.theme_generation.contrast_level;
    Ok(Json(get_color_theme(
        &params.color,
        variant,
        contrast_level as f32,
    )?))
}

/// Download a media file.
///
/// This endpoint streams a specific media file to the client. The path to the media
/// file must be a valid and secure path within the configured media directory.
///
/// # Errors
///
/// This function returns a `PhotosError` if the path is invalid, the file
/// isn't found, the user lacks permissions, or an internal server error occurs.
#[utoipa::path(
    get,
    path = "/photos/download",
    tag = "Photos",
    params(
        ("path" = String, Query, description = "The path of the media file to download")
    ),
    responses(
        (status = 200, description = "Media file streamed successfully.", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 400, description = "Invalid path provided, such as a directory traversal attempt."),
        (status = 403, description = "Permission denied when trying to access the file."),
        (status = 404, description = "The requested media file could not be found."),
        (status = 415, description = "The requested file is not a supported media type."),
        (status = 500, description = "An internal server error occurred."),
    )
)]
pub async fn download_full_file(
    State(ingestion): State<IngestSettings>,
    Extension(user): Extension<User>,
    Query(query): Query<DownloadMediaParams>,
) -> Result<impl IntoResponse, PhotosError> {
    let response = download_media_file(&ingestion, &user, &query.path).await?;
    Ok(response)
}

#[instrument(skip(context), err(Debug))]
pub async fn get_photo_thumbnail(
    State(context): State<ApiContext>,
    Query(query): Query<PhotoThumbnailParams>,
    Path(media_item_id): Path<String>,
) -> Result<impl IntoResponse, PhotosError> {
    let size = query.size.unwrap_or(360);
    if size > 1440 {
        return Err(PhotosError::AccessDenied);
    }

    // 1. Check Cache (Now looking for .jpg)
    let cache_dir = context.settings.ingest.thumbnail_root.join(".jpg-cache");
    let cache_path = cache_dir.join(format!("{media_item_id}_{size}.jpg"));

    if let Ok(cached_data) = fs::read(&cache_path).await {
        println!("Using cache");
        return Ok((
            [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            cached_data,
        ));
    }

    // 2. Database lookup for file path
    let Some(rel_path) =
        MediaItemStore::find_relative_path_by_id(&context.pool, &media_item_id).await?
    else {
        return Err(PhotosError::MediaNotFound(media_item_id));
    };
    let image_path = context.settings.ingest.media_root.join(&rel_path);

    // 3. Process Image
    let handle = task::spawn_blocking(move || -> Result<Vec<u8>, PhotosError> {
        let now = Instant::now();
        let file_bytes = std::fs::read(&image_path)
            .map_err(|e| PhotosError::Internal(eyre!(e).wrap_err("Failed to read image")))?;
        let time_read_bytes = now.elapsed();
        let start = Instant::now();

        let is_jpeg =
            rel_path.to_lowercase().ends_with(".jpg") || rel_path.to_lowercase().ends_with(".jpeg");

        // --- PATH A: EXIF Thumbnail (JPEG only) ---
        if is_jpeg {
            let mut cursor = Cursor::new(&file_bytes);
            if let Ok(exif_data) = exif::Reader::new().read_from_container(&mut cursor) {
                // Find Thumbnail Offset and Length in IFD1 (In(1))
                let offset = exif_data.get_field(Tag::JPEGInterchangeFormat, In(1));
                let length = exif_data.get_field(Tag::JPEGInterchangeFormatLength, In(1));

                if let (Some(off_f), Some(len_f)) = (offset, length) {
                    // Extract values from EXIF types (Short or Long)
                    let off_val = match off_f.value {
                        Value::Long(ref v) if !v.is_empty() => Some(v[0]),
                        Value::Short(ref v) if !v.is_empty() => Some(v[0] as u32),
                        _ => None,
                    };
                    let len_val = match len_f.value {
                        Value::Long(ref v) if !v.is_empty() => Some(v[0]),
                        Value::Short(ref v) if !v.is_empty() => Some(v[0] as u32),
                        _ => None,
                    };

                    if let (Some(o), Some(l)) = (off_val, len_val) {
                        let start = o as usize;
                        let end = start + l as usize;

                        if end <= exif_data.buf().len() {
                            let thumb_bytes = &exif_data.buf()[start..end];

                            // Check dimensions using imagesize crate (no full decoding)
                            if let Ok(dim) = imagesize::blob_size(thumb_bytes)
                                && dim.height >= ((size as f32 * 0.9) as usize)
                            {
                                println!(
                                    "Req size {size}. Using EXIF thumb ({}x{}) path. {:?}, read_bytes: {time_read_bytes:?}",
                                    dim.width,
                                    dim.height,
                                    now.elapsed()
                                );
                                return Ok(thumb_bytes.to_vec());
                            }
                        }
                    }
                }
            }
        }

        // --- PATH B: Resize Path ---
        let mut img = image::load_from_memory(&file_bytes)
            .map_err(|e| PhotosError::Internal(eyre!(e).wrap_err("Failed to decode image")))?;

        let orient_start = Instant::now();
        let mut decoder = ImageReader::open(image_path)
            .map_err(|e| eyre!("Failed to open image: {}", e))?
            .into_decoder()
            .map_err(|e| eyre!("Failed to read image: {}", e))?;
        let orientation = decoder
            .orientation()
            .map_err(|e| eyre!("Failed to get image orientation: {}", e))?;
        img.apply_orientation(orientation);
        println!("Fix orientation took: {:?}", orient_start.elapsed());

        let (width, height) = (img.width(), img.height());
        let aspect_ratio = width as f32 / height as f32;
        let (dst_width, dst_height) = if width > height {
            (size as u32, (size as f32 / aspect_ratio) as u32)
        } else {
            ((size as f32 * aspect_ratio) as u32, size as u32)
        };

        let src_image = fr::images::Image::from_vec_u8(
            width,
            height,
            img.to_rgb8().into_raw(),
            fr::PixelType::U8x3,
        )
        .map_err(|e| PhotosError::Internal(eyre!("Resize source error: {e}")))?;

        let mut dst_image = fr::images::Image::new(dst_width, dst_height, fr::PixelType::U8x3);
        let mut resizer = fr::Resizer::new();
        resizer
            .resize(&src_image, &mut dst_image, None)
            .map_err(|e| PhotosError::Internal(eyre!("Resizing failed: {e}")))?;

        // Encode to JPEG
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 80);

        encoder
            .encode(
                dst_image.buffer(),
                dst_width,
                dst_height,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| PhotosError::Internal(eyre!(e).wrap_err("Failed to encode JPEG")))?;

        println!(
            "Full resize: {:?}, read_bytes: {time_read_bytes:?}",
            start.elapsed()
        );
        Ok(buffer)
    });

    let jpeg_buffer = handle.await??;

    // 4. Save to cache
    let _ = fs::create_dir_all(&cache_dir).await;
    let _ = fs::write(&cache_path, &jpeg_buffer).await;

    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        jpeg_buffer,
    ))
}
