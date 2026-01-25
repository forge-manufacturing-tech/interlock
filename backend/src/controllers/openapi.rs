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
    ),
    components(
        schemas(
            models::users::LoginParams,
            models::users::RegisterParams,
            views::auth::LoginResponse,
            controllers::sessions::Params,
            controllers::sessions::SessionResponse,
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
