mod routes;

use crate::auth::model::User;
use crate::routes::auth;
use crate::routes::auth::middleware::{require_role};
use crate::routes::auth::{check_admin, get_me, login, logout, refresh_session, register, UserRole};
use crate::routes::root::route::root;
use axum::{
    middleware, routing::{get, post},
    Router,
};
use common_photos::get_db_pool;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    start_server().await?;
    Ok(())
}

async fn start_server() -> color_eyre::Result<()> {
    let pool = get_db_pool().await?;

    let public_routes = Router::new()
        .route("/", get(root))
        .route("/auth/refresh", post(refresh_session))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout));

    let protected_routes = Router::new().route("/auth/me", get(get_me)).route_layer(
        middleware::from_extractor_with_state::<User, PgPool>(pool.clone()),
    );

    let admin_routes = Router::new()
        .route("/auth/admin-check", get(check_admin))
        .route_layer(middleware::from_fn_with_state(UserRole::Admin, require_role))
        .route_layer(middleware::from_extractor_with_state::<User, PgPool>(pool.clone()));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3567").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
