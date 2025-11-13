use common_types::app_user::UserRole;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents the data required to create a new user.
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    #[schema(value_type = String, format = "password", example = "my-secret-password")]
    pub password: String,
}

/// Represents the data required for user login.
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginUser {
    pub email: String,
    #[schema(value_type = String, format = "password", example = "my-secret-password")]
    pub password: String,
}

/// Represents the payload for a refresh token request.
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

/// Represents a pair of access and refresh tokens.
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub expiry: u64,
    pub access_token: String,
    pub refresh_token: String,
}

/// Represents the claims contained within a JWT.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthClaims {
    pub sub: i32, // Subject (user ID)
    pub exp: i64, // Expiration time
    pub role: UserRole,
}
