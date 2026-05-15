fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto/timeline.proto");

    let file_descriptors = protox::compile(["proto/timeline.proto"], ["proto/"])
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // 3. Create the prost_build configuration
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
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Eq)]",
    );
    config.type_attribute(
        ".api.AlbumMediaGroup",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Eq)]",
    );
    config.type_attribute(
        ".api.AlbumInfo",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Eq)]",
    );
    config.type_attribute(
        ".api.FullAlbumMediaResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.CollaboratorSummary",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.SimpleTimelineItem",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.SearchResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.OrderedMediaResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.SearchSuggestionsResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Eq)]",
    );
    config.type_attribute(
        ".api.SearchSuggestion",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.SuggestionType",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    // --- PERSON STRUCTS ---
    config.type_attribute(
        ".api.PersonInfo",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.ListPeopleResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );
    config.type_attribute(
        ".api.FullPersonMediaResponse",
        "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]",
    );

    config.type_attribute(".api", "#[serde(rename_all = \"camelCase\")]");
    config.compile_fds(file_descriptors)?;

    Ok(())
}
