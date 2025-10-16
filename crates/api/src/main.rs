#![allow(clippy::needless_for_each, clippy::cognitive_complexity)]

mod routes;

use crate::routes::root::route::__path_root;
use crate::routes::setup::interfaces;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme}, Modify,
    OpenApi,
};

use crate::routes::auth;
use crate::routes::auth::handlers::{
    check_admin, get_me, login, logout, refresh_session, register,
};
use crate::routes::auth::middleware::require_role;
use crate::routes::auth::UserRole;
use crate::routes::root::route::root;
use crate::routes::scalar_config::get_custom_html;
use crate::routes::setup;
use crate::routes::setup::handlers::{
    get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder,
    welcome_needed,
};
use auth::db_model::User;
use axum::http::{HeaderValue, Method};
use axum::{
    middleware, routing::{get, post},
    Router,
};
use common_photos::{get_db_pool, settings};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        // Auth handlers
        auth::handlers::login,
        auth::handlers::register,
        auth::handlers::refresh_session,
        auth::handlers::logout,
        auth::handlers::get_me,
        auth::handlers::check_admin,
        // Setup handlers
        setup::handlers::welcome_needed,
        setup::handlers::get_disk_response,
        setup::handlers::get_folder_media_sample,
        setup::handlers::get_folder_unsupported,
        setup::handlers::get_folders,
        setup::handlers::make_folder,
    ),
    components(
        schemas(
            // Auth schemas
            auth::db_model::User,
            auth::db_model::UserRole,
            auth::interfaces::CreateUser,
            auth::interfaces::LoginUser,
            auth::interfaces::RefreshTokenPayload,
            auth::interfaces::Tokens,
            // Setup schemas
            interfaces::FolderQuery,
            interfaces::MakeFolderBody,
            interfaces::PathInfoResponse,
            interfaces::MediaSampleResponse,
            interfaces::UnsupportedFilesResponse,
            interfaces::DiskResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Ruurd Photos", description = "Ruurd Photos' API")
    )
)]
struct ApiDoc;

/// A modifier to add bearer token security to the `OpenAPI` specification.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

/// The main entry point for the application.
///
/// # Errors
///
/// * Returns an error if `color_eyre` fails to install or if `start_server` fails.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    color_eyre::install()?;

    start_server().await?;
    Ok(())
}

/// Initializes and runs the Axum web server.
///
/// # Errors
///
/// * Returns an error if the database pool cannot be created or the server fails to bind or start.
async fn start_server() -> color_eyre::Result<()> {
    let pool = get_db_pool().await?;
    let allowed_origins_from_settings: Vec<HeaderValue> = settings()
        .api
        .allowed_origins
        .iter()
        .filter_map(|s| match s.parse::<HeaderValue>() {
            Ok(hv) => Some(hv),
            Err(e) => {
                error!("Invalid CORS origin configured: {} - Error: {}", s, e);
                None
            }
        })
        .collect();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_origin(allowed_origins_from_settings)
        .allow_headers(Any);

    let public_routes = Router::new()
        // ======== [ ROOT ] =========
        .route("/", get(root))
        // ======== [ /setup/ ] =========
        .route("/setup/welcome-needed", get(welcome_needed))
        // ======== [ /auth/ ] =========
        .route("/auth/refresh", post(refresh_session))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout));

    let protected_routes = Router::new()
        // ======== [ /auth/ ] =========
        .route("/auth/me", get(get_me))
        // ======== [ MIDDLEWARES ] =========
        .route_layer(middleware::from_extractor_with_state::<User, PgPool>(
            pool.clone(),
        ));

    let admin_routes = Router::new()
        // ======== [ /auth/ ] =========
        .route("/auth/admin-check", get(check_admin))
        // ======== [ /setup/ ] =========
        .route("/setup/disk-info", get(get_disk_response))
        .route("/setup/media-sample", get(get_folder_media_sample))
        .route("/setup/unsupported-files", get(get_folder_unsupported))
        .route("/setup/folders", get(get_folders))
        .route("/setup/make-folder", post(make_folder))
        // ======== [ MIDDLEWARES ] =========
        .route_layer(middleware::from_fn_with_state(
            UserRole::Admin,
            require_role,
        ))
        .route_layer(middleware::from_extractor_with_state::<User, PgPool>(
            pool.clone(),
        ));

    let openapi = ApiDoc::openapi();

    let app = Router::new()
        .merge(Scalar::with_url("/docs", openapi.clone()).custom_html(get_custom_html(&openapi)))
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(pool)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                // Do NOT use .on_request() or keep it at a very high level (e.g., Level::TRACE)
                // if you want to completely suppress start logs.
                // By default, TraceLayer::new_for_http() uses DefaultOnRequest at Level::DEBUG.
                // To remove it, you can explicitly set it to a no-op or a very high level:
                // .on_request(DefaultOnRequest::new().level(Level::TRACE)) // Almost never logs
                // Or you can configure on_response to log what you want.

                // This configures what happens when the response is sent.
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO) // Log the finished request at INFO level
                        .latency_unit(LatencyUnit::Millis) // Show latency in milliseconds
                )
                // You can optionally add .make_span_with() to control the span creation itself
                // If you remove DefaultOnRequest and only use DefaultOnResponse, the span still starts.
                // To completely control it, you might need to create a custom MakeSpan.
                // However, simply overriding .on_request with a higher level effectively hides it.
                .on_request(|request: &axum::extract::Request, _span: &tracing::Span| {
                    // This closure is called when the request starts.
                    // If you return `()` (unit), it does nothing.
                    // You could also log specific things here if needed,
                    // but for "finished only", just leave it as a no-op or set a very high level.
                    // For example:
                    // tracing::event!(Level::TRACE, "Request starting (but we mostly ignore this)");
                })
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3567").await?;
    info!("🚀 Server listening on http://0.0.0.0:3567");
    info!("📚 Docs available at http://0.0.0.0:3567/docs");
    axum::serve(listener, app).await?;
    Ok(())
}
