use crate::api_state::ApiContext;
use crate::auth::middlewares::common::{decode_token, extract_context, extract_token};
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts

    ,
};
use tracing::{error, info};
use common_services::api::auth::error::AuthError;
use common_services::database::app_user::User;
use common_services::database::user_store::UserStore;

#[derive(Clone, Debug)]
pub struct WsUser(pub User);

/// Get auth token from WebSocket Protocol Header.
/// Expected format: `["access_token", "YOUR_JWT"]`
fn extract_websocket_token(parts: &Parts) -> Result<String, AuthError> {
    // 1. Try standard header first (tools like Postman can do this)
    if let Ok(token) = extract_token(parts) {
        return Ok(token);
    }

    // 2. Try Sec-WebSocket-Protocol
    let proto_header = parts
        .headers
        .get("Sec-WebSocket-Protocol")
        .ok_or(AuthError::MissingToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?;

    // Split "access_token, <jwt>" by comma and trim
    let parts: Vec<&str> = proto_header.split(',').map(str::trim).collect();

    // We look for the "access_token" key and take the NEXT item as the token
    let token_index = parts.iter().position(|&p| p == "access_token");

    match token_index {
        Some(idx) if idx + 1 < parts.len() => Ok(parts[idx + 1].to_string()),
        _ => Err(AuthError::MissingToken),
    }
}

impl<S> FromRequestParts<S> for WsUser
where
    S: Send + Sync,
    State<ApiContext>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Log incoming headers for debugging
        if let Some(proto) = parts.headers.get("Sec-WebSocket-Protocol") {
            info!("🔍 Incoming WS Protocol: {:?}", proto);
        } else {
            info!("⚠️ No Sec-WebSocket-Protocol header found");
        }

        // 2. Extract Token
        let token = match extract_websocket_token(parts) {
            Ok(t) => t,
            Err(e) => {
                error!("❌ WS Token Extraction failed: {:?}", e);
                return Err(e);
            }
        };

        // 3. Extract Context
        let context = extract_context(parts, state).await?;

        // 4. Decode Token
        let claims = match decode_token(&token, &context.settings.secrets.jwt) {
            Ok(c) => c,
            Err(e) => {
                error!("❌ Token Decoding failed: {:?}", e);
                return Err(e);
            }
        };

        // 5. Lookup User (Database check)
        let user_result = UserStore::find_by_id(&context.pool, claims.sub).await;

        match user_result {
            Ok(Some(user)) => {
                parts.extensions.insert(user.clone());
                Ok(Self(user))
            }
            Ok(None) => {
                error!("❌ User ID {} from token not found in DB", claims.sub);
                Err(AuthError::UserNotFound)
            }
            Err(e) => {
                error!("❌ Database error looking up user: {:?}", e);
                Err(AuthError::Internal(e.into())) // likely the cause of 500
            }
        }
    }
}