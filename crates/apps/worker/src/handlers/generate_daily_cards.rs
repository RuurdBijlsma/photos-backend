use crate::context::WorkerContext;
use crate::handlers::JobResult;
use async_trait::async_trait;
use color_eyre::Result;
use common_services::api::album::service::get_representative_thumbnail;
use common_services::database::jobs::Job;
use app_state::AppSettings;
use sqlx::PgTransaction;
use chrono::{Utc, Datelike};
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use tracing::{info, warn};

#[async_trait]
trait DailyCardGenerator {
    fn card_type(&self) -> &'static str;
    async fn generate(
        &self,
        tx: &mut PgTransaction<'_>,
        user_id: i32,
        settings: &AppSettings,
    ) -> Result<()>;
}

struct ClusterCardGenerator;

#[async_trait]
impl DailyCardGenerator for ClusterCardGenerator {
    fn card_type(&self) -> &'static str {
        "cluster"
    }

    async fn generate(
        &self,
        tx: &mut PgTransaction<'_>,
        user_id: i32,
        _settings: &AppSettings,
    ) -> Result<()> {
        let latest_cluster_update: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar!(
            "SELECT MAX(updated_at) FROM photo_cluster WHERE user_id = $1",
            user_id
        )
        .fetch_one(&mut **tx)
        .await?;

        let latest_card_creation: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar!(
            "SELECT MAX(created_at) FROM daily_card WHERE user_id = $1 AND card_type = 'cluster'",
            user_id
        )
        .fetch_one(&mut **tx)
        .await?;

        let needs_regeneration = match (latest_cluster_update, latest_card_creation) {
            (Some(upd), Some(cre)) => upd > cre,
            (Some(_), None) => true,
            _ => false,
        };

        if !needs_regeneration {
            return Ok(());
        }

        // Delete all cluster daily_cards for this user
        sqlx::query!(
            "DELETE FROM daily_card WHERE user_id = $1 AND card_type = 'cluster'",
            user_id
        )
        .execute(&mut **tx)
        .await?;

        // Fetch all clusters for the user
        let clusters = sqlx::query!(
            "SELECT id, friendly_label FROM photo_cluster WHERE user_id = $1",
            user_id
        )
        .fetch_all(&mut **tx)
        .await?;

        for cluster in clusters {
            // Fetch media items in this cluster
            let items = sqlx::query!(
                r#"
                SELECT m.id, m.width, m.height, m.is_video, m.use_panorama_viewer as "is_panorama!"
                FROM media_item_photo_cluster pc
                JOIN media_item m ON pc.media_item_id = m.id
                WHERE pc.photo_cluster_id = $1 AND m.deleted = false
                "#,
                cluster.id
            )
            .fetch_all(&mut **tx)
            .await?;

            if items.is_empty() {
                continue;
            }

            let item_ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
            let thumbnail_id = get_representative_thumbnail(tx, &item_ids)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to get representative thumbnail: {:?}", e))?;

            let payload_items: Vec<serde_json::Value> = items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "id": i.id,
                        "width": i.width,
                        "height": i.height,
                        "is_video": i.is_video,
                        "is_panorama": i.is_panorama
                    })
                })
                .collect();

            let payload = serde_json::json!({
                "media_items": payload_items
            });

            sqlx::query!(
                r#"
                INSERT INTO daily_card (user_id, card_type, title, thumbnail_media_item_id, payload)
                VALUES ($1, 'cluster', $2, $3, $4)
                "#,
                user_id,
                cluster.friendly_label.unwrap_or_else(|| "Cluster".to_string()),
                thumbnail_id,
                payload
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}

struct OnThisDayCardGenerator;

