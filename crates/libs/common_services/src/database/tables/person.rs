use pgvector::Vector;
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct ExistingPerson {
    pub id: i64,
    pub name: Option<String>,
    #[sqlx(try_from = "Option<pgvector::Vector>")]
    pub centroid: Option<Vector>,
}
