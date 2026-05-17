use pgvector::Vector;

#[derive(Debug, Clone)]
pub struct ExistingPhotoCluster {
    pub id: String,
    pub title: Option<String>,
    pub centroid: Option<Vector>,
}
