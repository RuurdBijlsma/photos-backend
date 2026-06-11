use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::clustering::{self, ClusterEntity};
use color_eyre::Result;
use common_services::database::jobs::Job;
use common_services::database::photo_cluster::ExistingPhotoCluster;
use common_services::database::visual_analysis::visual_analysis::MediaEmbedding;
use common_services::utils::nice_id;
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::{PgPool, Transaction, query, query_as, query_scalar};
use std::collections::{HashMap, HashSet};
use tracing::info;

const ENTITY_NAME: &str = "photo";
const MIN_ITEMS_TO_CLUSTER: usize = 3;
const MIN_SAMPLES: usize = 4;
const CENTROID_MATCH_THRESHOLD: f32 = 0.6;

impl ClusterEntity for ExistingPhotoCluster {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn centroid(&self) -> Option<&Vector> {
        self.centroid.as_ref()
    }
}

async fn load_vocab_labels() -> Result<Vec<String>> {
    // load every line from assets/*.txt and put in a Vec<string>
    todo!();
}

async fn load_object_tags(pool: &PgPool, user_id: i32) -> Result<Vec<String>> {
    // load every distinct object tag from the db for this user
    todo!();
}

async fn load_all_tags(pool: &PgPool, user_id: i32) -> Result<Vec<String>> {
    let vocab_labels = load_vocab_labels().await?;
    let object_tags = load_object_tags(&pool, user_id).await?;
    // combine vocab labels and object tags
    // make first letter of every string capitalized
    // trim all
    // other normalization?
    // make deduplicate full list
    todo!();
}

async fn load_tag_embeddings(
    pool: &PgPool,
    user_id: i32,
    text_embedder: &TextEmbedder,
) -> Result<()> {
    let tags = load_all_tags(pool, user_id).await?;
    // check which tags are already in the db table `cluster_tags`
    // make `tags_to_process: Vec<String>` that's all tags excluding ones already in db
    // make mapping of every tag to process -> tag to embedding using text_embedder.embed_texts
    // - this returns a Result<Array2<f32>, ClipError>
    // - can't put m all in at once, batch by N items so it doesn't overload memory
    // remove from db tags that aren't in the result of load all tags
    // add to db the newly processed tags
    todo!();
}

async fn fetch_existing_clusters(pool: &PgPool, user_id: i32) -> Result<Vec<ExistingPhotoCluster>> {
    query_as!(
        ExistingPhotoCluster,
        r#"SELECT id, title, centroid as "centroid: _" FROM photo_cluster WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

async fn fetch_embeddings(pool: &PgPool, user_id: i32) -> Result<Vec<MediaEmbedding>> {
    query_as!(
        MediaEmbedding,
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

async fn find_cluster_label(centroid: &Vec<f32>) -> Result<String> {
    // find nearest row from cluster_tags to get a friendly label for the cluster
    todo!()
}

async fn upsert_and_link(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    user_id: i32,
    clusters: HashMap<usize, Vec<&MediaEmbedding>>,
    new_centroids: &[Vec<f32>],
    cluster_map: &HashMap<usize, String>,
) -> Result<()> {
    for (cluster_idx, photos_in_cluster) in clusters {
        let media_item_ids: Vec<String> = photos_in_cluster
            .iter()
            .map(|p| p.media_item_id.clone())
            .collect();
        let new_centroid_vec = new_centroids.get(cluster_idx);
        let new_centroid = new_centroid_vec.map(|v| Vector::from(v.clone()));
        let thumbnail_media_item_id = &photos_in_cluster[0].media_item_id;

        let user_friendly_label = if let Some(centroid) = new_centroid_vec {
            Some(find_cluster_label(centroid).await?)
        } else {
            None
        };

        let photo_cluster_id = if let Some(existing_id) = cluster_map.get(&cluster_idx) {
            query("UPDATE photo_cluster SET centroid = $1, thumbnail_media_item_id = $2, friendly_label = $3, updated_at = now() WHERE id = $4")
                .bind(&new_centroid).bind(thumbnail_media_item_id).bind(user_friendly_label).bind(existing_id)
                .execute(&mut **tx).await?;
            existing_id.to_owned()
        } else {
            query_scalar("INSERT INTO photo_cluster (id, user_id, thumbnail_media_item_id, centroid, friendly_label) VALUES ($1, $2, $3, $4, $5) RETURNING id")
                .bind(nice_id(10))
                .bind(user_id)
                .bind(thumbnail_media_item_id)
                .bind(&new_centroid)
                .bind(user_friendly_label)
                .fetch_one(&mut **tx).await?
        };

        query!("INSERT INTO media_item_photo_cluster (media_item_id, photo_cluster_id) SELECT unnest($1::varchar[]), $2 ON CONFLICT DO NOTHING", &media_item_ids, photo_cluster_id)
            .execute(&mut **tx).await?;
    }
    Ok(())
}

async fn cleanup_obsolete(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    existing_clusters: &[ExistingPhotoCluster],
    matched_ids: &HashSet<String>,
) -> Result<()> {
    let obsolete: Vec<String> = existing_clusters
        .iter()
        .filter(|c| !matched_ids.contains(&c.id))
        .map(|c| c.id.clone())
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

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let user_ids = clustering::fetch_user_ids(&context.pool, job).await?;

    for user_id in user_ids {
        load_tag_embeddings(&context.pool, user_id, &context.text_embedder).await?;

        let existing_clusters = fetch_existing_clusters(&context.pool, user_id).await?;
        let items_to_cluster = fetch_embeddings(&context.pool, user_id).await?;

        if items_to_cluster.len() < MIN_ITEMS_TO_CLUSTER {
            continue;
        }

        let embeddings: Vec<Vec<f32>> = items_to_cluster
            .iter()
            .map(|p| p.embedding.to_vec())
            .collect();
        let (labels, new_centroids) =
            clustering::run_hdbscan(&embeddings, MIN_ITEMS_TO_CLUSTER, MIN_SAMPLES)?;

        let cluster_map = clustering::match_centroids(
            &new_centroids,
            &existing_clusters,
            CENTROID_MATCH_THRESHOLD,
        )?;
        let matched_old_ids: HashSet<String> = cluster_map.values().cloned().collect();
        let new_clusters = clustering::group_by_cluster(&labels, &items_to_cluster);

        let mut tx = context.pool.begin().await?;

        upsert_and_link(&mut tx, user_id, new_clusters, &new_centroids, &cluster_map).await?;
        cleanup_obsolete(&mut tx, &existing_clusters, &matched_old_ids).await?;

        tx.commit().await?;
        info!("Reconciled {} clusters for user {}", ENTITY_NAME, user_id);
    }

    Ok(JobResult::Done)
}
