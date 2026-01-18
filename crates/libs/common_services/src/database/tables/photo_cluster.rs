use pgvector::Vector;

#[derive(Debug, Clone)]
pub struct ExistingPhotoCluster {
    pub id: i64,
    pub title: Option<String>,
    pub centroid: Option<Vector>,
}
