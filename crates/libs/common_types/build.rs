fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto/timeline.proto");

    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");

    // --- TIMELINE STRUCTS ---
    config.type_attribute(
        ".api.TimelineRatiosResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.TimelineMonthRatios",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]",
    );

    config.type_attribute(
        ".api.TimelineItemsResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Eq)]",
    );

    config.type_attribute(
        ".api.TimelineMonthItems",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Eq)]",
    );

    config.type_attribute(
        ".api.TimelineItem",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]",
    );

    // --- ALBUM STRUCTS ---
    config.type_attribute(
        ".api.AlbumRatiosResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.AlbumRatioGroup",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.AlbumMediaResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.AlbumMediaGroup",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.AlbumInfo",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.compile_protos(&["proto/timeline.proto"], &["proto/"])?;

    Ok(())
}
