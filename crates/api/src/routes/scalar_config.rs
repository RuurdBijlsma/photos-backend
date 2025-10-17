//! This module provides a function to generate custom HTML for the `OpenAPI` documentation.

use serde_json::to_string_pretty;
use utoipa::openapi::OpenApi;

/// Generates a custom HTML page for the Scalar API reference UI.
///
/// # Panics
///
/// This function will panic if the provided `OpenApi` struct cannot be serialized to a JSON string.
#[must_use]
pub fn get_custom_html(open_api: &OpenApi) -> String {
    let json = to_string_pretty(open_api).expect("failed to serialize OpenApi to JSON");

    format!(
        r#"
<!doctype html>
<html>
  <head>
    <title>Scalar API Reference</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  </head>
  <body>
    <div id="app"></div>

    <!-- Load the Script -->
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>

    <!-- Initialize the Scalar API Reference -->
    <script>
      Scalar.createApiReference('#app', {{
        "content": {json},
        "layout": "classic",
        "theme": "purple",
        "showSidebar": true,
        "hideModels": false,
        "withDefaultFonts": true
      }})
    </script>
  </body>
</html>
"#,
    )
}
