use async_trait::async_trait;
use sqlx::PgTransaction;
use app_state::AppSettings;
use common_services::api::album::service::get_representative_thumbnail;
use crate::handlers::daily_cards::DailyCardGenerator;

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
        _settings: &AppSettings,
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
                VALUES ($1, 'estimatr', 'Location Estimatr', $2, $3)
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