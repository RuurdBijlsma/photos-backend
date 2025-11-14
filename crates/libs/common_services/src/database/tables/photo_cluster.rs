use pgvector::Vector;
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct ExistingPhotoCluster {
    pub id: i64,
    pub title: Option<String>,
    #[sqlx(try_from = "Option<pgvector::Vector>")]
    pub centroid: Option<Vector>,
}
