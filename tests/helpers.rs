use sea_orm::{ConnectionTrait, DatabaseConnection};

pub async fn teardown(db: &DatabaseConnection) {
    db.execute_unprepared("DROP EXTENSION IF EXISTS vector CASCADE")
        .await
        .expect("Ono!");
}
