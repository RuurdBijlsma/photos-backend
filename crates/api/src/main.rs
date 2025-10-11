#![allow(clippy::needless_for_each)]

use crate::routes::root::route::__path_root;
use crate::routes::setup::interfaces;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

mod routes;

use crate::routes::auth;
use crate::routes::auth::UserRole;
use crate::routes::auth::handlers::{
    check_admin, get_me, login, logout, refresh_session, register,
};
use crate::routes::auth::middleware::require_role;
use crate::routes::root::route::root;
use crate::routes::scalar_config::get_custom_html;
use crate::routes::setup;
use crate::routes::setup::handlers::{
    get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder,
    setup_needed,
};
use auth::db_model::User;
use axum::{
    Router, middleware,
    routing::{get, post},
};
use common_photos::{get_db_pool, settings};
use sqlx::PgPool;
use tracing::info;
use tracing_subscriber::fmt;
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
        setup::handlers::setup_needed,
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
            auth::interfaces::GetMeResponse,
            auth::interfaces::AdminResponse,
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
    fmt::Subscriber::builder().with_ansi(true).init();
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

    let public_routes = Router::new()
        // ======== [ ROOT ] =========
        .route("/", get(root))
        // ======== [ /setup/ ] =========
        .route("/setup/needed", get(setup_needed))
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
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3567").await?;
    info!("🚀 Server listening on http://0.0.0.0:3567");
    info!("📚 Docs available at http://0.0.0.0:3567/docs");
    axum::serve(listener, app).await?;
    Ok(())
}
