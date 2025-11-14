use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::clustering;
use crate::handlers::common::clustering::{Clusterable, ClusteringStrategy};
use async_trait::async_trait;
use color_eyre::Result;
use common_services::database::jobs::Job;
use pgvector::Vector;
use sqlx::{Transaction, query, query_as, query_scalar};
use std::collections::{HashMap, HashSet};
use common_services::database::person::ExistingPerson;
use common_services::database::visual_analysis::face::FaceEmbedding;

impl Clusterable for ExistingPerson {
    fn id(&self) -> i64 {
        self.id
    }
    fn centroid(&self) -> Option<&Vector> {
        self.centroid.as_ref()
    }
}

struct FaceClusteringStrategy;

#[async_trait]
impl ClusteringStrategy for FaceClusteringStrategy {
    type ExistingCluster = ExistingPerson;
    type Embedding = FaceEmbedding;

    const ENTITY_NAME: &'static str = "face";
    const MIN_ITEMS_TO_CLUSTER: usize = 4;
    const MIN_SAMPLES: usize = 5;
    const CENTROID_MATCH_THRESHOLD: f32 = 0.6;

    fn embedding_vector(item: &Self::Embedding) -> Vec<f32> {
        item.embedding.to_vec()
    }

    async fn fetch_existing_clusters(
        pool: &sqlx::PgPool,
        user_id: i32,
    ) -> Result<Vec<Self::ExistingCluster>> {
        query_as!(
            ExistingPerson,
            r#"SELECT id, name, centroid as "centroid: _" FROM person WHERE user_id = $1"#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_embeddings(pool: &sqlx::PgPool, user_id: i32) -> Result<Vec<Self::Embedding>> {
        query_as!(
            FaceEmbedding,
            r#"SELECT f.id, va.media_item_id, f.embedding as "embedding!: Vector"
               FROM face f
               JOIN visual_analysis va ON f.visual_analysis_id = va.id
               JOIN media_item mi ON mi.id = va.media_item_id
               WHERE mi.user_id = $1"#,
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
        for (cluster_id, faces_in_cluster) in clusters {
            let face_ids: Vec<i64> = faces_in_cluster.iter().map(|f| f.id).collect();
            let new_centroid = new_centroids
                .get(cluster_id)
                .map(|v| Vector::from(v.clone()));
            let thumbnail_media_item_id = &faces_in_cluster[0].media_item_id;

            let person_id = if let Some(existing_id) = cluster_map.get(&cluster_id) {
                query("UPDATE person SET centroid = $1, thumbnail_media_item_id = $2, updated_at = now() WHERE id = $3")
                    .bind(&new_centroid).bind(thumbnail_media_item_id).bind(existing_id)
                    .execute(&mut **tx).await?;
                *existing_id
            } else {
                query_scalar("INSERT INTO person (user_id, thumbnail_media_item_id, centroid) VALUES ($1, $2, $3) RETURNING id")
                    .bind(user_id).bind(thumbnail_media_item_id).bind(&new_centroid)
                    .fetch_one(&mut **tx).await?
            };

            query!(
                "UPDATE face SET person_id = $1 WHERE id = ANY($2)",
                person_id,
                &face_ids
            )
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    async fn cleanup_obsolete(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        existing_clusters: &[Self::ExistingCluster],
        matched_ids: &HashSet<i64>,
    ) -> Result<()> {
        let obsolete_ids: Vec<i64> = existing_clusters
            .iter()
            .filter(|p| !matched_ids.contains(&p.id))
            .map(|p| p.id)
            .collect();
        if !obsolete_ids.is_empty() {
            query!(
                "UPDATE face SET person_id = NULL WHERE person_id = ANY($1)",
                &obsolete_ids
            )
            .execute(&mut **tx)
            .await?;
            query!("DELETE FROM person WHERE id = ANY($1)", &obsolete_ids)
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }
}

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    clustering::handle::<FaceClusteringStrategy>(context, job).await
}
