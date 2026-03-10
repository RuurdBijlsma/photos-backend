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
use crate::timeline::websocket::create_media_item_transmitter;
use app_state::AppSettings;
use axum::routing::get_service;
use axum_server::tls_rustls::RustlsConfig;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use common_services::s2s_client::S2SClient;
use http::{HeaderValue, header};
use open_clip_inference::TextEmbedder;
use reqwest::Client;
use sqlx::PgPool;
use std::iter::once;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs;
use tower_http::compression::CompressionLayer;
use tower_http::cors;
use tower_http::cors::CorsLayer;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

pub async fn serve(pool: PgPool, settings: AppSettings) -> Result<()> {
    dbg!(1);
    let y = TextEmbedder::from_hf(&settings.ingest.analyzer.search.embedder_model_id)
        .build()
        .await?;
    dbg!(1);
    let x = Arc::new(y);
    dbg!(1);
    // --- Server Startup ---
    info!("🚀 Initializing server...");
    let api_state = ApiContext {
        pool: pool.clone(),
        s2s_client: S2SClient::new(Client::new()),
        settings: settings.clone(),
        timeline_broadcaster: create_media_item_transmitter(&pool)?,
        embedder: x,
    };
    dbg!(1);

    fs::create_dir_all(&settings.ingest.thumbnail_root.join(".jpg-cache")).await?;
    dbg!(1);

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
    dbg!(1);

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
    dbg!(1);

    // Static file serving
    let serve_dir = ServeDir::new("thumbnails");
    dbg!(1);

    // Create a middleware layer to add the Cache-Control header.
    let cache_layer = SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    dbg!(1);

    // --- Create Router ---
    let app = create_router(api_state)
        .layer(TraceLayer::new_for_http().on_request(()))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            header::AUTHORIZATION,
        )))
        .nest_service("/thumbnails", get_service(serve_dir).layer(cache_layer));
    dbg!(1);

    // Serve with https local cert
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    dbg!(1);
    let config = RustlsConfig::from_pem_file(
        "C:/Users/Ruurd/Desktop/localhost.pem",
        "C:/Users/Ruurd/Desktop/localhost-key.pem",
    )
    .await?;
    dbg!(1);

    let addr: SocketAddr = format!("{}:{}", settings.api.host, settings.api.port)
        .parse()
        .map_err(|e| eyre!("Invalid address: {}", e))?;
    dbg!(1);

    info!("🐸 Server listening on https://{}", addr);
    dbg!(1);

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    Ok(())
}
