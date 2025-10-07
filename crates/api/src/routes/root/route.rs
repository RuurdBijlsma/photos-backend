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