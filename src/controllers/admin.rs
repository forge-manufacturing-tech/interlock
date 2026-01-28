use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::{
    _entities::{users, groups, users_groups},
};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResetPasswordParams {
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub pid: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub created_at: String,
}

impl From<users::Model> for UserResponse {
    fn from(m: users::Model) -> Self {
        Self {
            id: m.id,
            pid: m.pid,
            email: m.email,
            name: m.name,
            role: m.role,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GroupParams {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GroupResponse {
    pub id: i64,
    pub pid: Uuid,
    pub name: String,
}

impl From<groups::Model> for GroupResponse {
    fn from(m: groups::Model) -> Self {
        Self {
            id: m.id,
            pid: m.pid,
            name: m.name,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddUserToGroupParams {
    pub user_id: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserParams {
    pub role: Option<String>,
}

fn ensure_admin(user: &users::Model) -> Result<()> {
    if user.role != "admin" {
        return unauthorized("Access denied: Admins only");
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "List users", body = Vec<UserResponse>),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn list_users(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&user)?;

    let users = users::Entity::find()
        .order_by_asc(users::Column::Id)
        .all(&ctx.db)
        .await?;

    let res: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    format::json(res)
}

#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User PID")
    ),
    responses(
        (status = 200, description = "Get user", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn get_user(Path(pid): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    let user = crate::models::users::Model::find_by_pid(&ctx.db, &pid.to_string()).await?;
    format::json(UserResponse::from(user))
}

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/promote",
    params(
        ("id" = Uuid, Path, description = "User PID")
    ),
    responses(
        (status = 200, description = "User promoted"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn promote(Path(pid): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    let user = crate::models::users::Model::find_by_pid(&ctx.db, &pid.to_string()).await?;
    let mut active = user.into_active_model();
    active.role = ActiveValue::Set("admin".to_string());
    active.update(&ctx.db).await?;

    format::empty()
}

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/demote",
    params(
        ("id" = Uuid, Path, description = "User PID")
    ),
    responses(
        (status = 200, description = "User demoted"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn demote(Path(pid): Path<Uuid>, auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    if current_user.pid == pid {
        return bad_request("Cannot demote yourself");
    }

    let user = crate::models::users::Model::find_by_pid(&ctx.db, &pid.to_string()).await?;
    let mut active = user.into_active_model();
    active.role = ActiveValue::Set("user".to_string());
    active.update(&ctx.db).await?;

    format::empty()
}

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/reset_password",
    params(
        ("id" = Uuid, Path, description = "User PID")
    ),
    request_body = ResetPasswordParams,
    responses(
        (status = 200, description = "Password reset"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn reset_password(
    Path(pid): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<ResetPasswordParams>
) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    let user = crate::models::users::Model::find_by_pid(&ctx.db, &pid.to_string()).await?;
    let active = user.into_active_model();
    active.reset_password(&ctx.db, &params.password).await?;

    format::empty()
}

#[utoipa::path(
    post,
    path = "/api/admin/groups",
    request_body = GroupParams,
    responses(
        (status = 200, description = "Group created", body = GroupResponse),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn create_group(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<GroupParams>
) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    let group = groups::ActiveModel {
        name: Set(params.name),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    format::json(GroupResponse::from(group))
}

#[utoipa::path(
    post,
    path = "/api/admin/groups/{id}/users",
    params(
        ("id" = i64, Path, description = "Group ID")
    ),
    request_body = AddUserToGroupParams,
    responses(
        (status = 200, description = "User added to group"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn add_user_to_group(
    Path(group_id): Path<i64>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<AddUserToGroupParams>
) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    users_groups::ActiveModel {
        user_id: Set(params.user_id),
        group_id: Set(group_id),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    format::empty()
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = UpdateUserParams,
    responses(
        (status = 200, description = "User updated"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Unauthorized")
    )
)]
pub async fn update_user(
    Path(id): Path<i64>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateUserParams>
) -> Result<Response> {
    let current_user = crate::models::users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    ensure_admin(&current_user)?;

    let user = users::Entity::find_by_id(id).one(&ctx.db).await?.ok_or(Error::NotFound)?;
    let mut active = user.into_active_model();
    if let Some(role) = params.role {
        active.role = Set(role);
    }
    active.update(&ctx.db).await?;

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/admin")
        .add("/users", get(list_users))
        .add("/users/{id}", get(get_user))
        .add("/users/{id}", put(update_user))
        .add("/users/{id}/promote", post(promote))
        .add("/users/{id}/demote", post(demote))
        .add("/users/{id}/reset_password", post(reset_password))
        .add("/groups", post(create_group))
        .add("/groups/{id}/users", post(add_user_to_group))
}
