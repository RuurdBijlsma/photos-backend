use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

/// A custom enum that maps to the `user_role` PostgreSQL enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    Admin,
    User,
}

// Implement the Display trait for UserRole
impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "ADMIN"),
            UserRole::User => write!(f, "USER"),
        }
    }
}

/// Represents a user record to be safely sent to clients.
/// Note the absence of the `password` field.
#[derive(Debug, Serialize, FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub media_folder: Option<String>,
    pub role: UserRole,
}

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

#[derive(serde::Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Claims {
    pub sub: i32, // Subject (user id)
    pub role: String,
    pub exp: usize, // Expiration time
}
