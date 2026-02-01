use loco_rs::prelude::*;
use utoipa::openapi::security::{
    ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme,
};
use utoipa::{Modify, OpenApi};
use axum::response::Html;
use crate::{controllers, models, views};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::auth::register,
        controllers::auth::login,
        controllers::auth::initialized,
        controllers::auth::current,
        controllers::auth::regenerate_api_key,
        controllers::admin::list_users,
        controllers::admin::get_user,
        controllers::admin::promote,
        controllers::admin::demote,
        controllers::admin::reset_password,
        controllers::admin::delete_user,
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
            views::auth::CurrentResponse,
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
    modifiers(&SecurityAddon),
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub async fn get_openapi() -> Result<Json<serde_json::Value>> {
    let openapi = ApiDoc::openapi();
    // Serialize to JSON Value using serde_json
    let json_value = serde_json::to_value(&openapi)
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(json_value))
}

pub async fn serve_swagger_ui() -> Result<Html<String>> {
    let html = r#"<!doctype html>
<html>
  <head>
    <title>API Reference</title>
    <meta charset="utf-8" />
    <meta
      name="viewport"
      content="width=device-width, initial-scale=1" />
  </head>
  <body>
    <script
      id="api-reference"
      data-url="/api-docs/openapi.json"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
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
