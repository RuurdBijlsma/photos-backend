// crates/api/src/routes/mod.rs

pub mod albums;
pub mod auth;
pub mod download;
pub mod photos;
pub mod root;
pub mod scalar_config;
pub mod setup;

use crate::albums::handlers::{
    add_collaborator_handler, add_media_to_album_handler, create_album_handler,
    get_album_details_handler, get_user_albums_handler, remove_collaborator_handler,
    remove_media_from_album_handler, update_album_handler,
};
use crate::auth::db_model::User;
use crate::auth::handlers::{get_me, login, logout, refresh_session, register};
use crate::auth::middleware::require_role;
use crate::download::handlers::download_full_file;
use crate::photos::handlers::{
    get_color_theme_handler, get_full_item_handler, get_photos_by_month_handler, get_random_photo,
    get_timeline_ids_handler, get_timeline_ratios_handler,
};
use crate::root::handlers::root;
use crate::scalar_config::get_custom_html;
use crate::setup::handlers::{
    get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder,
    post_start_processing,
};
use axum::middleware::{from_extractor_with_state, from_fn_with_state};
use axum::routing::delete;
use axum::{
    routing::{get, post},
    Router,
};
use common_photos::UserRole;
use sqlx::PgPool;
use tower_http::{trace::TraceLayer, LatencyUnit};
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
        setup::handlers::get_disk_response,
        setup::handlers::get_folder_media_sample,
        setup::handlers::get_folder_unsupported,
        setup::handlers::get_folders,
        setup::handlers::make_folder,
        // Download handlers
        download::handlers::download_full_file,
        // --- Photos handlers ---
        photos::handlers::get_random_photo,
        // --- Album handlers ---
        albums::handlers::create_album_handler,
        albums::handlers::get_user_albums_handler,
        albums::handlers::get_album_details_handler,
        albums::handlers::update_album_handler,
        albums::handlers::add_media_to_album_handler,
        albums::handlers::remove_media_from_album_handler,
        albums::handlers::add_collaborator_handler,
        albums::handlers::remove_collaborator_handler,
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
            // --- Add Album schemas ---
            albums::db_model::Album,
            albums::db_model::AlbumRole,
            albums::db_model::AlbumCollaborator,
            albums::interfaces::CreateAlbumRequest,
            albums::interfaces::UpdateAlbumRequest,
            albums::interfaces::AddMediaToAlbumRequest,
            albums::interfaces::AddCollaboratorRequest,
            albums::interfaces::AlbumDetailsResponse,
            albums::interfaces::AlbumMediaItemSummary,
            albums::interfaces::CollaboratorSummary,
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Ruurd Photos", description = "Ruurd Photos' API"),
        (name = "Photos", description = "Endpoints for browsing and managing media items"),
        (name = "Albums", description = "Endpoints for managing photo albums and collaboration")
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
                    .latency_unit(LatencyUnit::Millis),
            ),
        )
}

fn public_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(root))
        .route("/auth/refresh", post(refresh_session))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
}

fn protected_routes(pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/auth/me", get(get_me))
        .route("/download/full-file", get(download_full_file))
        .merge(photos_routes())
        .merge(album_routes())
        .route_layer(from_extractor_with_state::<User, PgPool>(pool))
}

fn photos_routes() -> Router<PgPool> {
    Router::new()
        .route("/photos/random", get(get_random_photo))
        .route("/photos/theme", get(get_color_theme_handler))
        .route("/photos/timeline/ratios", get(get_timeline_ratios_handler))
        .route("/photos/timeline/ids", get(get_timeline_ids_handler))
        .route("/photos/by-month", get(get_photos_by_month_handler))
        .route("/photos/item", get(get_full_item_handler))
}

fn album_routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/albums",
            post(create_album_handler).get(get_user_albums_handler),
        )
        .route(
            "/albums/{album_id}",
            get(get_album_details_handler).put(update_album_handler),
        )
        .route("/albums/{album_id}/media", post(add_media_to_album_handler))
        .route(
            "/albums/{album_id}/media/{media_item_id}",
            delete(remove_media_from_album_handler),
        )
        .route(
            "/albums/{album_id}/collaborators",
            post(add_collaborator_handler),
        )
        .route(
            "/albums/{album_id}/collaborators/{collaborator_id}",
            delete(remove_collaborator_handler),
        )
}

fn admin_routes(pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/setup/disk-info", get(get_disk_response))
        .route("/setup/media-sample", get(get_folder_media_sample))
        .route("/setup/unsupported-files", get(get_folder_unsupported))
        .route("/setup/folders", get(get_folders))
        .route("/setup/make-folder", post(make_folder))
        .route("/setup/start-processing", post(post_start_processing))
        .route_layer(from_fn_with_state(UserRole::Admin, require_role))
        .route_layer(from_extractor_with_state::<User, PgPool>(pool))
}
