use crate::api_state::ApiContext;
use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{header, request::Parts, Request},
    middleware::Next,
    response::Response,
};
use color_eyre::eyre::eyre;
use common_services::api::auth::error::AuthError;
use common_services::api::auth::interfaces::AuthClaims;
use common_services::database::app_user::{User, UserRole};
use common_services::database::user_store::UserStore;
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Clone, Debug)]
pub struct ApiUser(pub User);

async fn extract_context<S>(parts: &mut Parts, state: &S) -> Result<ApiContext, AuthError>
where
    S: Send + Sync,
    State<ApiContext>: FromRequestParts<S>,
{
    let State(context) = State::<ApiContext>::from_request_parts(parts, state)
        .await
        .map_err(|_| AuthError::Internal(eyre!("Server state is not configured correctly.")))?;
    Ok(context)
}

/// Get auth token from Parts.
fn extract_token(parts: &Parts) -> Result<String, AuthError> {
    let auth_header = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    auth_header
        .strip_prefix("Bearer ")
        .map(ToOwned::to_owned)
        .ok_or(AuthError::InvalidToken)
}

fn decode_token(token: &str, jwt_secret: &str) -> Result<AuthClaims, AuthError> {
    decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidToken)
}

impl<S> FromRequestParts<S> for ApiUser
where
    S: Send + Sync,
    State<ApiContext>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts)?;
        let context = extract_context(parts, state).await?;
        let claims = decode_token(&token, &context.settings.secrets.jwt)?;
        let user = UserStore::find_by_id(&context.pool, claims.sub).await?.ok_or(AuthError::UserNotFound)?;
        parts.extensions.insert(user.clone());
        Ok(Self(user))
    }
}

#[derive(Clone, Debug)]
pub struct OptionalUser(pub Option<User>);

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
    State<ApiContext>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match extract_token(parts) {
            Ok(token) => {
                let context = extract_context(parts, state).await?;
                let claims = decode_token(&token, &context.settings.secrets.jwt)?;
                let user = UserStore::find_by_id(&context.pool, claims.sub).await?.ok_or(AuthError::UserNotFound)?;
                parts.extensions.insert(Self(Some(user.clone())));
                Ok(Self(Some(user)))
            }
            Err(AuthError::MissingToken) => {
                parts.extensions.insert(Self(None));
                Ok(Self(None))
            }
            Err(e) => Err(e),
        }
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
