use loco_rs::prelude::*;
use sea_orm::{ActiveValue, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use axum::{
    extract::{Path, State},
};
use crate::models::{
    _entities::{messages::{ActiveModel, Entity, Model}, sessions},
};
use crate::agent::run_agent_cycle;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatParams {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

impl From<Model> for MessageResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            session_id: m.session_id,
            role: m.role,
            content: m.content,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

async fn check_session_access(ctx: &AppContext, _auth: &auth::JWT, session_id: Uuid) -> Result<sessions::Model> {
    let session = sessions::Entity::find_by_id(session_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    Ok(session)
}

#[utoipa::path(
    post,
    path = "/api/sessions/{session_id}/chat",
    params(
        ("session_id" = Uuid, Path, description = "Session ID")
    ),
    request_body = ChatParams,
    responses(
        (status = 200, description = "Chat response", body = MessageResponse),
        (status = 404, description = "Session not found")
    )
)]
pub async fn chat(
    Path(session_id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<ChatParams>,
) -> Result<Response> {
    let _session = check_session_access(&ctx, &auth, session_id).await?;
    
    // 1. Save user message
    let user_msg = ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        session_id: ActiveValue::Set(session_id),
        role: ActiveValue::Set("user".to_string()),
        content: ActiveValue::Set(params.message.clone()),
        ..Default::default()
    };
    user_msg.insert(&ctx.db).await?;

    // 2. Fetch available blobs for context
    let blobs = crate::models::_entities::blobs::Entity::find()
        .filter(crate::models::_entities::blobs::Column::SessionId.eq(session_id))
        .all(&ctx.db)
        .await?;
    
    let blobs_context: Vec<(String, String)> = blobs.into_iter()
        .map(|b| (b.id.to_string(), b.file_name))
        .collect();

    // 3. Run agent
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| Error::BadRequest("GEMINI_API_KEY not set".into()))?;
    
    let agent_response = run_agent_cycle(&ctx, session_id, &params.message, &api_key, blobs_context).await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    // 3. Save agent response
    let assistant_msg = ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        session_id: ActiveValue::Set(session_id),
        role: ActiveValue::Set("assistant".to_string()),
        content: ActiveValue::Set(agent_response.clone()),
        ..Default::default()
    };
    let assistant_msg = assistant_msg.insert(&ctx.db).await?;

    format::json(MessageResponse::from(assistant_msg))
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/messages",
    params(
        ("session_id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "List messages", body = Vec<MessageResponse>)
    )
)]
pub async fn list_messages(
    Path(session_id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let _session = check_session_access(&ctx, &auth, session_id).await?;
    
    let messages = Entity::find()
        .filter(crate::models::_entities::messages::Column::SessionId.eq(session_id))
        .all(&ctx.db)
        .await?;
        
    format::json(messages.into_iter().map(MessageResponse::from).collect::<Vec<_>>())
}

#[utoipa::path(
    delete,
    path = "/api/sessions/{session_id}/messages",
    params(
        ("session_id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Messages cleared")
    )
)]
pub async fn clear_messages(
    Path(session_id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let _session = check_session_access(&ctx, &auth, session_id).await?;
    
    crate::models::_entities::messages::Entity::delete_many()
        .filter(crate::models::_entities::messages::Column::SessionId.eq(session_id))
        .exec(&ctx.db)
        .await?;
        
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/api/sessions/{session_id}/chat", post(chat))
        .add("/api/sessions/{session_id}/messages", get(list_messages))
        .add("/api/sessions/{session_id}/messages", delete(clear_messages))
}
