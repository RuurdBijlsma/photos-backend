use color_eyre::Result;
use sqlx::PgExecutor;

pub struct KeyJsonStore;

impl KeyJsonStore {
    pub async fn get_value(
        executor: impl PgExecutor<'_>,
        key: &str,
    ) -> Result<Option<serde_json::Value>> {
        let row = sqlx::query!("SELECT value FROM key_json_store WHERE key = $1", key)
            .fetch_optional(executor)
            .await?;

        Ok(row.and_then(|r| r.value))
    }

    pub async fn set_value(
        executor: impl PgExecutor<'_>,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO key_json_store (key, value, updated_at)
            VALUES ($1, $2, now())
            ON CONFLICT (key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = now()
            "#,
            key,
            value
        )
        .execute(executor)
        .await?;

        Ok(())
    }
}
