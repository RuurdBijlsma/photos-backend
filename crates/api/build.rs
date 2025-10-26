fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto/photos.proto");

    let mut config = prost_build::Config::new();

    config.type_attribute(
        ".api.TimelineResponse",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.TimelineMonth",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.ByMonthResponse",
        "#[derive(Eq, serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.MediaMonth",
        "#[derive(Eq, serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.MediaItem",
        "#[derive(Eq, serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]",
    );

    config.compile_protos(&["proto/photos.proto"], &["proto/"])?;

    Ok(())
}
