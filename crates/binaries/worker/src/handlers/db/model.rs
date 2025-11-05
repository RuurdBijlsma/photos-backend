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
    pub centroid: Option<Vector>,
}