use crate::database::DbError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres};
use std::fmt;
use utoipa::ToSchema;

pub async fn get_user_by_email(
    executor: impl Executor<'_, Database = Postgres>,
    email: &str,
) -> Result<Option<User>, DbError> {
    Ok(sqlx::query_as!(
        User,
        r#"SELECT 
            id, email, name, media_folder, 
            created_at, updated_at,
            role as "role: UserRole"
        FROM app_user 
        WHERE email = $1"#,
        email
    )
    .fetch_optional(executor)
    .await?)
}

/// Derives the user ID from a given relative path by extracting the username and querying the database.
/// # Errors
///
/// * If the username cannot be extracted from the path.
/// * If the database query to find the user by username fails.
/// * If no user is found for the extracted username.
pub async fn user_from_relative_path<'c, E>(
    relative_path: &str,
    executor: E,
) -> color_eyre::Result<Option<User>>
where
    E: Executor<'c, Database = Postgres>,
{
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, created_at, updated_at, email, name, media_folder, role as "role: UserRole"
        FROM app_user
        WHERE media_folder IS NOT null
    "#
    )
    .fetch_all(executor)
    .await?;

    let mut best_match: Option<User> = None;
    let mut max_len = 0;

    for user in users {
        if let Some(media_folder) = &user.media_folder
            && relative_path.starts_with(media_folder)
            && media_folder.len() > max_len
        {
            max_len = media_folder.len();
            best_match = Some(user);
        }
    }

    Ok(best_match)
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
#[derive(Debug, FromRow)]
pub struct UserWithPassword {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub media_folder: Option<String>,
    pub role: UserRole,
    pub password: String,
}

/// Maps to the `user_role` Postgres enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
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
