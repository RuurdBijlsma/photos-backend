#![allow(
    clippy::needless_for_each,
    clippy::cognitive_complexity,
    clippy::cast_sign_loss
)]

mod pb;
pub mod routes;

use axum::routing::get_service;
pub use routes::*;

use color_eyre::Result;
use common_photos::{get_db_pool, settings};
use http::{header, HeaderValue};
use tower_http::compression::CompressionLayer;
use tower_http::cors;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    // --- Tracing & Error Handling Setup (unchanged) ---
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    color_eyre::install()?;

    // --- Server Startup ---
    info!("🚀 Initializing server...");
    let pool = get_db_pool().await?;
    let api_settings = &settings().api;

    // --- CORS Configuration ---
    let allowed_origins: Vec<HeaderValue> = api_settings
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
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            header::USER_AGENT,
            header::CACHE_CONTROL,
            header::PRAGMA,
        ]);

    // Static file serving
    let serve_dir = ServeDir::new("thumbnails");

    // Create a middleware layer to add the Cache-Control header.
    let cache_layer = SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    // --- Create Router & Start Server ---
    let app = create_router(pool)
        .layer(cors)
        .layer(CompressionLayer::new())
        .nest_service("/thumbnails", get_service(serve_dir).layer(cache_layer));
    let listen_address = format!("{}:{}", api_settings.host, api_settings.port);
    let listener = tokio::net::TcpListener::bind(&listen_address).await?;

    info!("📚 Docs available at http://{listen_address}/docs");
    info!("✅ Server listening on http://{listen_address}");

    axum::serve(listener, app).await?;
    Ok(())
}
