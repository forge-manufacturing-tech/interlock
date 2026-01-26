use loco_rs::prelude::*;
use utoipa::OpenApi;
use crate::{controllers, models, views};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::auth::register,
        controllers::auth::login,
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
    ),
    components(
        schemas(
            models::users::LoginParams,
            models::users::RegisterParams,
            views::auth::LoginResponse,
            controllers::sessions::Params,
            controllers::sessions::SessionResponse,
            controllers::projects::CreateProjectParams,
            controllers::projects::UpdateProjectParams,
            controllers::projects::ShareProjectParams,
            controllers::projects::ProjectResponse,
            controllers::projects::UserSearchResponse,
            controllers::blobs::BlobResponse,
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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api-docs")
        .add("/openapi.json", get(get_openapi))
}
