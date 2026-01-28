use loco_rs::prelude::*;
use sea_orm::{prelude::DateTimeWithTimeZone, ActiveValue, EntityTrait, QueryFilter, ColumnTrait};
use serde::Serialize;
use uuid::Uuid;
use utoipa::ToSchema;
use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
    body::Body,
};
use axum::http::{header, StatusCode};
use crate::models::{
    _entities::{blobs::{ActiveModel, Entity, Model}, sessions},
};
use crate::storage::get_storage;
use object_store::path::Path as ObjectPath;

#[derive(Debug, Serialize, ToSchema)]
pub struct BlobResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeWithTimeZone,
}

impl From<Model> for BlobResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            session_id: m.session_id,
            file_name: m.file_name,
            content_type: m.content_type,
            size: m.size,
            created_at: m.created_at,
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
    path = "/api/sessions/{session_id}/blobs",
    params(
        ("session_id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "File uploaded", body = BlobResponse),
        (status = 404, description = "Session not found")
    )
)]
pub async fn upload(
    Path(session_id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let _session = check_session_access(&ctx, &auth, session_id).await?;

    if let Some(field) = multipart.next_field().await.map_err(|e| Error::BadRequest(e.to_string()))? {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        let data = field.bytes().await.map_err(|e| Error::BadRequest(e.to_string()))?;
        let size = data.len() as i64;
        
        let blob_id = Uuid::new_v4();
        let storage_key = format!("{}/{}", session_id, blob_id);
        let storage = get_storage();
        
        storage.put(&ObjectPath::from(storage_key.clone()), data.into())
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        let blob = ActiveModel {
            id: ActiveValue::Set(blob_id),
            session_id: ActiveValue::Set(session_id),
            file_name: ActiveValue::Set(file_name),
            content_type: ActiveValue::Set(content_type),
            size: ActiveValue::Set(size),
            storage_key: ActiveValue::Set(storage_key),
            ..Default::default()
        };

        let blob = blob.insert(&ctx.db).await?;
        return format::json(BlobResponse::from(blob));
    }

    bad_request("No file provided")
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/blobs",
    params(
        ("session_id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "List blobs", body = Vec<BlobResponse>)
    )
)]
pub async fn list(
    Path(session_id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let _session = check_session_access(&ctx, &auth, session_id).await?;
    
    let blobs = Entity::find()
        .filter(crate::models::_entities::blobs::Column::SessionId.eq(session_id))
        .all(&ctx.db)
        .await?;
        
    format::json(blobs.into_iter().map(BlobResponse::from).collect::<Vec<_>>())
}

#[utoipa::path(
    get,
    path = "/api/blobs/{id}/download",
    params(
        ("id" = Uuid, Path, description = "Blob ID")
    ),
    responses(
        (status = 200, description = "Download blob"),
        (status = 404, description = "Blob not found")
    )
)]
pub async fn download(
    Path(id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let blob = Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let _session = check_session_access(&ctx, &auth, blob.session_id).await?;

    let storage = get_storage();
    let result = storage.get(&ObjectPath::from(blob.storage_key))
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    let stream = result.into_stream();
    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, blob.content_type),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", blob.file_name)),
            (header::CONTENT_LENGTH, blob.size.to_string()),
        ],
        body,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/blobs/{id}",
    params(
        ("id" = Uuid, Path, description = "Blob ID")
    ),
    responses(
        (status = 200, description = "Blob deleted"),
        (status = 404, description = "Blob not found")
    )
)]
pub async fn remove(
    Path(id): Path<Uuid>,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let blob = Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let _session = check_session_access(&ctx, &auth, blob.session_id).await?;

    let storage = get_storage();
    storage.delete(&ObjectPath::from(blob.storage_key.clone()))
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    blob.delete(&ctx.db).await?;
    
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/api/sessions/{session_id}/blobs", post(upload))
        .add("/api/sessions/{session_id}/blobs", get(list))
        .add("/api/blobs/{id}/download", get(download))
        .add("/api/blobs/{id}", delete(remove))
}
