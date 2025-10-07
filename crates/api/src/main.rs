use crate::routes::root::route::__path_root;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

mod routes;

use crate::auth::model::User;
use crate::routes::auth;
use crate::routes::auth::middleware::require_role;
use crate::routes::auth::{UserRole, handlers};
use crate::routes::root::route::root;
use crate::routes::scalar_config::get_custom_html;
use axum::{
    Router, middleware,
    routing::{get, post},
};
use common_photos::get_db_pool;
use sqlx::PgPool;
use tracing::info;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        handlers::login,
        handlers::register,
        handlers::refresh_session,
        handlers::logout,
        handlers::get_me,
        handlers::check_admin,
    ),
    components(
        schemas(
            crate::auth::model::User,
            crate::auth::model::UserRole,
            crate::auth::model::CreateUser,
            crate::auth::model::LoginUser,
            crate::auth::model::RefreshTokenPayload,
            crate::auth::model::Tokens,
            crate::auth::model::ProtectedResponse,
            crate::auth::model::AdminResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Ruurd Photos", description = "Ruurd Photos' API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    start_server().await?;
    Ok(())
}

async fn start_server() -> color_eyre::Result<()> {
    let pool = get_db_pool().await?;

    // --- routes setup is the same ---
    let public_routes = Router::new()
        .route("/", get(root))
        .route("/auth/refresh", post(handlers::refresh_session))
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/logout", post(handlers::logout));

    let protected_routes = Router::new()
        .route("/auth/me", get(handlers::get_me))
        .route_layer(middleware::from_extractor_with_state::<User, PgPool>(
            pool.clone(),
        ));

    let admin_routes = Router::new()
        .route("/auth/admin-check", get(handlers::check_admin))
        .route_layer(middleware::from_fn_with_state(
            UserRole::Admin,
            require_role,
        ))
        .route_layer(middleware::from_extractor_with_state::<User, PgPool>(
            pool.clone(),
        ));

    // The openapi spec is generated once and can be cloned for each UI
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
