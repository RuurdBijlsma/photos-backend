use crate::context::WorkerContext;
use crate::handlers::db::model::{ExistingPerson, FaceEmbedding};
use crate::handlers::JobResult;
use color_eyre::{eyre::eyre, Result};
use common_photos::Job;
use hdbscan::{Center, DistanceMetric, Hdbscan, HdbscanHyperParams};
use pgvector::Vector;
use sqlx::{query, query_as, query_scalar, PgPool, Transaction};
use std::collections::{HashMap, HashSet};
use tracing::info;

const CENTROID_MATCH_THRESHOLD: f32 = 0.6;
const MIN_FACES_TO_CLUSTER: usize = 5;

pub fn l2_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != b.len() {
        return Err(eyre!("Vectors must have the same dimension"));
    }
    let sum_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();
    Ok(sum_sq.sqrt())
}

pub async fn fetch_user_ids(pool: &PgPool, job: &Job) -> Result<Vec<i32>> {
    if let Some(user_id) = job.user_id {
        Ok(vec![user_id])
    } else {
        let ids = query_scalar!("SELECT id FROM app_user")
            .fetch_all(pool)
            .await?;
        Ok(ids)
    }
}

pub async fn fetch_existing_people(pool: &PgPool, user_id: i32) -> Result<Vec<ExistingPerson>> {
    let people = query_as!(
        ExistingPerson,
        r#"SELECT id, name, centroid as "centroid: _" FROM person WHERE user_id = $1"#,
        user_id
    )
        .fetch_all(pool)
        .await?;
    Ok(people)
}

pub async fn fetch_face_embeddings(pool:&PgPool, user_id: i32) -> Result<Vec<FaceEmbedding>> {
    let faces = query_as!(
        FaceEmbedding,
        r#"SELECT
               face.id,
               visual_analysis.media_item_id,
               face.embedding as "embedding!: Vector"
           FROM face
           JOIN visual_analysis ON face.visual_analysis_id = visual_analysis.id
           JOIN media_item ON media_item.id = visual_analysis.media_item_id
           WHERE media_item.user_id = $1"#,
        user_id
    )
        .fetch_all(pool)
        .await?;
    Ok(faces)
}

pub fn run_hdbscan(embeddings: &[Vec<f32>]) -> Result<(Vec<i32>, Vec<Vec<f32>>)> {
    let params = HdbscanHyperParams::builder()
        .min_cluster_size(4)
        .min_samples(5)
        .allow_single_cluster(false)
        .dist_metric(DistanceMetric::Euclidean)
        .build();

    let clusterer = Hdbscan::new(embeddings, params);
    let labels = clusterer.cluster()?;
    let centroids = clusterer.calc_centers(Center::Centroid, &labels)?;
    Ok((labels, centroids))
}

pub fn match_centroids(
    new_centroids: &[Vec<f32>],
    existing_people: &[ExistingPerson],
) -> Result<HashMap<usize, i64>> {
    let mut map = HashMap::new();
    let mut used_old = HashSet::new();

    for (cid, centroid) in new_centroids.iter().enumerate() {
        let mut best: Option<(i64, f32)> = None;
        for person in existing_people.iter().filter(|p| p.centroid.is_some()) {
            let old = person.centroid.as_ref().unwrap();
            let distance = l2_distance(centroid.as_slice(), old.as_slice())?;
            if distance < CENTROID_MATCH_THRESHOLD {
                match best {
                    None => best = Some((person.id, distance)),
                    Some((_, d)) if distance < d => best = Some((person.id, distance)),
                    _ => {}
                }
            }
        }
        if let Some((id, _)) = best
            && !used_old.contains(&id) {
                map.insert(cid, id);
                used_old.insert(id);
            }
    }

    Ok(map)
}

#[must_use]
pub fn group_faces_by_cluster<'a>(
    labels: &[i32],
    faces: &'a [FaceEmbedding],
) -> HashMap<usize, Vec<&'a FaceEmbedding>> {
    let mut clusters: HashMap<usize, Vec<&FaceEmbedding>> = HashMap::new();
    for (i, &label) in labels.iter().enumerate() {
        if label >= 0 {
            clusters.entry(label as usize).or_default().push(&faces[i]);
        }
    }
    clusters
}

pub async fn upsert_people_and_link_faces<S: ::std::hash::BuildHasher>(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    user_id: i32,
    clusters: HashMap<usize, Vec<&FaceEmbedding>, S>,
    new_centroids: &[Vec<f32>],
    cluster_to_person: &HashMap<usize, i64, S>,
) -> Result<()> {
    for (cluster_id, faces_in_cluster) in clusters {
        let face_ids: Vec<i64> = faces_in_cluster.iter().map(|f| f.id).collect();
        let new_centroid_vec = new_centroids.get(cluster_id).map(|v| Vector::from(v.clone()));
        let thumbnail_media_item_id = &faces_in_cluster[0].media_item_id;

        let person_id: i64 = if let Some(existing) = cluster_to_person.get(&cluster_id) {
            query(
                "UPDATE person SET centroid = $1, thumbnail_media_item_id = $2, updated_at = now() WHERE id = $3",
            )
                .bind(&new_centroid_vec)
                .bind(thumbnail_media_item_id)
                .bind(existing)
                .execute(&mut **tx)
                .await?;
            *existing
        } else {
            query_scalar(
                "INSERT INTO person (user_id, thumbnail_media_item_id, centroid) VALUES ($1, $2, $3) RETURNING id",
            )
                .bind(user_id)
                .bind(thumbnail_media_item_id)
                .bind(&new_centroid_vec)
                .fetch_one(&mut **tx)
                .await?
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

pub async fn cleanup_obsolete<S: ::std::hash::BuildHasher>(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    existing_people: &[ExistingPerson],
    matched_ids: &HashSet<i64, S>,
) -> Result<()> {
    let obsolete: Vec<i64> = existing_people
        .iter()
        .filter(|p| !matched_ids.contains(&p.id))
        .map(|p| p.id)
        .collect();

    if !obsolete.is_empty() {
        query!("UPDATE face SET person_id = NULL WHERE person_id = ANY($1)", &obsolete)
            .execute(&mut **tx)
            .await?;
        query!("DELETE FROM person WHERE id = ANY($1)", &obsolete)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let user_ids = fetch_user_ids(&context.pool, job).await?;

    for user_id in user_ids {
        let existing_people = fetch_existing_people(&context.pool, user_id).await?;
        let face_data = fetch_face_embeddings(&context.pool, user_id).await?;

        if face_data.len() < MIN_FACES_TO_CLUSTER {
            continue;
        }

        let embeddings: Vec<Vec<f32>> = face_data.iter().map(|f| f.embedding.to_vec()).collect();
        let (labels, new_centroids) = run_hdbscan(&embeddings)?;

        let mut tx = context.pool.begin().await?;

        let cluster_to_person = match_centroids(&new_centroids, &existing_people)?;
        let matched_old_person_ids: HashSet<i64> =
            cluster_to_person.values().copied().collect();

        let clusters = group_faces_by_cluster(&labels, &face_data);
        upsert_people_and_link_faces(
            &mut tx,
            user_id,
            clusters,
            &new_centroids,
            &cluster_to_person,
        )
            .await?;

        cleanup_obsolete(&mut tx, &existing_people, &matched_old_person_ids).await?;

        tx.commit().await?;
        info!("Reconciled face clusters for user {}", user_id);
    }

    Ok(JobResult::Done)
}
