use crate::api::search::error::SearchError;
use crate::api::search::interfaces::{
    SearchFilterRanges, SearchMediaConfig, SearchMediaType, SearchSortBy,
};
use crate::api::search::search_variants::{
    advanced_search_media, basic_search_media, filter_only_search_media,
};
use crate::database::app_user::User;
use common_types::pb::api::{
    SearchSuggestion, SearchSuggestionsResponse, SimpleTimelineItem, SuggestionType,
};
use image::DynamicImage;
use open_clip_inference::{TextEmbedder, VisionEmbedder};
use sqlx::PgPool;
use std::sync::Arc;

pub async fn search_media(
    user: &User,
    pool: &PgPool,
    embedder: Arc<TextEmbedder>,
    query: &str,
    config: SearchMediaConfig,
) -> Result<Vec<SimpleTimelineItem>, SearchError> {
    if query.trim().is_empty() {
        if has_active_filters(&config) {
            return filter_only_search_media(user, pool, config).await;
        }
        return Ok(vec![]);
    }

    if config.media_type == SearchMediaType::All
        && config.sort_by == SearchSortBy::Relevancy
        && config.start_date.is_none()
        && config.end_date.is_none()
        && config.negative_query.is_none()
        && config.face_names.is_empty()
        && config.country_codes.is_empty()
    {
        basic_search_media(user, pool, embedder, query, config).await
    } else {
        advanced_search_media(user, pool, embedder, query, config).await
    }
}

pub async fn search_by_image(
    user: &User,
    pool: &PgPool,
    text_embedder: Arc<TextEmbedder>,
    vision_embedder: Arc<VisionEmbedder>,
    query: Option<String>,
    img: &DynamicImage,
    config: SearchMediaConfig,
) -> Result<Vec<SimpleTimelineItem>, SearchError> {
    let image_embedding = vision_embedder.embed_image(img)?;

    todo!()
}

pub async fn search_filter_ranges(
    user: &User,
    pool: &PgPool,
) -> Result<SearchFilterRanges, SearchError> {
    let months_task = sqlx::query!(
        r#"
        SELECT DISTINCT month_id AS "months!"
        FROM media_item
        WHERE user_id = $1
          AND deleted = false
        ORDER BY month_id
        "#,
        user.id
    )
    .fetch_all(pool);
    let countries_task = sqlx::query!(
        r#"
        SELECT DISTINCT l.country_code, l.country_name
        FROM location l
        JOIN gps g ON l.id = g.location_id
        JOIN media_item mi ON g.media_item_id = mi.id
        WHERE mi.user_id = $1 AND mi.deleted = false
        ORDER BY l.country_name
        "#,
        user.id
    )
    .fetch_all(pool);
    let people_task = sqlx::query!(
        r#"
        SELECT DISTINCT name, id
        FROM person
        WHERE user_id = $1 AND name IS NOT NULL AND name != ''
        ORDER BY name
        "#,
        user.id
    )
    .fetch_all(pool);

    let (available_month_records, countries_records, people_records) =
        tokio::try_join!(months_task, countries_task, people_task)?;
    let countries = countries_records
        .into_iter()
        .map(|c| (c.country_code, c.country_name))
        .collect();
    let people = people_records
        .into_iter()
        .filter_map(|c| c.name.map(|name| (name, c.id.clone())))
        .collect();
    let available_months = available_month_records.iter().map(|r| r.months).collect();

    Ok(SearchFilterRanges {
        available_months,
        people,
        countries,
    })
}

fn has_active_filters(config: &SearchMediaConfig) -> bool {
    config.media_type != SearchMediaType::All
        || config.start_date.is_some()
        || config.end_date.is_some()
        || !config.country_codes.is_empty()
        || !config.face_names.is_empty()
        || config.negative_query.is_some()
}

