use crate::routes::auth::structs::{Claims, User, UserRole};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use common_photos::get_config;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::PgPool;

pub async fn auth(
    State(pool): State<PgPool>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut request: Request<axum::body::Body>, // The body type is now explicit
    next: Next,                             // `Next` no longer takes a generic parameter
) -> Result<Response, StatusCode> {
    let secret = &get_config().auth.jwt_secret;

    let token_data = decode::<Claims>(
        auth_header.token(),
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // The SQL query is now fixed:
    // 1. We select columns explicitly instead of using `*`.
    // 2. We omit the `password` column.
    // 3. We provide the type hint for the `role` column to fix the mapping error.
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, media_folder, created_at, updated_at, role as "role: UserRole"
        FROM app_user 
        WHERE id = $1
        "#,
        token_data.claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // This part adds the user data to the request, so handlers
    // after this middleware can access it.
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}
