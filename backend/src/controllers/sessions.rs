#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::{prelude::DateTimeWithTimeZone, QuerySelect, JoinType, PaginatorTrait, RelationTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::{ToSchema, IntoParams};
use crate::models::{
    _entities::{sessions::{ActiveModel, Entity, Model}, users_projects},
    users,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Params {
    pub title: Option<String>,
    pub content: Option<String>,
    pub project_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
pub struct ListParams {
    pub project_id: Option<Uuid>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        if let Some(title) = &self.title {
            item.title = Set(Some(title.clone()));
        }
        if let Some(content) = &self.content {
            item.content = Set(Some(content.clone()));
        }
        if let Some(project_id) = &self.project_id {
            item.project_id = Set(Some(*project_id));
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub project_id: Option<Uuid>,
    #[schema(value_type = String, format = Date)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String, format = Date)]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<Model> for SessionResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            content: m.content,
            project_id: m.project_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/sessions",
    params(ListParams),
    responses(
        (status = 200, description = "List sessions", body = Vec<SessionResponse>)
    )
)]
pub async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| Error::Unauthorized("User not found".into()))?;
    
    let mut query = Entity::find();

    if let Some(project_id) = params.project_id {
        // Check access to this project
        let has_access = users_projects::Entity::find()
            .filter(users_projects::Column::UserId.eq(user.id))
            .filter(users_projects::Column::ProjectId.eq(project_id))
            .count(&ctx.db)
            .await? > 0;
            
        if !has_access {
            return unauthorized("User does not have access to this project");
        }
        query = query.filter(crate::models::_entities::sessions::Column::ProjectId.eq(project_id));
    } else {
         query = query
            .join(JoinType::InnerJoin, crate::models::_entities::sessions::Relation::Project.def())
            .join(JoinType::InnerJoin, crate::models::_entities::projects::Relation::UsersProjects.def())
            .filter(crate::models::_entities::users_projects::Column::UserId.eq(user.id));
    }
    
    let items = query.all(&ctx.db).await?;
    format::json(items.into_iter().map(SessionResponse::from).collect::<Vec<_>>())
}

#[utoipa::path(
    post,
    path = "/api/sessions",
    request_body = Params,
    responses(
        (status = 200, description = "Session created", body = SessionResponse)
    )
)]
pub async fn add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| Error::Unauthorized("User not found".into()))?;
    
    if let Some(project_id) = params.project_id {
        let has_access = users_projects::Entity::find()
            .filter(users_projects::Column::UserId.eq(user.id))
            .filter(users_projects::Column::ProjectId.eq(project_id))
            .count(&ctx.db)
            .await? > 0;
        if !has_access {
             return unauthorized("User does not have access to this project");
        }
    } else {
         return bad_request("project_id is required");
    }

    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);

    let txn = ctx.db.begin().await?;
    let item = item.insert(&txn).await.map_err(|e| {
        tracing::error!("Failed to insert session: {:?}", e);
        e
    })?;
    txn.commit().await?;

    format::json(SessionResponse::from(item))
}

#[utoipa::path(
    put,
    path = "/api/sessions/{id}",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    request_body = Params,
    responses(
        (status = 200, description = "Session updated", body = SessionResponse),
        (status = 404, description = "Session not found")
    )
)]
pub async fn update(
    Path(id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    let item = load_item(&ctx, id).await?;
    
    // Check access via project_id
    if let Some(project_id) = item.project_id {
          let has_access = users_projects::Entity::find()
            .filter(users_projects::Column::UserId.eq(user.id))
            .filter(users_projects::Column::ProjectId.eq(project_id))
            .count(&ctx.db)
            .await? > 0;
        if !has_access {
             return unauthorized("Authorized access required");
        }
    }

    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(SessionResponse::from(item))
}

#[utoipa::path(
    delete,
    path = "/api/sessions/{id}",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session deleted"),
        (status = 404, description = "Session not found")
    )
)]
pub async fn remove(Path(id): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    
     if let Some(project_id) = item.project_id {
          let has_access = users_projects::Entity::find()
            .filter(users_projects::Column::UserId.eq(user.id))
            .filter(users_projects::Column::ProjectId.eq(project_id))
            .count(&ctx.db)
            .await? > 0;
        if !has_access {
             return unauthorized("Authorized access required");
        }
    }
    
    // Delete associated blobs from object store
    let blobs = crate::models::_entities::blobs::Entity::find()
        .filter(crate::models::_entities::blobs::Column::SessionId.eq(id))
        .all(&ctx.db)
        .await?;
    
    let storage = crate::storage::get_storage();
    for blob in blobs {
        if let Err(e) = storage.delete(&object_store::path::Path::from(blob.storage_key.clone())).await {
            tracing::error!("Failed to delete blob {} from storage: {:?}", blob.id, e);
        }
    }

    item.delete(&ctx.db).await?;
    format::empty()
}

#[utoipa::path(
    get,
    path = "/api/sessions/{id}",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Get session", body = SessionResponse),
        (status = 404, description = "Session not found")
    )
)]
pub async fn get_one(Path(id): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;

    if let Some(project_id) = item.project_id {
          let has_access = users_projects::Entity::find()
            .filter(users_projects::Column::UserId.eq(user.id))
            .filter(users_projects::Column::ProjectId.eq(project_id))
            .count(&ctx.db)
            .await? > 0;
        if !has_access {
             return unauthorized("Authorized access required");
        }
    }
    
    format::json(SessionResponse::from(item))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/sessions")
        .add("/", get(list))
        .add("/", post(add))
        .add("/{id}", get(get_one))
        .add("/{id}", delete(remove))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
}

pub async fn load_item(ctx: &AppContext, id: Uuid) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| {
        tracing::error!("Session not found with ID: {}", id);
        Error::NotFound
    })
}
