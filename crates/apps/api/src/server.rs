#![allow(
    clippy::needless_for_each,
    clippy::cognitive_complexity,
    clippy::cast_sign_loss,
    clippy::struct_excessive_bools,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation
)]

use crate::api_state::ApiContext;
use crate::create_router;
use app_state::{load_app_settings, AppSettings};
use axum::routing::get_service;
use color_eyre::Result;
use common_services::database::get_db_pool;
use common_services::s2s_client::S2SClient;
use http::{HeaderValue, header};
use reqwest::Client;
use sqlx::PgPool;
use tower_http::compression::CompressionLayer;
use tower_http::cors;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::{error, info};

pub async fn serve(pool: PgPool,settings: AppSettings) -> Result<()> {
    // --- Server Startup ---
    info!("🚀 Initializing server...");
    let api_state = ApiContext {
        pool,
        s2s_client: S2SClient::new(Client::new()),
        settings: settings.clone(),
    };

    // --- CORS Configuration ---
    let allowed_origins: Vec<HeaderValue> = settings
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
    let app = create_router(api_state)
        .layer(cors)
        .layer(CompressionLayer::new())
        .nest_service("/thumbnails", get_service(serve_dir).layer(cache_layer));
    let listen_address = format!("{}:{}", settings.api.host, settings.api.port);
    let listener = tokio::net::TcpListener::bind(&listen_address).await?;

    info!("📚 Docs available at http://{listen_address}/docs");
    info!("✅ Server listening on http://{listen_address}");

    axum::serve(listener, app).await?;
    Ok(())
}
