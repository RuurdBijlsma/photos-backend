use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(sqlx::Error),

    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}
