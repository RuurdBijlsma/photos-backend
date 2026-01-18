use pgvector::Vector;

#[derive(Debug, Clone)]
pub struct ExistingPerson {
    pub id: i64,
    pub name: Option<String>,
    pub centroid: Option<Vector>,
}
