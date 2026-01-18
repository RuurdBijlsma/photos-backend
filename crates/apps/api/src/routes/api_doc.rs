use crate::routes::{album, auth, onboarding, photos, root, s2s, timeline};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        root::handlers::root,
        root::handlers::health_check,
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
        onboarding::handlers::post_start_processing,
        // Photos handlers
        photos::handlers::get_random_photo,
        photos::handlers::get_full_item_handler,
        photos::handlers::get_color_theme_handler,
        photos::handlers::download_full_file,
        // Album handlers
        album::handlers::create_album_handler,
        album::handlers::get_user_albums_handler,
        album::handlers::update_album_handler,
        album::handlers::add_media_to_album_handler,
        album::handlers::remove_media_from_album_handler,
        album::handlers::add_collaborator_handler,
        album::handlers::remove_collaborator_handler,
        album::handlers::generate_invite_handler,
        album::handlers::check_invite_handler,
        album::handlers::accept_invite_handler,
        album::handlers::get_album_media_handler,
        // Timeline handlers
        timeline::handlers::get_timeline_ratios_handler,
        timeline::handlers::get_timeline_ids_handler,
        timeline::handlers::get_photos_by_month_handler,
        timeline::handlers::timeline_websocket_handler,
        // S2S handlers
        s2s::handlers::invite_summary_handler,
        s2s::handlers::download_file_handler,
    ),
    components(
        schemas(
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Ruurd Photos", description = "Ruurd Photos' API"),
        (name = "Photos", description = "Endpoints for browsing and managing media items"),
        (name = "Album", description = "Endpoints for managing photo albums and collaboration"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Onboarding", description = "Onboarding and setup endpoints"),
        (name = "S2S", description = "Server-to-server communication endpoints"),
        (name = "Timeline", description = "Timeline and media retrieval endpoints"),
        (name = "System", description = "Health check"),
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
