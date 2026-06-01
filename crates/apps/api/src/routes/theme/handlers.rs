use app_state::IngestSettings;
use axum::extract::{Query, State};
use axum::{Extension, Json};

use common_services::database::app_user::User;

use crate::api_state::ApiContext;
use common_services::api::theme::error::ThemeError;
use common_services::api::theme::interfaces::{
    ColorThemeParams, RandomPhotoParams, RandomPhotoResponse,
};
use common_services::api::theme::service::random_photo_theme;
use material_color_utils::dynamic::variant::Variant;
use material_color_utils::utils::color_utils::Argb;
use material_color_utils::{MaterializedTheme, theme_from_color};

pub async fn get_random_photo_theme(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<RandomPhotoParams>,
) -> Result<Json<Option<RandomPhotoResponse>>, ThemeError> {
    let variant: Variant = serde_json::from_str(&format!("\"{}\"", params.variant))
        .unwrap_or(context.settings.ingest.analyzer.theme_generation.variant);
    let contrast_level = context
        .settings
        .ingest
        .analyzer
        .theme_generation
        .contrast_level;
    let result = random_photo_theme(&user, &context.pool, variant, contrast_level).await?;
    Ok(Json(result))
}

pub async fn get_color_theme_handler(
    State(ingestion): State<IngestSettings>,
    Query(params): Query<ColorThemeParams>,
) -> Result<Json<MaterializedTheme>, ThemeError> {
    let variant: Variant = serde_json::from_str(&format!("\"{}\"", params.variant))
        .unwrap_or(ingestion.analyzer.theme_generation.variant);
    let contrast_level = ingestion.analyzer.theme_generation.contrast_level;
    let theme = theme_from_color(Argb::from_hex(&params.color)?)
        .variant(variant)
        .contrast_level(contrast_level)
        .call();
    Ok(Json(theme))
}