pub async fn get_search_suggestions(
    user: &User,
    pool: &PgPool,
    query: &str,
    limit: Option<i64>,
) -> Result<SearchSuggestionsResponse, SearchError> {
    if query.trim().is_empty() {
        return Ok(SearchSuggestionsResponse::default());
    }

    let limit = limit.unwrap_or(10).min(50);
    let ilike_query = format!("%{query}%");
    let suggestions = sqlx::query!(
        r#"
        WITH matched_terms AS (
            (SELECT c.search_term as suggestion, COUNT(DISTINCT va.media_item_id) as photo_count, 'SEARCH' as "type!", NULL as "id"
            FROM classification c
            JOIN visual_analysis va ON c.visual_analysis_id = va.id
            WHERE va.user_id = $1
              AND c.search_term ILIKE $2
              AND c.search_term != ''
            GROUP BY c.search_term
            LIMIT $3 * 2)

            UNION ALL

            (SELECT p.name as suggestion, COUNT(DISTINCT va.media_item_id) as photo_count, 'PERSON' as "type!", p.id::text as "id"
            FROM person p
            JOIN face_cluster fc ON fc.person_id = p.id
            JOIN face f ON f.face_cluster_id = fc.id
            JOIN visual_analysis va ON f.visual_analysis_id = va.id
            WHERE p.user_id = $1
              AND p.name ILIKE $2
              AND p.name != ''
            GROUP BY p.name, p.id
            LIMIT $3 * 2)

            UNION ALL

            (SELECT loc.val as suggestion, COUNT(DISTINCT g.media_item_id) as photo_count, 'SEARCH' as "type!", NULL as "id"
            FROM (
                SELECT id, name as val FROM location WHERE name ILIKE $2
                UNION
                SELECT id, admin1 as val FROM location WHERE admin1 ILIKE $2
                UNION
                SELECT id, country_name as val FROM location WHERE country_name ILIKE $2
            ) loc
            JOIN gps g ON g.location_id = loc.id
            JOIN media_item mi ON g.media_item_id = mi.id
            WHERE mi.user_id = $1 AND mi.deleted = false
            GROUP BY loc.val
            LIMIT $3 * 2)

            UNION ALL

            (SELECT a.name as suggestion, COUNT(DISTINCT am.media_item_id) as photo_count, 'ALBUM' as "type!", a.id::text as "id"
            FROM album a
            LEFT JOIN album_media_item am ON a.id = am.album_id
            LEFT JOIN album_collaborator ac ON a.id = ac.album_id AND ac.user_id = $1
            WHERE (a.owner_id = $1 OR ac.user_id IS NOT NULL)
              AND a.name ILIKE $2
              AND a.name != ''
            GROUP BY a.name, a.id
            LIMIT $3 * 2)

            UNION ALL

            (SELECT o.tag as suggestion, COUNT(DISTINCT va.media_item_id) as photo_count, 'SEARCH' as "type!", NULL as "id"
            FROM object o
            JOIN visual_analysis va ON o.visual_analysis_id = va.id
            WHERE va.user_id = $1
              AND o.tag ILIKE $2
              AND o.tag != ''
            GROUP BY o.tag
            LIMIT $3 * 2)
        )
        SELECT suggestion as "suggestion!", "type!" as "type!", "id" as "id?", SUM(photo_count)::int8 as "photo_count!"
        FROM matched_terms
        GROUP BY suggestion, "type!", "id"
        ORDER BY (CASE WHEN "type!" = 'ALBUM' THEN 0 ELSE (CASE WHEN "type!" = 'PERSON' THEN 1 ELSE 2 END) END), "photo_count!" DESC, suggestion ASC
        LIMIT $3
        "#,
        user.id,
        ilike_query,
        limit as i32
    )
        .fetch_all(pool)
        .await?;

    Ok(SearchSuggestionsResponse {
        suggestions: suggestions
            .into_iter()
            .map(|row| SearchSuggestion {
                text: row.suggestion,
                suggestion_type: match row.r#type.as_str() {
                    "ALBUM" => SuggestionType::Album as i32,
                    "PERSON" => SuggestionType::Person as i32,
                    _ => SuggestionType::Search as i32,
                },
                id: row.id,
            })
            .collect(),
    })
}

pub async fn get_random_search_suggestion(
    user: &User,
    pool: &PgPool,
) -> Result<Option<String>, SearchError> {
    let rows = sqlx::query!(
        r#"
        WITH matched_terms AS (
            (SELECT c.search_term as suggestion
            FROM classification c
            JOIN visual_analysis va ON c.visual_analysis_id = va.id
            WHERE va.user_id = $1
              AND c.search_term != ''
            GROUP BY c.search_term
            ORDER BY COUNT(DISTINCT va.media_item_id) DESC
            LIMIT 100)

            UNION ALL

            (SELECT p.name as suggestion
            FROM person p
            JOIN face_cluster fc ON fc.person_id = p.id
            JOIN face f ON f.face_cluster_id = fc.id
            JOIN visual_analysis va ON f.visual_analysis_id = va.id
            WHERE p.user_id = $1
              AND p.name != ''
            GROUP BY p.name
            ORDER BY COUNT(DISTINCT va.media_item_id) DESC
            LIMIT 100)

            UNION ALL

            (SELECT val as suggestion
            FROM (
                (SELECT l.name as val, COUNT(mi.id) as cnt
                FROM location l
                JOIN gps g ON g.location_id = l.id
                JOIN media_item mi ON g.media_item_id = mi.id
                WHERE mi.user_id = $1 AND mi.deleted = false AND l.name != ''
                GROUP BY l.name
                LIMIT 100)
                UNION ALL
                (SELECT l.admin1 as val, COUNT(mi.id) as cnt
                FROM location l
                JOIN gps g ON g.location_id = l.id
                JOIN media_item mi ON g.media_item_id = mi.id
                WHERE mi.user_id = $1 AND mi.deleted = false AND l.admin1 != ''
                GROUP BY l.admin1
                LIMIT 100)
                UNION ALL
                (SELECT l.country_name as val, COUNT(mi.id) as cnt
                FROM location l
                JOIN gps g ON g.location_id = l.id
                JOIN media_item mi ON g.media_item_id = mi.id
                WHERE mi.user_id = $1 AND mi.deleted = false AND l.country_name != ''
                GROUP BY l.country_name
                LIMIT 100)
            ) locs
            ORDER BY cnt DESC
            LIMIT 100)

            UNION ALL

            (SELECT a.name as suggestion
            FROM album a
            LEFT JOIN album_collaborator ac ON a.id = ac.album_id AND ac.user_id = $1
            JOIN album_media_item am ON a.id = am.album_id
            WHERE (a.owner_id = $1 OR ac.user_id IS NOT NULL)
              AND a.name != ''
            GROUP BY a.name, a.id
            ORDER BY COUNT(DISTINCT am.media_item_id) DESC
            LIMIT 100)

            UNION ALL

            (SELECT o.tag as suggestion
            FROM object o
            JOIN visual_analysis va ON o.visual_analysis_id = va.id
            WHERE va.user_id = $1
              AND o.tag != ''
            GROUP BY o.tag
            ORDER BY RANDOM()
            LIMIT 100)
        )
        SELECT suggestion as "suggestion!"
        FROM matched_terms
        ORDER BY RANDOM()
        -- `LIMIT 500` because I get like 95%/5% ratio of locations/objects otherwise
        -- I think Postgres is optimizing something away if I `LIMIT 1`
        -- This endpoint doesn't have to be fast anyway
        LIMIT 500
        "#,
        user.id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.first().map(|r| r.suggestion.clone()))
}
