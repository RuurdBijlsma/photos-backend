use crate::handlers::daily_cards::DailyCardGenerator;
use app_state::AppSettings;
use async_trait::async_trait;
use common_services::api::album::service::get_representative_thumbnail;
use sqlx::PgTransaction;

pub struct LocationEstimatrCardGenerator;

#[async_trait]
impl DailyCardGenerator for LocationEstimatrCardGenerator {
    fn card_type(&self) -> &'static str {
        "estimatr"
    }

    async fn generate(
        &self,
        tx: &mut PgTransaction<'_>,
        user_id: i32,
        settings: &AppSettings,
    ) -> color_eyre::Result<()> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM daily_card WHERE user_id = $1 AND card_type = 'estimatr' AND shown = false",
            user_id
        )
            .fetch_one(&mut **tx)
            .await?
            .unwrap_or(0);

        if count >= 7 {
            return Ok(());
        }

        let cards_to_generate = 7 - count;

        // todo: add areaKm2 calculation
        // and different sources for location estmiatr fotos
        // Perhaps all in one country, or all in one year
        // Misschien kan t ook met cluster term? (wel even checken of t verschillende locaties zijn anders is t niet leuk)

        for _ in 0..cards_to_generate {
            // Select random media items with valid GPS
            let limit = settings.daily_cards.estimatr.rounds_per_day;
            let items = sqlx::query!(
                r#"
                SELECT m.id, m.width, m.height, m.is_video, m.use_panorama_viewer as "is_panorama!",
                       g.latitude, g.longitude, m.duration_ms, m.taken_at_local, m.has_thumbnails
                FROM media_item m
                JOIN gps g ON m.id = g.media_item_id
                WHERE m.user_id = $1 AND m.deleted = false
                  AND g.latitude != 0.0 AND g.longitude != 0.0
                ORDER BY random()
                LIMIT $2
                "#,
                user_id,
                limit
            )
            .fetch_all(&mut **tx)
            .await?;

            if items.is_empty() {
                break;
            }

            let mut item_ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
            let thumbnail_id = get_representative_thumbnail(tx, &item_ids)
                .await
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to get representative thumbnail: {:?}", e)
                })?;
            // Make sure thumbnail is first round
            if let Some(ref thumb_id) = thumbnail_id
                && let Some(pos) = item_ids.iter().position(|id| id == thumb_id) {
                    let id = item_ids.remove(pos);
                    item_ids.insert(0, id);
                }
            let rounds: Vec<serde_json::Value> = items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "mediaItem": {
                            "id": i.id,
                            "ratio": f64::from(i.width) / f64::from(i.height),
                            "durationMs": i.duration_ms,
                            "hasThumbnails": i.has_thumbnails,
                            "isVideo": i.is_video,
                            "width": i.width,
                            "height": i.height,
                            "isPanorama": i.is_panorama,
                            "takenAtLocal": i.taken_at_local,
                        },
                        "latitude": i.latitude,
                        "longitude": i.longitude,
                    })
                })
                .collect();

            let payload = serde_json::json!({
                "rounds": rounds,
                "areaKm2": 10_530_000, // todo actually calculate
            });

            sqlx::query!(
                r#"
                INSERT INTO daily_card (user_id, card_type, title, subtitle, thumbnail_media_item_id, payload)
                VALUES ($1, 'estimatr', 'Location Estimatr', 'Where was this taken?', $2, $3)
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
