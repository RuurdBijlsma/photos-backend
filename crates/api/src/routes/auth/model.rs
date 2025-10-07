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
            UserRole::Admin => write!(f, "ADMIN"),
            UserRole::User => write!(f, "USER"),
        }
    }
}

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

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    #[schema(value_type = String, format = "password", example = "my-secret-password")]
    pub password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct LoginUser {
    pub email: String,
    #[schema(value_type = String, format = "password", example = "my-secret-password")]
    pub password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
    pub role: UserRole,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ProtectedResponse {
    pub message: String,
    pub user_email: String,
    pub user_id: i32,
}
#[derive(Serialize, Debug, ToSchema)]
pub struct AdminResponse {
    pub message: String,
    pub user_email: String,
    pub user_id: i32,
    pub user_role: UserRole,
}
