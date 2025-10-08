use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use utoipa::ToSchema; // Import ToSchema

/// Maps to the `user_role` Postgres enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    Admin,
    User,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admin => write!(f, "ADMIN"),
            Self::User => write!(f, "USER"),
        }
    }
}

/// Represents a user in the application.
#[derive(Debug, Serialize, FromRow, Clone, ToSchema)]
pub struct User {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub media_folder: Option<String>,
    pub role: UserRole,
}

/// Represents a user record from db, including the password hash.
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserRecord {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents the data required to create a new user.
#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    #[schema(value_type = String, format = "password", example = "my-secret-password")]
    pub password: String,
}

/// Represents the data required for user login.
#[derive(Deserialize, Debug, ToSchema)]
pub struct LoginUser {
    pub email: String,
    #[schema(value_type = String, format = "password", example = "my-secret-password")]
    pub password: String,
}

/// Represents the payload for a refresh token request.
#[derive(Deserialize, Debug, ToSchema)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

/// Represents a pair of access and refresh tokens.
#[derive(Serialize, Debug, ToSchema)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Represents the claims contained within a JWT.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: i32, // Subject (user ID)
    pub exp: i64, // Expiration time
    pub role: UserRole,
}

/// Represents the response for a protected route, containing user details.
#[derive(Serialize, Debug, ToSchema)]
pub struct ProtectedResponse {
    pub message: String,
    pub user_email: String,
    pub user_id: i32,
}

/// Represents the response for an admin-protected route.
#[derive(Serialize, Debug, ToSchema)]
pub struct AdminResponse {
    pub message: String,
    pub user_email: String,
    pub user_id: i32,
    pub user_role: UserRole,
}
