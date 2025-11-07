use pgvector::Vector;
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct FaceEmbedding {
    pub id: i64,
    pub media_item_id: String,
    pub embedding: Vector,
}

#[derive(Debug, FromRow, Clone)]
pub struct ExistingPerson {
    pub id: i64,
    pub name: Option<String>,
    #[sqlx(try_from = "Option<pgvector::Vector>")]
    pub centroid: Option<Vector>,
}

/// Represents a single photo's embedding data fetched for clustering.
#[derive(Debug, FromRow, Clone)]
pub struct PhotoEmbedding {
    pub media_item_id: String,
    pub embedding: Vector,
}

#[derive(Debug, FromRow, Clone)]
pub struct ExistingPhotoCluster {
    pub id: i64,
    pub title: Option<String>,
    #[sqlx(try_from = "Option<pgvector::Vector>")]
    pub centroid: Option<Vector>,
}
