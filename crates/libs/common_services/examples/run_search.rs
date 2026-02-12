#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use app_state::load_app_settings;
use common_services::api::search::service::{SearchMediaConfig, search_media};
use common_services::database::get_db_pool;
use common_services::database::user_store::UserStore;
use common_types::dev_constants::EMAIL;
use open_clip_inference::TextEmbedder;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let settings = load_app_settings()?;
    let pool = get_db_pool(&settings.secrets.database_url, true).await?;

    let user = UserStore::find_by_email(&pool, EMAIL)
        .await?
        .expect("no such user");
    let embedder = TextEmbedder::from_hf(&settings.ingest.analyzer.search.embedder_model_id)
        .build()
        .await?;

    let search_result = search_media(
        &user,
        &pool,
        &embedder,
        "kayak",
        SearchMediaConfig {
            text_weight: 0.3,
            semantic_weight: 1.0,
            threshold: Some(0.2),
            limit: Some(100),
        },
    )
    .await?;

    println!(
        "search result: {}",
        serde_json::to_string_pretty(&search_result)?
    );

    Ok(())
}
