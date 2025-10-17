#![allow(clippy::needless_for_each, clippy::cognitive_complexity)]

pub mod routes;
pub use routes::*;

use color_eyre::Result;
use common_photos::{get_db_pool, settings};
use http::HeaderValue;
use tower_http::cors;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    // --- Tracing & Error Handling Setup (unchanged) ---
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    color_eyre::install()?;

    // --- Server Startup ---
    info!("🚀 Initializing server...");
    let pool = get_db_pool().await?;

    // --- CORS Configuration ---
    let allowed_origins: Vec<HeaderValue> = settings()
        .api
        .allowed_origins
        .iter()
        .filter_map(|s| match s.parse() {
            Ok(hv) => Some(hv),
            Err(e) => {
                error!("Invalid CORS origin configured: {} - Error: {}", s, e);
                None
            }
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_origin(allowed_origins)
        .allow_headers(cors::Any);

    // --- Create Router & Start Server ---
    let app = create_router(pool).layer(cors);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3567").await?;

    info!("📚 Docs available at http://0.0.0.0:3567/docs");
    info!("✅ Server listening on http://0.0.0.0:3567");

    axum::serve(listener, app).await?;
    Ok(())
}
