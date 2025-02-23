use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS vector")
            .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        // Drop the extension. Using CASCADE ensures that dependent objects (like the type)
        // are also removed.
        db.execute_unprepared("DROP EXTENSION IF EXISTS vector CASCADE")
            .await?;
        Ok(())
    }
}
