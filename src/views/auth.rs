use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::users;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub pid: String,
    pub name: String,
    pub is_verified: bool,
    pub role: String,
}

impl LoginResponse {
    #[must_use]
    pub fn new(user: &users::Model, token: &String) -> Self {
        Self {
            token: token.to_string(),
            pid: user.pid.to_string(),
            name: user.name.clone(),
            is_verified: user.email_verified_at.is_some(),
            role: user.role.clone(),
        }
    }
}


#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CurrentResponse {
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub api_key: String,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: &users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            api_key: user.api_key.clone(),
        }
    }
}
