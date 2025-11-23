use crate::api::photos::error::PhotosError;

use crate::api::photos::interfaces::RandomPhotoResponse;
use crate::database::app_user::{User, UserRole};
use app_state::{IngestSettings, MakeRelativePath};
use axum::body::Body;
use color_eyre::Report;
use http::{Response, StatusCode, header};
use rand::Rng;
use sqlx::PgPool;
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{debug, warn};

/// Fetches a random photo with its color theme data for a specific user.
///
/// # Errors
///
/// Returns an error if either of the database queries fail.
pub async fn random_photo(
    user: &User,
    pool: &PgPool,
) -> Result<Option<RandomPhotoResponse>, PhotosError> {
    // Count the total number of photos with associated color data for the given user.
    let count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(cd.visual_analysis_id)
        FROM color_data AS cd
        JOIN visual_analysis AS va ON cd.visual_analysis_id = va.id
        JOIN media_item AS mi ON va.media_item_id = mi.id
        WHERE mi.user_id = $1 AND mi.deleted = false
        "#,
        user.id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0); // Default to 0 if count is NULL

    if count == 0 {
        warn!("No photos with color data for user {}", user.id);
        return Ok(None);
    }

    // Use a thread-safe random number generator to select a random offset.
    let random_offset = rand::rng().random_range(0..count);

    // Fetch a single row from `color_data` using the random offset,
    // along with the associated `media_item_id`.
    let random_data = sqlx::query_as!(
        RandomPhotoResponse,
        r#"
        SELECT
            cd.themes,
            mi.id as media_id
        FROM color_data AS cd
        JOIN visual_analysis AS va ON cd.visual_analysis_id = va.id
        JOIN media_item AS mi ON va.media_item_id = mi.id
        WHERE mi.user_id = $1 AND mi.deleted = false
        ORDER BY mi.id -- Consistent ordering is important for OFFSET
        LIMIT 1
        OFFSET $2
        "#,
        user.id,
        random_offset
    )
    .fetch_optional(pool)
    .await?;

    if random_data.is_none() {
        // This can happen in a race condition if photos are deleted between the COUNT and this query.
        warn!(
            "No photo found at offset {} for user {}",
            random_offset, user.id
        );
    }

    Ok(random_data)
}

/// Securely streams a validated media file to the client after performing authorization checks.
///
/// # Errors
///
/// Returns a `PhotosError` if path validation fails, the user is not authorized,
/// the file is not found, the media type is unsupported, or if any file system
/// or response building error occurs.
pub async fn download_media_file(
    ingestion: &IngestSettings,
    user: &User,
    path: &str,
) -> Result<Response<Body>, PhotosError> {
    // --- 1. Security & Path Validation ---
    let media_dir_canon = ingestion
        .media_root
        .canonicalize()
        .map_err(|e| Report::new(e).wrap_err("Failed to canonicalize media directory"))?;

    let file_path = ingestion.media_root.join(path);
    let file_canon = match file_path.canonicalize() {
        Ok(path) => path,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("File not found at path: {}", file_path.display());
            return Err(PhotosError::MediaNotFound(path.to_owned()));
        }
        Err(e) => {
            return Err(Report::new(e)
                .wrap_err("Failed to canonicalize path")
                .into());
        }
    };

    if !file_canon.starts_with(&media_dir_canon) {
        warn!("Blocked directory traversal attempt for: {}", path);
        return Err(PhotosError::InvalidPath);
    }

    // --- 2. Authorization ---
    let relative_path = file_canon.make_relative_canon(&ingestion.media_root_canon)?;
    if user.role != UserRole::Admin {
        let Some(user_media_folder) = &user.media_folder else {
            warn!(
                "Access denied for user {}: No media folder assigned.",
                user.id
            );
            return Err(PhotosError::AccessDenied);
        };
        if !relative_path.starts_with(user_media_folder) {
            warn!(
                "Access denied for user {}: Attempted to access path outside their media folder.",
                user.id
            );
            return Err(PhotosError::AccessDenied);
        }
    }

    // --- 3. Media Type Validation ---
    if !ingestion.is_media_file(&file_canon) {
        warn!("Unsupported media type requested: {}", file_canon.display());
        return Err(PhotosError::UnsupportedMediaType);
    }

    // --- 4. File Access ---
    let file = match File::open(&file_canon).await {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Err(PhotosError::MediaNotFound(path.to_owned())),
            std::io::ErrorKind::PermissionDenied => Err(PhotosError::AccessDenied),
            _ => Err(Report::new(e).wrap_err("Failed to open media file").into()),
        }?,
    };

    // --- 5. Build the Streaming Response ---
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::from_stream(stream);
    let mime_type = mime_guess::from_path(&file_canon).first_or_octet_stream();
    let filename = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mediafile");
    let disposition = format!("inline; filename=\"{filename}\"");
    let disposition_header = header::HeaderValue::from_str(&disposition)
        .unwrap_or(header::HeaderValue::from_static("inline"));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type.as_ref())
        .header(header::CONTENT_DISPOSITION, disposition_header)
        .body(body)
        .map_err(|e| Report::new(e).wrap_err("Failed to build response"))?)
}
