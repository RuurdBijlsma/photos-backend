fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto/photos.proto");

    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");

    config.type_attribute(
        ".api.TimelineResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.TimelineMonth",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.ByMonthResponse",
        "#[derive(Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.MediaMonth",
        "#[derive(Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.type_attribute(
        ".api.MediaItem",
        "#[derive(Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.compile_protos(&["proto/photos.proto"], &["proto/"])?;

    Ok(())
}
