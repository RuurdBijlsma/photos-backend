use crate::auth::UserRole;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
};
use crate::auth::model::{Claims, User};
use common_photos::get_config;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::PgPool;

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    State<PgPool>: FromRequestParts<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let cfg = get_config();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(cfg.auth.jwt_secret.as_ref()),
            &Validation::default(),
        )
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let user_id = token_data.claims.sub;

        let State(pool) = State::<PgPool>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let user = sqlx::query_as!(
            User,
            r#"SELECT id, email, name, media_folder, role as "role: UserRole",
                      created_at, updated_at
               FROM app_user WHERE id = $1"#,
            user_id
        )
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        parts.extensions.insert(user.clone());
        Ok(user)
    }
}
