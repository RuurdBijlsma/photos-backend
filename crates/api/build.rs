fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto/ratios.proto");

    let mut config = prost_build::Config::new();

    config.type_attribute(
        ".api.TimelineResponse",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]"
    );

    config.type_attribute(
        ".api.MonthTimeline",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]"
    );

    config.type_attribute(
        ".api.PhotosByMonthResponse",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]"
    );

    config.type_attribute(
        ".api.MonthMedia",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]"
    );

    config.type_attribute(
        ".api.MediaItem",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]"
    );

    config.compile_protos(&["proto/ratios.proto"], &["proto/"])?;

    Ok(())
}