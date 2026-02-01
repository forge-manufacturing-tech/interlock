use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use loco_rs::{app::AppContext, prelude::*};
use uuid::Uuid;
use crate::models::users;
use loco_rs::auth::jwt::JWT;

pub struct ApiAuth {
    pub pid: Uuid,
}

impl<S> FromRequestParts<S> for ApiAuth
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = loco_rs::Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = AppContext::from_ref(state);

        // Try JWT
        if let Some(auth_header) = parts.headers.get("Authorization") {
             if let Ok(auth_str) = auth_header.to_str() {
                 if auth_str.starts_with("Bearer ") {
                     let token = &auth_str[7..];
                     let jwt_config = ctx.config.get_jwt_config().map_err(|e| {
                         tracing::error!("failed to get jwt config: {}", e);
                         loco_rs::Error::InternalServerError
                     })?;

                     let jwt_validator = JWT::new(&jwt_config.secret);
                     // validate returns Result<TokenData<UserClaims>, ...>
                     if let Ok(token_data) = jwt_validator.validate(token) {
                          let pid = Uuid::parse_str(&token_data.claims.pid).map_err(|e| {
                                tracing::error!("invalid pid in jwt: {}", e);
                                loco_rs::Error::Unauthorized("invalid token".to_string())
                          })?;
                          return Ok(Self { pid });
                     }
                 }
             }
        }

        // Check X-API-Key
        if let Some(api_key) = parts.headers.get("X-API-Key") {
             let api_key_str = api_key.to_str().map_err(|_| loco_rs::Error::Unauthorized("invalid api key".to_string()))?;

             // Validate against DB
             let user = users::Model::find_by_api_key(&ctx.db, api_key_str).await.map_err(|e| {
                 tracing::error!("api key lookup failed: {}", e);
                 loco_rs::Error::Unauthorized("invalid api key".to_string())
             })?;

             return Ok(Self { pid: user.pid });
        }

        loco_rs::controller::unauthorized("unauthorized")
    }
}
