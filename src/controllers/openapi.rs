use loco_rs::prelude::*;
use utoipa::OpenApi;
use axum::response::Html;
use crate::{controllers, models, views};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::auth::register,
        controllers::auth::login,
        controllers::auth::initialized,
        controllers::admin::list_users,
        controllers::admin::get_user,
        controllers::admin::promote,
        controllers::admin::demote,
        controllers::admin::reset_password,
        controllers::sessions::list,
        controllers::sessions::add,
        controllers::sessions::update,
        controllers::sessions::remove,
        controllers::sessions::get_one,
        controllers::projects::list,
        controllers::projects::create,
        controllers::projects::get_one,
        controllers::projects::update,
        controllers::projects::remove,
        controllers::projects::share,
        controllers::projects::search_users,
        controllers::blobs::upload,
        controllers::blobs::list,
        controllers::blobs::download,
        controllers::blobs::remove,
        controllers::chat::chat,
        controllers::chat::list_messages,
        controllers::chat::clear_messages,
    ),
    components(
        schemas(
            models::users::LoginParams,
            models::users::RegisterParams,
            views::auth::LoginResponse,
            controllers::auth::InitResponse,
            controllers::admin::ResetPasswordParams,
            controllers::admin::UserResponse,
            controllers::sessions::Params,
            controllers::sessions::SessionResponse,
            controllers::projects::CreateProjectParams,
            controllers::projects::UpdateProjectParams,
            controllers::projects::ShareProjectParams,
            controllers::projects::ProjectResponse,
            controllers::projects::UserSearchResponse,
            controllers::blobs::BlobResponse,
            controllers::chat::ChatParams,
            controllers::chat::MessageResponse,
        )
    ),
)]
pub struct ApiDoc;

pub async fn get_openapi() -> Result<Json<serde_json::Value>> {
    let openapi = ApiDoc::openapi();
    // Serialize to JSON Value using serde_json
    let json_value = serde_json::to_value(&openapi)
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(json_value))
}

pub async fn serve_swagger_ui() -> Result<Html<String>> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="description" content="SwaggerUI" />
  <title>SwaggerUI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js" crossorigin></script>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-standalone-preset.js" crossorigin></script>
<script>
  window.onload = () => {
    window.ui = SwaggerUIBundle({
      url: '/api-docs/openapi.json',
      dom_id: '#swagger-ui',
      presets: [
        SwaggerUIBundle.presets.apis,
        SwaggerUIStandalonePreset
      ],
      layout: "StandaloneLayout",
    });
  };
</script>
</body>
</html>"#;
    Ok(Html(html.to_string()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api-docs")
        .add("/openapi.json", get(get_openapi))
        .add("/ui", get(serve_swagger_ui))
}
