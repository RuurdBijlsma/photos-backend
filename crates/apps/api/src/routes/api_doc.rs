use crate::routes::{album, auth, download, onboarding, photos, root};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

// todo: fix this
#[derive(OpenApi)]
#[openapi(
    paths(
        root::handlers::root,
        // Auth handlers
        auth::handlers::login,
        auth::handlers::register,
        auth::handlers::refresh_session,
        auth::handlers::logout,
        auth::handlers::get_me,
        // Onboarding handlers
        onboarding::handlers::get_disk_response,
        onboarding::handlers::get_folder_media_sample,
        onboarding::handlers::get_folder_unsupported,
        onboarding::handlers::get_folders,
        onboarding::handlers::make_folder,
        // Download handlers
        download::handlers::download_full_file,
        // Photos handlers
        photos::handlers::get_random_photo,
        // Album handlers
        album::handlers::create_album_handler,
        album::handlers::get_user_albums_handler,
        album::handlers::get_album_details_handler,
        album::handlers::update_album_handler,
        album::handlers::add_media_to_album_handler,
        album::handlers::remove_media_from_album_handler,
        album::handlers::add_collaborator_handler,
        album::handlers::remove_collaborator_handler,
    ),
    components(
        schemas(
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Ruurd Photos", description = "Ruurd Photos' API"),
        (name = "Photos", description = "Endpoints for browsing and managing media items"),
        (name = "Albums", description = "Endpoints for managing photo albums and collaboration")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
