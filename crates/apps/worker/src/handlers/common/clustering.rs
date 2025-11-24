use crate::context::WorkerContext;
use crate::handlers::JobResult;
use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use common_services::database::jobs::Job;
use common_services::database::user_store::UserStore;
use hdbscan::{Center, DistanceMetric, Hdbscan, HdbscanHyperParams};
use pgvector::Vector;
use sqlx::{PgPool, Transaction};
use std::collections::{HashMap, HashSet};
use tracing::info;

/// A trait for items that can be clustered. They must provide an ID and a centroid vector.
pub trait Clusterable {
    fn id(&self) -> i64;
    fn centroid(&self) -> Option<&Vector>;
}

/// A trait that defines the specific operations for a type of clustering (e.g., for faces or photos).
/// This allows the main `handle` function to be generic.
#[async_trait]
pub trait ClusteringStrategy {
    /// The type representing an existing cluster (e.g., `ExistingPerson` or `ExistingPhotoCluster`).
    type ExistingCluster: Clusterable + Send + Sync;
    /// The type representing a single item with an embedding (e.g., `FaceEmbedding` or `PhotoEmbedding`).
    type Embedding: Send + Sync;

    /// The name of the entity being clustered, for logging purposes (e.g., "face", "photo").
    const ENTITY_NAME: &'static str;
    /// The minimum number of items required to run the clustering algorithm.
    const MIN_ITEMS_TO_CLUSTER: usize;
    /// The minimum number of samples for hdbscan.
    const MIN_SAMPLES: usize;
    /// The distance threshold for matching a new cluster centroid to an existing one.
    const CENTROID_MATCH_THRESHOLD: f32;

    /// Extracts the embedding vector from an item.
    fn embedding_vector(item: &Self::Embedding) -> Vec<f32>;

    /// Fetches all existing clusters for a given user.
    async fn fetch_existing_clusters(
        pool: &PgPool,
        user_id: i32,
    ) -> Result<Vec<Self::ExistingCluster>>;

    /// Fetches all embeddings for a given user.
    async fn fetch_embeddings(pool: &PgPool, user_id: i32) -> Result<Vec<Self::Embedding>>;

    /// Creates or updates clusters and links the items to them.
    async fn upsert_and_link(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        user_id: i32,
        clusters: HashMap<usize, Vec<&Self::Embedding>>,
        new_centroids: &[Vec<f32>],
        cluster_map: &HashMap<usize, i64>,
    ) -> Result<()>;

    /// Cleans up obsolete clusters that no longer have associated items.
    async fn cleanup_obsolete(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        existing_clusters: &[Self::ExistingCluster],
        matched_ids: &HashSet<i64>,
    ) -> Result<()>;
}

/// Calculates the L2 (Euclidean) distance between two equal-length vectors.
fn l2_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != b.len() {
        return Err(eyre!("Vectors must have the same dimension"));
    }
    Ok(a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt())
}

/// Runs the HDBSCAN algorithm to find clusters and their centroids.
pub fn run_hdbscan(
    embeddings: &[Vec<f32>],
    min_cluster_size: usize,
    min_samples: usize,
) -> Result<(Vec<i32>, Vec<Vec<f32>>)> {
    let params = HdbscanHyperParams::builder()
        .min_cluster_size(min_cluster_size)
        .min_samples(min_samples)
        .allow_single_cluster(false)
        .dist_metric(DistanceMetric::Euclidean)
        .build();

    let clusterer = Hdbscan::new(embeddings, params);
    let labels = clusterer.cluster()?;
    let centroids = clusterer.calc_centers(Center::Centroid, &labels)?;
    Ok((labels, centroids))
}

/// Matches new centroids to existing clusters if they are within a distance threshold.
fn match_centroids<T: Clusterable>(
    new_centroids: &[Vec<f32>],
    existing_clusters: &[T],
    threshold: f32,
) -> Result<HashMap<usize, i64>> {
    let mut map = HashMap::new();
    let mut used_old_ids = HashSet::new();

    for (new_cid, new_centroid) in new_centroids.iter().enumerate() {
        let mut best_match: Option<(i64, f32)> = None;
        for existing_cluster in existing_clusters {
            if let Some(existing_centroid) = existing_cluster.centroid() {
                let distance = l2_distance(new_centroid.as_slice(), existing_centroid.as_slice())?;
                if distance < threshold {
                    if let Some((_, best_dist)) = best_match {
                        if distance < best_dist {
                            best_match = Some((existing_cluster.id(), distance));
                        }
                    } else {
                        best_match = Some((existing_cluster.id(), distance));
                    }
                }
            }
        }

        if let Some((id, _)) = best_match
            && used_old_ids.insert(id)
        {
            map.insert(new_cid, id);
        }
    }
    Ok(map)
}

/// Groups items into a map based on their assigned cluster label.
pub fn group_by_cluster<'a, T>(labels: &[i32], items: &'a [T]) -> HashMap<usize, Vec<&'a T>> {
    let mut clusters: HashMap<usize, Vec<&'a T>> = HashMap::new();
    for (i, &label) in labels.iter().enumerate() {
        if label >= 0 {
            clusters.entry(label as usize).or_default().push(&items[i]);
        }
    }
    clusters
}

/// Fetches all user IDs to process, or a single user ID if specified in the job.
async fn fetch_user_ids(pool: &PgPool, job: &Job) -> Result<Vec<i32>> {
    if let Some(user_id) = job.user_id {
        Ok(vec![user_id])
    } else {
        Ok(UserStore::list_user_ids(pool).await?)
    }
}

/// The main generic handler that orchestrates the entire clustering and reconciliation process.
pub async fn handle<S: ClusteringStrategy + Sync>(
    context: &WorkerContext,
    job: &Job,
) -> Result<JobResult> {
    let user_ids = fetch_user_ids(&context.pool, job).await?;

    for user_id in user_ids {
        let existing_clusters = S::fetch_existing_clusters(&context.pool, user_id).await?;
        let items_to_cluster = S::fetch_embeddings(&context.pool, user_id).await?;

        if items_to_cluster.len() < S::MIN_ITEMS_TO_CLUSTER {
            continue;
        }

        let embeddings: Vec<Vec<f32>> = items_to_cluster.iter().map(S::embedding_vector).collect();
        let (labels, new_centroids) =
            run_hdbscan(&embeddings, S::MIN_ITEMS_TO_CLUSTER, S::MIN_SAMPLES)?;

        let cluster_map = match_centroids(
            &new_centroids,
            &existing_clusters,
            S::CENTROID_MATCH_THRESHOLD,
        )?;
        let matched_old_ids: HashSet<i64> = cluster_map.values().copied().collect();
        let new_clusters = group_by_cluster(&labels, &items_to_cluster);

        let mut tx = context.pool.begin().await?;

        S::upsert_and_link(&mut tx, user_id, new_clusters, &new_centroids, &cluster_map).await?;
        S::cleanup_obsolete(&mut tx, &existing_clusters, &matched_old_ids).await?;

        tx.commit().await?;
        info!(
            "Reconciled {} clusters for user {}",
            S::ENTITY_NAME,
            user_id
        );
    }

    Ok(JobResult::Done)
}
