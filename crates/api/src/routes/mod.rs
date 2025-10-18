pub mod auth;
pub mod download;
pub mod root;
pub mod scalar_config;
pub mod setup;

use crate::auth::db_model::User;
use crate::auth::handlers::{get_me, login, logout, refresh_session, register};
use crate::auth::middleware::require_role;
use crate::download::handlers::download_full_file;
use crate::root::handlers::root;
use crate::scalar_config::get_custom_html;
use crate::setup::handlers::{
    get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder,
    welcome_needed,
};
use axum::middleware::{from_extractor_with_state, from_fn_with_state};
use axum::{
    Router,
    routing::{get, post},
};
use common_photos::UserRole;
use sqlx::PgPool;
use tower_http::{LatencyUnit, trace::TraceLayer};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_scalar::{Scalar, Servable};

// --- API Documentation ---
#[derive(OpenApi)]
#[openapi(
    paths(
        root::handlers::root,
        // Auth handlers
        auth::handlers::login,
        auth::handlers::register,
        auth::handlers::refresh_session,
        auth::handlers::logout,
        auth::handlers::get_me,
        // Setup handlers
        setup::handlers::welcome_needed,
        setup::handlers::get_disk_response,
        setup::handlers::get_folder_media_sample,
        setup::handlers::get_folder_unsupported,
        setup::handlers::get_folders,
        setup::handlers::make_folder,
        // Download handlers
        download::handlers::download_full_file,
    ),
    components(
        schemas(
            // Auth schemas
            auth::db_model::User,
            common_photos::UserRole,
            auth::interfaces::CreateUser,
            auth::interfaces::LoginUser,
            auth::interfaces::RefreshTokenPayload,
            auth::interfaces::Tokens,
            // Setup schemas
            setup::interfaces::FolderQuery,
            setup::interfaces::MakeFolderBody,
            setup::interfaces::PathInfoResponse,
            setup::interfaces::MediaSampleResponse,
            setup::interfaces::UnsupportedFilesResponse,
            setup::interfaces::DiskResponse,
            // Download schemas
            download::interfaces::DownloadMediaQuery,
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

// --- Router Construction ---
pub fn create_router(pool: PgPool) -> Router {
    let openapi = ApiDoc::openapi();

    Router::new()
        .merge(Scalar::with_url("/docs", openapi.clone()).custom_html(get_custom_html(&openapi)))
        .merge(public_routes())
        .merge(protected_routes(pool.clone()))
        .merge(admin_routes(pool.clone()))
        .with_state(pool)
        .layer(
            TraceLayer::new_for_http().on_response(
                tower_http::trace::DefaultOnResponse::new()
                    .level(tracing::Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
        )
}

fn public_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(root))
        .route("/setup/welcome-needed", get(welcome_needed))
        .route("/auth/refresh", post(refresh_session))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
}

fn protected_routes(pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/auth/me", get(get_me))
        .route("/download/full-file", get(download_full_file))
        .route_layer(from_extractor_with_state::<User, PgPool>(pool))
}

fn admin_routes(pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/setup/disk-info", get(get_disk_response))
        .route("/setup/media-sample", get(get_folder_media_sample))
        .route("/setup/unsupported-files", get(get_folder_unsupported))
        .route("/setup/folders", get(get_folders))
        .route("/setup/make-folder", post(make_folder))
        .route_layer(from_fn_with_state(UserRole::Admin, require_role))
        .route_layer(from_extractor_with_state::<User, PgPool>(pool))
}
