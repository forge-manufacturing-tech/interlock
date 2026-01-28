#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::{prelude::DateTimeWithTimeZone, Condition, QuerySelect, JoinType, PaginatorTrait, RelationTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::{
    _entities::{projects::{ActiveModel, Entity, Model}, users_projects},
    users,
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProjectParams {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateProjectParams {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ShareProjectParams {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[schema(value_type = String, format = Date)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String, format = Date)]
    pub updated_at: DateTimeWithTimeZone,
    pub role: Option<String>, // 'owner' or 'member' - for now just 'member'
}

impl From<Model> for ProjectResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            created_at: m.created_at,
            updated_at: m.updated_at,
            role: None,
        }
    }
}

// Additional response for search
#[derive(Debug, Serialize, ToSchema)]
pub struct UserSearchResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[utoipa::path(
    get,
    path = "/api/projects",
    responses(
        (status = 200, description = "List all projects for current user", body = Vec<ProjectResponse>)
    )
)]
pub async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| Error::Unauthorized("User not found".into()))?;
    
    // Join users_projects to find projects for this user
    let projects = Entity::find()
        .join(JoinType::InnerJoin, crate::models::_entities::projects::Relation::UsersProjects.def())
        .filter(crate::models::_entities::users_projects::Column::UserId.eq(user.id))
        .all(&ctx.db)
        .await?;

    format::json(projects.into_iter().map(ProjectResponse::from).collect::<Vec<_>>())
}

#[utoipa::path(
    post,
    path = "/api/projects",
    request_body = CreateProjectParams,
    responses(
        (status = 200, description = "Project created", body = ProjectResponse)
    )
)]
pub async fn create(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<CreateProjectParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| Error::Unauthorized("User not found".into()))?;

    if user.role == "viewer" {
        return unauthorized("Viewers cannot create projects");
    }

    let txn = ctx.db.begin().await?;

    let project = ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(params.name),
        description: Set(params.description),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    // Add user as member (owner logic implies first member)
    let _relation = users_projects::ActiveModel {
        user_id: Set(user.id),
        project_id: Set(project.id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    format::json(ProjectResponse::from(project))
}

#[utoipa::path(
    get,
    path = "/api/projects/{id}",
    params(
        ("id" = Uuid, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Get project", body = ProjectResponse),
        (status = 404, description = "Project not found"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn get_one(Path(id): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    // Check access
    let project = Entity::find_by_id(id).one(&ctx.db).await?;
    let project = project.ok_or_else(|| Error::NotFound)?;

    let has_access = users_projects::Entity::find()
        .filter(users_projects::Column::UserId.eq(user.id))
        .filter(users_projects::Column::ProjectId.eq(id))
        .count(&ctx.db)
        .await? > 0;

    if !has_access {
        return unauthorized("User does not have access to this project");
    }

    format::json(ProjectResponse::from(project))
}

#[utoipa::path(
    put,
    path = "/api/projects/{id}",
    params(
        ("id" = Uuid, Path, description = "Project ID")
    ),
    request_body = UpdateProjectParams,
    responses(
        (status = 200, description = "Project updated", body = ProjectResponse),
        (status = 404, description = "Project not found")
    )
)]
pub async fn update(
    Path(id): Path<Uuid>,
    auth: auth::JWT, 
    State(ctx): State<AppContext>, 
    Json(params): Json<UpdateProjectParams>
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    // Check access
    let has_access = users_projects::Entity::find()
        .filter(users_projects::Column::UserId.eq(user.id))
        .filter(users_projects::Column::ProjectId.eq(id))
        .count(&ctx.db)
        .await? > 0;

    if !has_access {
        return unauthorized("User does not have access to this project");
    }

    let item = Entity::find_by_id(id).one(&ctx.db).await?.ok_or(Error::NotFound)?;
    let mut item = item.into_active_model();

    if let Some(name) = params.name {
        item.name = Set(name);
    }
    if let Some(desc) = params.description {
        item.description = Set(Some(desc));
    }

    let item = item.update(&ctx.db).await?;
    format::json(ProjectResponse::from(item))
}

#[utoipa::path(
    delete,
    path = "/api/projects/{id}",
    params(
        ("id" = Uuid, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project deleted"),
        (status = 404, description = "Project not found")
    )
)]
pub async fn remove(Path(id): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    // Check access (maybe only allow if user created it? For now any member can delete to keep it simple or check DB logic later)
    // Realistically should check if user is OWNER. But our Join table has no roles.
    // I'll assume any member can delete for this prototype, or stricter: check logic.
    // Let's just check access.
    let has_access = users_projects::Entity::find()
        .filter(users_projects::Column::UserId.eq(user.id))
        .filter(users_projects::Column::ProjectId.eq(id))
        .count(&ctx.db)
        .await? > 0;

    if !has_access {
        return unauthorized("User does not have access to this project");
    }

    Entity::delete_by_id(id).exec(&ctx.db).await?;
    format::empty()
}

#[utoipa::path(
    post,
    path = "/api/projects/{id}/share",
    params(
        ("id" = Uuid, Path, description = "Project ID")
    ),
    request_body = ShareProjectParams,
    responses(
        (status = 200, description = "Project shared"),
        (status = 404, description = "Project or User not found"),
        (status = 400, description = "User already in project")
    )
)]
pub async fn share(
    Path(id): Path<Uuid>,
    auth: auth::JWT, 
    State(ctx): State<AppContext>,
    Json(params): Json<ShareProjectParams>
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    // Check access
    let has_access = users_projects::Entity::find()
        .filter(users_projects::Column::UserId.eq(user.id))
        .filter(users_projects::Column::ProjectId.eq(id))
        .count(&ctx.db)
        .await? > 0;

    if !has_access {
        return unauthorized("User does not have access to this project");
    }

    // Find target user
    let target_user = users::Model::find_by_email(&ctx.db, &params.email).await
        .map_err(|_| Error::BadRequest("User not found".into()))?;

    // Check if already member
    let exists = users_projects::Entity::find()
        .filter(users_projects::Column::UserId.eq(target_user.id))
        .filter(users_projects::Column::ProjectId.eq(id))
        .count(&ctx.db)
        .await? > 0;

    if exists {
        return bad_request("User is already a member of this project");
    }

    users_projects::ActiveModel {
        user_id: Set(target_user.id),
        project_id: Set(id),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    format::empty()
}

#[utoipa::path(
    get,
    path = "/api/users/search",
    params(
        ("q" = String, Query, description = "Search query")
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<UserSearchResponse>)
    )
)]
pub async fn search_users(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<serde_json::Value>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let q = params.get("q").and_then(|v| v.as_str()).unwrap_or("");
    
    if q.len() < 2 {
        return format::json(Vec::<UserSearchResponse>::new());
    }

    let users = users::Entity::find()
        .filter(
            Condition::any()
                .add(users::users::Column::Name.contains(q))
                .add(users::users::Column::Email.contains(q))
        )
        .limit(10)
        .all(&ctx.db)
        .await?;

    let res: Vec<UserSearchResponse> = users.into_iter().map(|u| UserSearchResponse {
        id: u.id,
        name: u.name,
        email: u.email,
    }).collect();

    format::json(res)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/projects")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
        .add("/{id}/share", post(share))
        .add("/search_users", get(search_users)) // Route nesting might be tricky, putting it here or under /api/users
        // Move search_users to users controller? Or keep here but change path.
        // Keeping here but mapped to internal logic is fine.
}
