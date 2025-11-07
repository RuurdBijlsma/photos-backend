// handlers/cluster_photos.rs

use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::clustering;
use crate::handlers::common::clustering::{Clusterable, ClusteringStrategy};
use crate::handlers::db::model::{ExistingPhotoCluster, PhotoEmbedding};
use async_trait::async_trait;
use color_eyre::Result;
use common_photos::Job;
use pgvector::Vector;
use sqlx::{Transaction, query, query_as, query_scalar};
use std::collections::{HashMap, HashSet};

impl Clusterable for ExistingPhotoCluster {
    fn id(&self) -> i64 {
        self.id
    }
    fn centroid(&self) -> Option<&Vector> {
        self.centroid.as_ref()
    }
}

struct PhotoClusteringStrategy;

#[async_trait]
impl ClusteringStrategy for PhotoClusteringStrategy {
    type ExistingCluster = ExistingPhotoCluster;
    type Embedding = PhotoEmbedding;

    const ENTITY_NAME: &'static str = "photo";
    const MIN_ITEMS_TO_CLUSTER: usize = 3;
    const MIN_SAMPLES: usize = 4;
    const CENTROID_MATCH_THRESHOLD: f32 = 0.6;

    fn embedding_vector(item: &Self::Embedding) -> Vec<f32> {
        item.embedding.to_vec()
    }

    async fn fetch_existing_clusters(
        pool: &sqlx::PgPool,
        user_id: i32,
    ) -> Result<Vec<Self::ExistingCluster>> {
        query_as!(
            ExistingPhotoCluster,
            r#"SELECT id, title, centroid as "centroid: _" FROM photo_cluster WHERE user_id = $1"#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_embeddings(pool: &sqlx::PgPool, user_id: i32) -> Result<Vec<Self::Embedding>> {
        // todo: fetch visual analysis that's earliest in percentage (for videos that have multiple visual analyses)
        query_as!(
            PhotoEmbedding,
            r#"SELECT DISTINCT ON (media_item.id)
                   media_item.id as media_item_id,
                   va.embedding as "embedding!: Vector"
               FROM visual_analysis va
               JOIN media_item ON media_item.id = va.media_item_id
               WHERE media_item.user_id = $1 AND media_item.deleted = false
               ORDER BY media_item.id, va.created_at"#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    async fn upsert_and_link(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        user_id: i32,
        clusters: HashMap<usize, Vec<&Self::Embedding>>,
        new_centroids: &[Vec<f32>],
        cluster_map: &HashMap<usize, i64>,
    ) -> Result<()> {
        for (cluster_id, photos_in_cluster) in clusters {
            let media_item_ids: Vec<String> = photos_in_cluster
                .iter()
                .map(|p| p.media_item_id.clone())
                .collect();
            let new_centroid = new_centroids
                .get(cluster_id)
                .map(|v| Vector::from(v.clone()));
            let thumbnail_media_item_id = &photos_in_cluster[0].media_item_id;

            let photo_cluster_id = if let Some(existing_id) = cluster_map.get(&cluster_id) {
                query("UPDATE photo_cluster SET centroid = $1, thumbnail_media_item_id = $2, updated_at = now() WHERE id = $3")
                    .bind(&new_centroid).bind(thumbnail_media_item_id).bind(existing_id)
                    .execute(&mut **tx).await?;
                *existing_id
            } else {
                query_scalar("INSERT INTO photo_cluster (user_id, thumbnail_media_item_id, centroid) VALUES ($1, $2, $3) RETURNING id")
                    .bind(user_id).bind(thumbnail_media_item_id).bind(&new_centroid)
                    .fetch_one(&mut **tx).await?
            };

            query!("INSERT INTO media_item_photo_cluster (media_item_id, photo_cluster_id) SELECT unnest($1::varchar[]), $2 ON CONFLICT DO NOTHING", &media_item_ids, photo_cluster_id)
                .execute(&mut **tx).await?;
        }
        Ok(())
    }

    async fn cleanup_obsolete(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        existing_clusters: &[Self::ExistingCluster],
        matched_ids: &HashSet<i64>,
    ) -> Result<()> {
        let obsolete: Vec<i64> = existing_clusters
            .iter()
            .filter(|c| !matched_ids.contains(&c.id))
            .map(|c| c.id)
            .collect();
        if !obsolete.is_empty() {
            query!(
                "DELETE FROM media_item_photo_cluster WHERE photo_cluster_id = ANY($1)",
                &obsolete
            )
            .execute(&mut **tx)
            .await?;
            query!("DELETE FROM photo_cluster WHERE id = ANY($1)", &obsolete)
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }
}

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    clustering::handle::<PhotoClusteringStrategy>(context, job).await
}
