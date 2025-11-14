use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{Request, header, request::Parts},
    middleware::Next,
    response::Response,
};
use color_eyre::eyre::eyre;
use common_services::api::auth::error::AuthError;
use common_services::api::auth::interfaces::AuthClaims;
use common_services::database::app_user::{User, UserRole};
use common_services::get_settings::settings;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct ApiUser(pub User);

/// Get `PgPool` from Parts
async fn extract_pool<S>(parts: &mut Parts, state: &S) -> Result<PgPool, AuthError>
where
    S: Send + Sync,
    State<PgPool>: FromRequestParts<S>,
{
    let State(pool) = State::<PgPool>::from_request_parts(parts, state)
        .await
        .map_err(|_| {
            tracing::error!("FATAL: Could not extract PgPool state. Missing `.with_state(pool)`?");
            AuthError::Internal(eyre!("Server state is not configured correctly."))
        })?;
    Ok(pool)
}

/// Get auth token from Parts.
fn extract_token(parts: &Parts) -> Result<&str, AuthError> {
    let auth_header = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)
}

fn decode_token(token: &str) -> Result<AuthClaims, AuthError> {
    let jwt_secret = &settings().auth.jwt_secret;
    decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidToken)
}

async fn fetch_user(pool: &PgPool, user_id: i32) -> Result<User, AuthError> {
    sqlx::query_as!(
        User,
        r#"SELECT id, email, name, media_folder, role as "role: UserRole",
                  created_at, updated_at
           FROM app_user WHERE id = $1"#,
        user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::UserNotFound)
}

impl<S> FromRequestParts<S> for ApiUser
where
    S: Send + Sync,
    State<PgPool>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts)?;
        let claims = decode_token(token)?;
        let pool = extract_pool(parts, state).await?;
        let user = fetch_user(&pool, claims.sub).await?;
        parts.extensions.insert(user.clone());
        Ok(Self(user))
    }
}

#[derive(Clone, Debug)]
pub struct OptionalUser(pub Option<User>);

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
    State<PgPool>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        let Some(token) = token else {
            parts.extensions.insert(Self(None));
            return Ok(Self(None));
        };

        let claims = decode_token(token)?;
        let pool = extract_pool(parts, state).await?;
        let user = fetch_user(&pool, claims.sub).await?;
        parts.extensions.insert(Self(Some(user.clone())));
        Ok(Self(Some(user)))
    }
}

pub async fn require_role(
    State(required_role): State<UserRole>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let user = req
        .extensions()
        .get::<User>()
        .ok_or(AuthError::UserNotFound)?;

    if user.role != required_role {
        return Err(AuthError::PermissionDenied {
            user_email: user.email.clone(),
            path: req.uri().to_string(),
        });
    }

    Ok(next.run(req).await)
}
