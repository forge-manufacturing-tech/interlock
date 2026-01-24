#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use crate::models::_entities::sessions::{ActiveModel, Entity, Model};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        if let Some(title) = &self.title {
            item.title = Set(Some(title.clone()));
        }
        if let Some(content) = &self.content {
            item.content = Set(Some(content.clone()));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    #[serde(with = "crate::models::i64_format")]
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl From<Model> for SessionResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            content: m.content,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let items = Entity::find().all(&ctx.db).await?;
    format::json(items.into_iter().map(SessionResponse::from).collect::<Vec<_>>())
}

pub async fn add(State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
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

pub async fn update(
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(SessionResponse::from(item))
}

pub async fn remove(Path(id): Path<i64>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub async fn get_one(Path(id): Path<i64>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(SessionResponse::from(load_item(&ctx, id).await?))
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

pub async fn load_item(ctx: &AppContext, id: i64) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| {
        tracing::error!("Session not found with ID: {}", id);
        Error::NotFound
    })
}
