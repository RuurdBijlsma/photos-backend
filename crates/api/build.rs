fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto/ratios.proto");

    let mut config = prost_build::Config::new();

    config.type_attribute(
        ".api.MediaItem",
        "#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]"
    );

    config.type_attribute(
        ".api.MonthGroup",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]"
    );

    config.type_attribute(
        ".api.MultiMonthGroup",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]"
    );

    config.compile_protos(&["proto/ratios.proto"], &["proto/"])?;

    Ok(())
}