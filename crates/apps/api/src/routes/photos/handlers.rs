use crate::api_state::ApiContext;
use app_state::IngestSettings;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use color_eyre::eyre::eyre;
use common_services::api::photos::error::PhotosError;
use common_services::api::photos::interfaces::{
    ColorThemeParams, DownloadMediaParams, GetMediaItemParams, PhotoThumbnailParams,
    RandomPhotoResponse,
};
use common_services::api::photos::service::{download_media_file, random_photo};
use common_services::database::app_user::User;
use common_services::database::media_item::media_item::FullMediaItem;
use common_services::database::media_item_store::MediaItemStore;
use fast_image_resize as fr;
use ml_analysis::get_color_theme;
use serde_json::Value;
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
) -> Result<Json<Value>, PhotosError> {
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

pub struct AbortOnDrop<T>(Option<tokio::task::JoinHandle<T>>);

impl<T> AbortOnDrop<T> {
    pub fn new(handle: tokio::task::JoinHandle<T>) -> Self {
        Self(Some(handle))
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}

#[instrument(skip(context), err(Debug))]
pub async fn get_photo_thumbnail(
    State(context): State<ApiContext>,
    Query(query): Query<PhotoThumbnailParams>,
    Path(media_item_id): Path<String>,
) -> Result<impl IntoResponse, PhotosError> {
    let size = query.size.unwrap_or(360);

    // 1. Check Cache (Fastest path, no semaphore needed)
    let cache_dir = context.settings.ingest.thumbnail_root.join("webp-cache");
    let cache_path = cache_dir.join(format!("{}_{}.webp", media_item_id, size));

    if let Ok(cached_data) = fs::read(&cache_path).await {
        return Ok((
            [
                (header::CONTENT_TYPE, "image/webp"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            cached_data,
        ));
    }

    // 2. LIFO Acquire (Permit is dropped at end of function)
    // This ensures if 50 requests are waiting, the newest one gets the next slot.
    let _permit = context.thumbnail_semaphore.acquire().await;

    // 3. Database lookup
    let Some(rel_path) =
        MediaItemStore::find_relative_path_by_id(&context.pool, &media_item_id).await?
    else {
        return Err(PhotosError::MediaNotFound(media_item_id));
    };
    let image_path = context.settings.ingest.media_root.join(&rel_path);

    let handle = task::spawn_blocking(move || -> Result<Vec<u8>, PhotosError> {
        let now = Instant::now();
        // 1. Load and decode image
        let img = image::ImageReader::open(image_path)
            .map_err(|e| PhotosError::Internal(eyre!(e).wrap_err("Failed to open image")))?
            .with_guessed_format()
            .map_err(|e| PhotosError::Internal(eyre!(e).wrap_err("Failed to guess image format")))?
            .decode() // This is still the slow part
            .map_err(|e| PhotosError::Internal(eyre!(e).wrap_err("Failed to decode image")))?;
        let dbg_decode_time = now.elapsed();
        let now = Instant::now();

        // 2. Calculate dimensions (Keeping aspect ratio)
        let (width, height) = (img.width(), img.height());
        let aspect_ratio = width as f32 / height as f32;
        let (dst_width, dst_height) = if width > height {
            (size as u32, (size as f32 / aspect_ratio) as u32)
        } else {
            ((size as f32 * aspect_ratio) as u32, size as u32)
        };

        // 3. Prepare Resize
        let src_image = fr::images::Image::from_vec_u8(
            width,
            height,
            img.to_rgb8().into_raw(),
            fr::PixelType::U8x3,
        )
        .map_err(|e| PhotosError::Internal(eyre!(format!("Resize source error: {e}"))))?;

        let mut dst_image = fr::images::Image::new(dst_width, dst_height, fr::PixelType::U8x3);
        let dbg_prepare_resize_time = now.elapsed();
        let now = Instant::now();

        // 4. Resize using the fastest high-quality filter (CatmullRom or Bilinear for speed)
        let mut resizer = fr::Resizer::new();
        resizer
            .resize(&src_image, &mut dst_image, None)
            .map_err(|e| PhotosError::Internal(eyre!(format!("Resizing failed: {e}"))))?;

        let dbg_resize_time = now.elapsed();
        let now = Instant::now();

        // 5. Encode to WebP using the `webp` crate (libwebp wrapper)
        let encoder = webp::Encoder::from_rgb(dst_image.buffer(), dst_width, dst_height);
        let memory = encoder.encode(80.0);

        let dbg_encode_time = now.elapsed();
        println!(
            "Decode: {dbg_decode_time:?}, PrepResize: {dbg_prepare_resize_time:?}, Resize: {dbg_resize_time:?}, Encode: {dbg_encode_time:?}"
        );

        Ok(memory.to_vec())
    });

    // 5. Wrap in Abort Guard
    let mut guard = AbortOnDrop::new(handle);

    // 6. Await the handle by taking it out of the guard
    let webp_buffer = guard.0.take().unwrap().await.map_err(|e| {
        if e.is_cancelled() {
            println!("Aborting due to cancelled thread");
            PhotosError::Cancelled
        } else {
            PhotosError::Internal(eyre!("Task panicked"))
        }
    })??;

    // 7. Save to cache
    let _ = fs::create_dir_all(&cache_dir).await;
    let _ = fs::write(&cache_path, &webp_buffer).await;

    Ok((
        [
            (header::CONTENT_TYPE, "image/webp"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        webp_buffer,
    ))
}
