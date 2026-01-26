use calamine::{Reader, Xlsx, open_workbook_from_rs, Data};
use rust_xlsxwriter::{Workbook, Format};
use uuid::Uuid;
use std::io::{Cursor, Read};
use crate::storage::get_storage;
use object_store::path::Path as ObjectPath;
use loco_rs::prelude::*;
use crate::models::_entities::{blobs};

pub async fn list_files(session_id: Uuid, ctx: &AppContext) -> anyhow::Result<String> {
    let files = blobs::Entity::find()
        .filter(blobs::Column::SessionId.eq(session_id))
        .all(&ctx.db)
        .await?;
    
    let mut res = String::from("Files in this session:\n");
    for file in files {
        res.push_str(&format!("- {} (ID: {})\n", file.file_name, file.id));
    }
    Ok(res)
}

pub async fn get_excel_sheets(blob_id: Uuid, ctx: &AppContext) -> anyhow::Result<Vec<String>> {
    let blob = blobs::Entity::find_by_id(blob_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Blob not found"))?;

    let storage = get_storage();
    let data = storage.get(&ObjectPath::from(blob.storage_key))
        .await?
        .bytes()
        .await?;

    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
    
    Ok(workbook.sheet_names().to_vec())
}

pub async fn excel_to_csv(
    blob_id: Uuid, 
    sheet_name: Option<String>, 
    ctx: &AppContext
) -> anyhow::Result<String> {
    let blob = blobs::Entity::find_by_id(blob_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Blob not found"))?;

    let storage = get_storage();
    let data = storage.get(&ObjectPath::from(blob.storage_key))
        .await?
        .bytes()
        .await?;

    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
    
    let sheet = if let Some(name) = sheet_name {
        name
    } else {
        workbook.sheet_names().first()
            .ok_or_else(|| anyhow::anyhow!("No sheets found in Excel file"))?
            .clone()
    };
    
    let range = workbook.worksheet_range(&sheet)?;

    let mut csv = String::new();
    for row in range.rows() {
        let row_str: Vec<String> = row.iter().map(|cell| match cell {
            Data::String(s) => s.clone(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            _ => "".to_string(),
        }).collect();
        csv.push_str(&row_str.join(","));
        csv.push('\n');
    }

    Ok(csv)
}

pub async fn create_excel(
    file_name: &str,
    rows: Vec<Vec<String>>,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join("output.xlsx");
    let file_path_str = file_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    
    let mut workbook = Workbook::new(file_path_str);
    let worksheet = workbook.add_worksheet();
    
    let format = Format::default();
    for (r, row) in rows.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            worksheet.write_string(r as u32, c as u16, val, &format)?;
        }
    }
    
    workbook.close()?;
    
    let mut file = std::fs::File::open(&file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    
    let blob_id = Uuid::new_v4();
    let storage_key = format!("{}/{}", session_id, blob_id);
    
    let storage = get_storage();
    storage.put(&ObjectPath::from(storage_key.clone()), buf.clone().into()).await?;

    let blob = blobs::ActiveModel {
        id: ActiveValue::Set(blob_id),
        session_id: ActiveValue::Set(session_id),
        file_name: ActiveValue::Set(file_name.to_string()),
        content_type: ActiveValue::Set("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
        size: ActiveValue::Set(buf.len() as i64),
        storage_key: ActiveValue::Set(storage_key),
        ..Default::default()
    };

    blob.insert(&ctx.db).await?;

    Ok(blob_id)
}
pub async fn create_text_file(
    file_name: &str,
    content: &str,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    let blob_id = Uuid::new_v4();
    let storage_key = format!("{}/{}", session_id, blob_id);
    
    let storage = get_storage();
    storage.put(&ObjectPath::from(storage_key.clone()), content.as_bytes().to_vec().into()).await?;

    let blob = blobs::ActiveModel {
        id: ActiveValue::Set(blob_id),
        session_id: ActiveValue::Set(session_id),
        file_name: ActiveValue::Set(file_name.to_string()),
        content_type: ActiveValue::Set("text/plain".to_string()),
        size: ActiveValue::Set(content.len() as i64),
        storage_key: ActiveValue::Set(storage_key),
        ..Default::default()
    };

    blob.insert(&ctx.db).await?;

    Ok(blob_id)
}

pub async fn download_from_url(
    url: &str,
    file_name: &str,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let content_type = resp.headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    
    let bytes = resp.bytes().await?;
    let size = bytes.len() as i64;
    
    let blob_id = Uuid::new_v4();
    let storage_key = format!("{}/{}", session_id, blob_id);
    
    let storage = get_storage();
    storage.put(&ObjectPath::from(storage_key.clone()), bytes.into()).await?;

    let blob = blobs::ActiveModel {
        id: ActiveValue::Set(blob_id),
        session_id: ActiveValue::Set(session_id),
        file_name: ActiveValue::Set(file_name.to_string()),
        content_type: ActiveValue::Set(content_type),
        size: ActiveValue::Set(size),
        storage_key: ActiveValue::Set(storage_key),
        ..Default::default()
    };

    blob.insert(&ctx.db).await?;

    Ok(blob_id)
}
