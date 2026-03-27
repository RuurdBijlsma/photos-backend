use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::clustering::{self, ClusterEntity};
use crate::handlers::common::utils::get_images_to_analyze;
use color_eyre::{Result, eyre::eyre};
use common_services::api::album::service::get_representative_thumbnail;
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::database::person::ExistingPerson;
use face_id::detector::BoundingBox;
use face_id::helpers::extract_face_thumbnail;
use generate_thumbnails::ffmpeg::FfmpegCommand;
use itertools::Itertools;
use pgvector::Vector;
use sqlx::{PgPool, Transaction, query, query_as, query_scalar};
use std::collections::{HashMap, HashSet};
use tempfile::Builder;
use tracing::info;

const ENTITY_NAME: &str = "face";
const MIN_ITEMS_TO_CLUSTER: usize = 4;
const MIN_SAMPLES: usize = 5;
const CENTROID_MATCH_THRESHOLD: f32 = 0.6;
const THUMBNAIL_SIZE: u32 = 256;
const PADDING_FACTOR: f32 = 1.3;

impl ClusterEntity for ExistingPerson {
    fn id(&self) -> i64 {
        self.id
    }
    fn centroid(&self) -> Option<&Vector> {
        self.centroid.as_ref()
    }
}

struct FaceToCluster {
    id: i64,
    media_item_id: String,
    embedding: Vector,
    position_x: f32,
    position_y: f32,
    width: f32,
    height: f32,
    percentage: i32,
}

async fn fetch_existing_clusters(pool: &PgPool, user_id: i32) -> Result<Vec<ExistingPerson>> {
    query_as!(
        ExistingPerson,
        r#"SELECT id, name, centroid as "centroid: _" FROM person WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

async fn fetch_embeddings(pool: &PgPool, user_id: i32) -> Result<Vec<FaceToCluster>> {
    query_as!(
        FaceToCluster,
        r#"SELECT 
               f.id, 
               va.media_item_id, 
               f.embedding as "embedding!: Vector", 
               f.position_x, 
               f.position_y, 
               f.width, 
               f.height,
               va.percentage
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

async fn extract_and_save_face_thumbnail(
    context: &WorkerContext,
    face: &FaceToCluster,
    person_id: i64,
) -> Result<()> {
    let relative_path =
        MediaItemStore::find_relative_path_by_id(&context.pool, &face.media_item_id)
            .await
            .map_err(|e| eyre!("Database error: {}", e))?
            .ok_or_else(|| eyre!("Media item not found"))?;

    let media_root = &context.settings.ingest.media_root;
    let file_path = media_root.join(&relative_path);

    let images = get_images_to_analyze(
        context,
        &file_path,
        &face.media_item_id,
        &[face.percentage as u64],
        context
            .settings
            .ingest
            .thumbnails
            .heights
            .iter()
            .max()
            .copied(),
    );

    let (_, image_path) = images.into_iter().next().ok_or_else(|| {
        eyre!(
            "No image found for analysis at percentage {}",
            face.percentage
        )
    })?;

    let temp_file = Builder::new().suffix(".png").tempfile()?;
    let temp_path = temp_file.path().to_path_buf();

    let mut ffmpeg = FfmpegCommand::new(&image_path);
    ffmpeg.map_still_output("0:v", &temp_path);
    ffmpeg.run().await?;

    // Load and decode the analysis image (which is already oriented correctly)
    let img = image::ImageReader::open(&temp_path)?
        .with_guessed_format()?
        .decode()?;

    let bbox = BoundingBox {
        x1: face.position_x,
        y1: face.position_y,
        x2: face.position_x + face.width,
        y2: face.position_y + face.height,
    };

    let thumb = extract_face_thumbnail(&img, &bbox, PADDING_FACTOR, THUMBNAIL_SIZE);

    let out_dir = context.settings.ingest.thumbnail_root.join("people");
    tokio::fs::create_dir_all(&out_dir).await?;

    // todo: try webp at least
    let out_path = out_dir.join(format!("{person_id}.webp"));
    thumb.save(out_path)?;

    Ok(())
}

async fn get_represenative_face(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    faces_in_cluster: &[&FaceToCluster],
) -> Result<FaceToCluster> {
    let media_items_in_cluster = faces_in_cluster
        .iter()
        .map(|f| f.media_item_id.clone())
        .unique()
        .collect::<Vec<_>>();
    let representative_media_item_id =
        get_representative_thumbnail(tx, &media_items_in_cluster).await?;
    todo!("return first face that has representative_media_item_id as media_item_id")
}

async fn get_biggest_face(faces_in_cluster: &[&FaceToCluster]) -> Result<FaceToCluster> {
    todo!("Return largest face by bounding box area");
}

async fn upsert_and_link(
    context: &WorkerContext,
    tx: &mut Transaction<'_, sqlx::Postgres>,
    user_id: i32,
    clusters: HashMap<usize, Vec<&FaceToCluster>>,
    new_centroids: &[Vec<f32>],
    cluster_map: &HashMap<usize, i64>,
) -> Result<()> {
    for (cluster_idx, faces_in_cluster) in clusters {
        let face_ids: Vec<i64> = faces_in_cluster.iter().map(|f| f.id).collect();
        let new_centroid = new_centroids
            .get(cluster_idx)
            .map(|v| Vector::from(v.clone()));

        let representative_face = get_represenative_face(tx, &faces_in_cluster).await?;
        let thumbnail_media_item_id = &representative_face.media_item_id;

        let person_id = if let Some(existing_id) = cluster_map.get(&cluster_idx) {
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

        // Extract and save face thumbnail for this person.
        extract_and_save_face_thumbnail(context, &representative_face, person_id).await?;
    }
    Ok(())
}

async fn cleanup_obsolete(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    existing_clusters: &[ExistingPerson],
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

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let user_ids = clustering::fetch_user_ids(&context.pool, job).await?;

    for user_id in user_ids {
        let existing_clusters = fetch_existing_clusters(&context.pool, user_id).await?;
        let items_to_cluster = fetch_embeddings(&context.pool, user_id).await?;

        if items_to_cluster.len() < MIN_ITEMS_TO_CLUSTER {
            continue;
        }

        let embeddings: Vec<Vec<f32>> = items_to_cluster
            .iter()
            .map(|f| f.embedding.to_vec())
            .collect();
        let (labels, new_centroids) =
            clustering::run_hdbscan(&embeddings, MIN_ITEMS_TO_CLUSTER, MIN_SAMPLES)?;

        let cluster_map = clustering::match_centroids(
            &new_centroids,
            &existing_clusters,
            CENTROID_MATCH_THRESHOLD,
        )?;
        let matched_old_ids: HashSet<i64> = cluster_map.values().copied().collect();
        let new_clusters = clustering::group_by_cluster(&labels, &items_to_cluster);

        let mut tx = context.pool.begin().await?;

        upsert_and_link(
            context,
            &mut tx,
            user_id,
            new_clusters,
            &new_centroids,
            &cluster_map,
        )
        .await?;
        cleanup_obsolete(&mut tx, &existing_clusters, &matched_old_ids).await?;

        tx.commit().await?;
        info!("Reconciled {} clusters for user {}", ENTITY_NAME, user_id);
    }

    Ok(JobResult::Done)
}