#[async_trait]
impl DailyCardGenerator for OnThisDayCardGenerator {
    fn card_type(&self) -> &'static str {
        "on_this_day"
    }

    async fn generate(
        &self,
        tx: &mut PgTransaction<'_>,
        user_id: i32,
        _settings: &AppSettings,
    ) -> Result<()> {
        let today = Utc::now().naive_utc().date();

        for offset in 0..7 {
            let target_date = today + chrono::Duration::days(offset);

            // Check if card of type on_this_day already exists for this date
            let card_exists = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM daily_card WHERE user_id = $1 AND card_type = 'on_this_day' AND card_date = $2)",
                user_id,
                target_date
            )
            .fetch_one(&mut **tx)
            .await?
            .unwrap_or(false);

            if card_exists {
                continue;
            }

            let month = target_date.month() as i32;
            let day = target_date.day() as i32;
            let current_year = target_date.year() as i32;

            let years_records = sqlx::query!(
                r#"
                SELECT EXTRACT(YEAR FROM taken_at_local)::integer as "year!"
                FROM media_item
                WHERE user_id = $1 AND deleted = false
                  AND EXTRACT(MONTH FROM taken_at_local)::integer = $2
                  AND EXTRACT(DAY FROM taken_at_local)::integer = $3
                  AND EXTRACT(YEAR FROM taken_at_local)::integer < $4
                GROUP BY 1
                HAVING COUNT(*) >= 4
                "#,
                user_id,
                month,
                day,
                current_year
            )
            .fetch_all(&mut **tx)
            .await?;

            if years_records.is_empty() {
                continue;
            }

            let selected_year = {
                let mut rng = rand::rng();
                years_records.choose(&mut rng).map(|y| y.year)
            };

            let Some(year) = selected_year else {
                continue;
            };

            let items = sqlx::query!(
                r#"
                SELECT id, width, height, is_video, use_panorama_viewer as "is_panorama!"
                FROM media_item
                WHERE user_id = $1 AND deleted = false
                  AND EXTRACT(MONTH FROM taken_at_local)::integer = $2
                  AND EXTRACT(DAY FROM taken_at_local)::integer = $3
                  AND EXTRACT(YEAR FROM taken_at_local)::integer = $4
                "#,
                user_id,
                month,
                day,
                year
            )
            .fetch_all(&mut **tx)
            .await?;

            if items.is_empty() {
                continue;
            }

            let item_ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
            let thumbnail_id = get_representative_thumbnail(tx, &item_ids)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to get representative thumbnail: {:?}", e))?;

            let diff_years = current_year - year;
            let title = "On This Day";
            let subtitle = format!("{} years ago ({})", diff_years, year);

            let payload_items: Vec<serde_json::Value> = items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "id": i.id,
                        "width": i.width,
                        "height": i.height,
                        "is_video": i.is_video,
                        "is_panorama": i.is_panorama
                    })
                })
                .collect();

            let payload = serde_json::json!({
                "media_items": payload_items
            });

            sqlx::query!(
                r#"
                INSERT INTO daily_card (user_id, card_type, card_date, title, subtitle, thumbnail_media_item_id, payload)
                VALUES ($1, 'on_this_day', $2, $3, $4, $5, $6)
                "#,
                user_id,
                target_date,
                title,
                subtitle,
                thumbnail_id,
                payload
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}

struct GeoguessrCardGenerator;

#[async_trait]
impl DailyCardGenerator for GeoguessrCardGenerator {
    fn card_type(&self) -> &'static str {
        "geoguessr"
    }

    async fn generate(
        &self,
        tx: &mut PgTransaction<'_>,
        user_id: i32,
        _settings: &AppSettings,
    ) -> Result<()> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM daily_card WHERE user_id = $1 AND card_type = 'geoguessr' AND shown = false",
            user_id
        )
        .fetch_one(&mut **tx)
        .await?
        .unwrap_or(0);

        if count >= 7 {
            return Ok(());
        }

        let cards_to_generate = 7 - count;

        for _ in 0..cards_to_generate {
            // Select 5 random media items with valid GPS
            let items = sqlx::query!(
                r#"
                SELECT m.id, m.width, m.height, m.is_video, m.use_panorama_viewer as "is_panorama!",
                       g.latitude, g.longitude
                FROM media_item m
                JOIN gps g ON m.id = g.media_item_id
                WHERE m.user_id = $1 AND m.deleted = false
                  AND g.latitude != 0.0 AND g.longitude != 0.0
                ORDER BY random()
                LIMIT 5
                "#,
                user_id
            )
            .fetch_all(&mut **tx)
            .await?;

            if items.len() < 5 {
                break;
            }

            let item_ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
            let thumbnail_id = get_representative_thumbnail(tx, &item_ids)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to get representative thumbnail: {:?}", e))?;

            let rounds: Vec<serde_json::Value> = items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "media_item": {
                            "id": i.id,
                            "width": i.width,
                            "height": i.height,
                            "is_video": i.is_video,
                            "is_panorama": i.is_panorama
                        },
                        "latitude": i.latitude,
                        "longitude": i.longitude
                    })
                })
                .collect();

            let payload = serde_json::json!({
                "rounds": rounds,
                "current_round": 0,
                "score": 0
            });

            sqlx::query!(
                r#"
                INSERT INTO daily_card (user_id, card_type, title, thumbnail_media_item_id, payload)
                VALUES ($1, 'geoguessr', 'Location Estimatr', $2, $3)
                "#,
                user_id,
                thumbnail_id,
                payload
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}

pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    info!("Running daily cards generation job");

    let mut tx = context.pool.begin().await?;

    // Cleanup daily cards:
    // - card_date before today
    // - card_date is NULL & shown is true
    let today = Utc::now().naive_utc().date();
    sqlx::query!(
        "DELETE FROM daily_card WHERE card_date < $1 OR (card_date IS NULL AND shown = true)",
        today
    )
    .execute(&mut *tx)
    .await?;

    // Fetch all active users
    let users = sqlx::query!("SELECT id FROM app_user")
        .fetch_all(&mut *tx)
        .await?;

    let generators: Vec<Box<dyn DailyCardGenerator + Send + Sync>> = vec![
        Box::new(ClusterCardGenerator),
        Box::new(OnThisDayCardGenerator),
        Box::new(GeoguessrCardGenerator),
    ];

    for user in users {
        for generator in &generators {
            info!(
                "Running {} daily card generator for user {}",
                generator.card_type(),
                user.id
            );
            if let Err(e) = generator.generate(&mut tx, user.id, &context.settings).await {
                warn!(
                    "Failed to generate daily cards of type {} for user {}: {:?}",
                    generator.card_type(),
                    user.id,
                    e
                );
            }
        }
    }

    tx.commit().await?;
    info!("Daily cards generation job completed successfully");

    Ok(JobResult::Done)
}
