use crate::api_state::ApiContext;
use axum::extract::State;
use axum::http::StatusCode;
use tracing::error;

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Root message")
    )
)]
pub async fn root() -> &'static str {
    "Hello, World!"
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "API is healthy and ready to accept traffic", body = String),
        (status = 503, description = "API is not healthy, likely due to a database issue.")
    )
)]
pub async fn health_check(State(context): State<ApiContext>) -> Result<&'static str, StatusCode> {
    println!("HEALTH CHECK TIME");
    match sqlx::query("SELECT 1").fetch_one(&context.pool).await {
        Ok(_) => Ok("OK"),
        Err(e) => {
            error!("Health check failed: database connection error: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}