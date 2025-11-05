use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use color_eyre::eyre::Result;
use common_photos::{UserRole, get_db_pool, nice_id};
use media_analyzer::{
    AnalyzeResult, CaptureDetails, FileMetadata, PanoInfo, SourceDetails, TagData, TimeInfo,
};
use rand::Rng;
use sqlx::{PgPool, PgTransaction};
use std::time::Instant;
use tracing::info;
use worker::handlers::db::store_media::store_media_item;

/// The main entry point for seeding the database.
pub async fn seed_database_for_dev(pool: &PgPool, num_items: u32) -> Result<()> {
    info!(
        "🚀 Starting full database seed for development with {} photos...",
        num_items
    );
    let start_time = Instant::now();

    let mut tx = pool.begin().await.expect("Failed to start transaction");

    // Step 1: Create or update the mock user and get their ID.
    let user_id = create_or_update_mock_user(&mut tx).await?;
    info!("Ensured user 'Ruurd' exists with user_id: {}", user_id);

    // Step 2: Seed photos for that user within the same transaction.
    seed_mock_photos_in_tx(&mut tx, user_id, num_items).await?;

    tx.commit()
        .await
        .expect("Failed to commit final transaction");

    info!(
        "✅ Full database seed completed in {:?}",
        start_time.elapsed()
    );

    Ok(())
}

/// Creates a specific user from the provided details or updates them if they exist.
async fn create_or_update_mock_user(tx: &mut PgTransaction<'_>) -> Result<i32> {
    let email = "ruurd@bijlsma.dev";
    let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$YaxGnrPYSbvNCw3DzW7DdA$IawtPEn4ATtgulHRHZtIQ3fiOtgGSPeIwXlZ9+VFgp0";
    let name = "Ruurd";
    let role = UserRole::Admin;

    // This query inserts the user. If a user with that email already exists
    // (violating the UNIQUE constraint), the ON CONFLICT clause triggers,
    // updating the existing record instead. In both cases, it returns the user's ID.
    let user_id: i32 = sqlx::query_scalar(
        r"
        INSERT INTO app_user (email, password, name, role, media_folder)
        VALUES ($1, $2, $3, $4::user_role, '')
        ON CONFLICT (email) DO UPDATE
        SET
            name = EXCLUDED.name,
            password = EXCLUDED.password,
            role = EXCLUDED.role,
            updated_at = now()
        RETURNING id
        ",
    )
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(role)
    .fetch_one(&mut **tx)
    .await?;

    Ok(user_id)
}

/// Seeds the database with mock media items within an existing transaction.
async fn seed_mock_photos_in_tx(
    tx: &mut PgTransaction<'_>,
    user_id: i32,
    num_items: u32,
) -> Result<()> {
    info!(
        "Seeding {} mock photos for user_id {}...",
        num_items, user_id
    );
    let mut rng = rand::rng();
    let seconds_in_five_years = 5 * 365 * 24 * 60 * 60;

    for i in 0..num_items {
        let item_id = nice_id(10);
        let relative_path = format!("/mock/{}.jpg", &item_id);

        let random_seconds_ago = rng.random_range(0..seconds_in_five_years);
        let taken_at: DateTime<Utc> = Utc::now() - Duration::seconds(random_seconds_ago);
        let taken_at_local: NaiveDateTime = taken_at.naive_utc();

        let data = AnalyzeResult {
            hash: nice_id(32),
            exif: serde_json::Value::Null,
            metadata: FileMetadata {
                width: rng.random_range(800..=4000),
                height: rng.random_range(800..=4000),
                mime_type: "image/jpeg".to_string(),
                duration: None,
                size_bytes: rng.random_range(1_000_000..=8_000_000),
                orientation: None,
            },
            capture_details: CaptureDetails {
                iso: None,
                exposure_time: None,
                aperture: None,
                focal_length: None,
                camera_make: None,
                camera_model: None,
            },
            tags: TagData {
                is_motion_photo: false,
                motion_photo_presentation_timestamp: None,
                is_night_sight: false,
                is_hdr: false,
                is_burst: false,
                burst_id: None,
                is_timelapse: false,
                is_slowmotion: false,
                is_video: false,
                capture_fps: None,
                video_fps: None,
            },
            time_info: TimeInfo {
                datetime_utc: Some(taken_at),
                datetime_local: taken_at_local,
                timezone: None,
                source_details: SourceDetails {
                    time_source: "mock".to_string(),
                    confidence: "high".to_string(),
                },
            },
            pano_info: PanoInfo {
                use_panorama_viewer: false,
                is_photosphere: false,
                view_info: None,
                projection_type: None,
            },
            gps_info: None,
            weather_info: None,
        };

        store_media_item(&mut *tx, &relative_path, &data, &item_id, user_id)
            .await
            .expect("Failed to store media item");

        if (i + 1) % 1000 == 0 {
            info!("... inserted {}/{} photos", i + 1, num_items);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let pool = get_db_pool().await?;
    seed_database_for_dev(&pool, 100000).await?;

    Ok(())
}
