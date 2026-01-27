use calamine::{Reader, Xlsx, open_workbook_from_rs, Data};
use rust_xlsxwriter::{Workbook, Format};
use uuid::Uuid;
use std::io::{Cursor, Read};
use crate::storage::get_storage;
use object_store::path::Path as ObjectPath;
use loco_rs::prelude::*;
use crate::models::_entities::{blobs};
use zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use lopdf::Document as PdfDocument;
use docx_rs::{Docx, Paragraph, Run};

pub async fn list_files(session_id: Uuid, ctx: &AppContext) -> anyhow::Result<String> {
    let files = blobs::Entity::find()
        .filter(blobs::Column::SessionId.eq(session_id))
        .all(&ctx.db)
        .await?;
    
    let mut res = String::from("Files in this session:\n");
    for file in files {
        res.push_str(&format!("- {} (ID: {}, Type: {})\n", file.file_name, file.id, file.content_type));
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
    let workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
    
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
    
    save_blob(file_name, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", buf, session_id, ctx).await
}

pub async fn create_text_file(
    file_name: &str,
    content: &str,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    save_blob(file_name, "text/plain", content.as_bytes().to_vec(), session_id, ctx).await
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
    save_blob(file_name, &content_type, bytes.into(), session_id, ctx).await
}

use base64::{Engine as _, engine::general_purpose};

pub async fn generate_image(
    prompt: &str,
    file_name: &str,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY not set"))?;
        
    let client = reqwest::Client::new();
    
    // Using gemini-2.5-flash-image as recommended for image generation "Nano Banana"
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-image:generateContent?key={}",
        api_key
    );

    // Payload structure for generateContent
    // Note: For gemini-2.5-flash-image, we should not specify responseMimeType: image/png in generationConfig
    // as it triggers INVALID_ARGUMENT. The model returns image data by default.
    let payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    let resp = client.post(&url)
        .json(&payload)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        println!("Image Gen API Error: Status: {}, Body: {}", status, text);
        return Err(anyhow::anyhow!("Image generation API failed: {}", text));
    }

    let json: serde_json::Value = resp.json().await?;
    
    // Parse response for generateContent image result
    // The image data should be in candidates[0].content.parts[].inline_data
    let parts = json["candidates"][0]["content"]["parts"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No parts in response. JSON: {:?}", json))?;
    
    let mut base64_data = None;
    for part in parts {
        if let Some(inline_data) = part.get("inlineData") {
             if let Some(data) = inline_data.get("data") {
                 base64_data = data.as_str();
                 break;
             }
        }
    }
    
    let base64_data = base64_data.ok_or_else(|| anyhow::anyhow!("No inlineData image found in response. JSON: {:?}", json))?;
    
    // Clean potential newlines
    let base64_clean = base64_data.replace('\n', "").replace('\r', "");

    let image_data = general_purpose::STANDARD
        .decode(&base64_clean)
        .map_err(|e| anyhow::anyhow!("Failed to decode base64 image: {}", e))?;

    save_blob(file_name, "image/png", image_data, session_id, ctx).await
}

// --- NEW TOOLS ---

// Helper to save blobs avoiding duplication
async fn save_blob(file_name: &str, content_type: &str, data: Vec<u8>, session_id: Uuid, ctx: &AppContext) -> anyhow::Result<Uuid> {
    let blob_id = Uuid::new_v4();
    let storage_key = format!("{}/{}", session_id, blob_id);
    let size = data.len() as i64;
    
    let storage = get_storage();
    storage.put(&ObjectPath::from(storage_key.clone()), data.into()).await?;

    let blob = blobs::ActiveModel {
        id: ActiveValue::Set(blob_id),
        session_id: ActiveValue::Set(session_id),
        file_name: ActiveValue::Set(file_name.to_string()),
        content_type: ActiveValue::Set(content_type.to_string()),
        size: ActiveValue::Set(size),
        storage_key: ActiveValue::Set(storage_key),
        ..Default::default()
    };

    blob.insert(&ctx.db).await?;
    Ok(blob_id)
}

fn extract_text_from_docx(data: &[u8]) -> anyhow::Result<String> {
    let reader = Cursor::new(data);
    let mut zip = ZipArchive::new(reader)?;
    let mut xml_content = String::new();
    // word/document.xml is the main content
    if let Ok(mut f) = zip.by_name("word/document.xml") {
         f.read_to_string(&mut xml_content)?;
    } else {
        return Err(anyhow::anyhow!("Invalid DOCX: missing word/document.xml"));
    }

    let mut reader = XmlReader::from_str(&xml_content);
    let mut txt = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => txt.push_str(&e.unescape()?),
            Ok(Event::Eof) => break,
            Err(_) => {}, // ignore errors for now, try to recover
            _ => (),
        }
        buf.clear();
    }
    Ok(txt)
}

fn extract_text_from_pdf(data: &[u8]) -> anyhow::Result<String> {
   let doc = PdfDocument::load_mem(data)?;
   let mut texts = Vec::new();
   // Simple text extraction from all pages
   for (i, _) in doc.get_pages() {
       if let Ok(text) = doc.extract_text(&[i]) {
            texts.push(text);
       }
   }
   Ok(texts.join("\n\n"))
}

pub async fn read_file(blob_id: Uuid, ctx: &AppContext) -> anyhow::Result<String> {
    let blob = blobs::Entity::find_by_id(blob_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Blob not found"))?;
    
    let storage = get_storage();
    let data = storage.get(&ObjectPath::from(blob.storage_key))
        .await?
        .bytes()
        .await?;
    
    let filename = blob.file_name.to_lowercase();
    
    if filename.ends_with(".docx") {
        extract_text_from_docx(&data)
    } else if filename.ends_with(".pdf") {
        extract_text_from_pdf(&data)
    } else {
        // Assume text
        String::from_utf8(data.to_vec())
            .map_err(|e| anyhow::anyhow!("Failed to read text file: {}", e))
    }
}

pub async fn create_word_doc(
    file_name: &str,
    content: &str,
    image_id: Option<Uuid>,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    use pulldown_cmark::{Event, Parser, Tag, HeadingLevel, TagEnd};

    let buf = {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("output.docx");
        let file = std::fs::File::create(&path)?;
        
        let mut doc = Docx::new();
        
        // Parse Markdown and add to document
        let parser = Parser::new(content);
        let mut current_paragraph: Option<Paragraph> = None;
        let mut is_bold = false;
        let mut is_italic = false;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            current_paragraph = Some(Paragraph::new());
                        }
                        Tag::Heading { level, .. } => {
                            let style = match level {
                                HeadingLevel::H1 => "Heading1",
                                HeadingLevel::H2 => "Heading2",
                                HeadingLevel::H3 => "Heading3",
                                _ => "Heading4",
                            };
                            current_paragraph = Some(Paragraph::new().style(style));
                        }
                        Tag::Strong => is_bold = true,
                        Tag::Emphasis => is_italic = true,
                        Tag::List(..) => {}
                        Tag::Item => {
                            current_paragraph = Some(Paragraph::new().add_run(Run::new().add_text("• ")));
                        }
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    match tag {
                        TagEnd::Paragraph | TagEnd::Heading(..) | TagEnd::Item => {
                            if let Some(p) = current_paragraph.take() {
                                doc = doc.add_paragraph(p);
                            }
                        }
                        TagEnd::Strong => is_bold = false,
                        TagEnd::Emphasis => is_italic = false,
                        TagEnd::List(..) => {}
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    let mut run = Run::new();
                    if is_bold { run = run.bold(); }
                    if is_italic { run = run.italic(); }
                    
                    if let Some(mut p) = current_paragraph.take() {
                        p = p.add_run(run.add_text(text.as_ref()));
                        current_paragraph = Some(p);
                    } else {
                        doc = doc.add_paragraph(Paragraph::new().add_run(run.add_text(text.as_ref())));
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if let Some(mut p) = current_paragraph.take() {
                        p = p.add_run(Run::new().add_break(docx_rs::BreakType::TextWrapping));
                        current_paragraph = Some(p);
                    }
                }
                _ => {}
            }
        }

        // Add Image if provided
        if let Some(img_id) = image_id {
             let blob = blobs::Entity::find_by_id(img_id)
                .one(&ctx.db)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to find image blob: {}", e))?;

             if let Some(blob_record) = blob {
                 let storage = get_storage();
                 let img_data = storage.get(&ObjectPath::from(blob_record.storage_key))
                    .await?
                    .bytes()
                    .await?;
                 
                 let img = docx_rs::Pic::new(&img_data.to_vec());
                 doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_image(img)));
             }
        }
        
        doc.build().pack(file)?;
        
        let mut f = std::fs::File::open(&path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        buf
    };
    
    save_blob(file_name, "application/vnd.openxmlformats-officedocument.wordprocessingml.document", buf, session_id, ctx).await
}

pub async fn create_pdf_doc(
    file_name: &str,
    content: &str,
    session_id: Uuid,
    ctx: &AppContext,
) -> anyhow::Result<Uuid> {
    // Ensure font exists locally to avoid system font issues
    let font_dir = std::path::Path::new("storage/fonts");
    if !font_dir.exists() {
        std::fs::create_dir_all(font_dir).map_err(|e| anyhow::anyhow!("Failed to create font dir: {}", e))?;
    }
    
    let font_path = font_dir.join("Roboto-Regular.ttf");
    
    if !font_path.exists() {
        // Download font (Roboto Regular)
        println!("Downloading font for PDF generation...");
        let url = "https://github.com/google/fonts/raw/main/apache/roboto/Roboto-Regular.ttf";
        // We can't use the simple reqwest::get if we need to reuse the client or similar, 
        // but creating a new client here is fine for this occasional task.
        let resp = reqwest::get(url).await
            .map_err(|e| anyhow::anyhow!("Failed to request font: {}", e))?;
            
        if !resp.status().is_success() {
             return Err(anyhow::anyhow!("Failed to download font from {}: {}", url, resp.status()));
        }
        
        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to get font bytes: {}", e))?;
            
        std::fs::write(&font_path, bytes)
            .map_err(|e| anyhow::anyhow!("Failed to save font to {:?}: {}", font_path, e))?;
        println!("Font downloaded successfully.");
    }

    // Wrap in block to ensure !Send types from genpdf are dropped before await (save_blob)
    let buf = {
        let font_bytes = std::fs::read(&font_path)
            .map_err(|e| anyhow::anyhow!("Failed to read font file from {:?}: {}", font_path, e))?;
            
        let font_data = genpdf::fonts::FontData::new(font_bytes, None)
            .map_err(|e| anyhow::anyhow!("Failed to parse font data: {}", e))?;
            
        // Use the same font for all styles since we're just downloading Regular
        // Ideally we'd download Bold/Italic too, but this is sufficient for basic reports
        let font_family = genpdf::fonts::FontFamily {
            regular: font_data.clone(),
            bold: font_data.clone(), 
            italic: font_data.clone(),
            bold_italic: font_data,
        };
        
        let mut doc = genpdf::Document::new(font_family);
        doc.set_title("Generated Document");
        doc.set_minimal_conformance(); 
        doc.set_line_spacing(1.2);
        
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(12); // slightly larger margin
        doc.set_page_decorator(decorator);
        
        // Add content, splitting by paragraphs
        for paragraph_text in content.split("\n\n") {
             doc.push(genpdf::elements::Paragraph::new(paragraph_text));
             // Add some spacing after paragraphs? genpdf doesn't have explicit margin-bottom for Paragraph element easily 
             // without wrapping, generally \n\n split is enough if we handle it right.
             doc.push(genpdf::elements::Break::new(1));
        }
        
        let mut buf = Vec::new();
        doc.render(&mut buf).map_err(|e| anyhow::anyhow!("Failed to render PDF: {}", e))?;
        buf
    };
    
    save_blob(file_name, "application/pdf", buf, session_id, ctx).await
}

pub async fn search_internet(query: &str) -> anyhow::Result<String> {
    let api_key = std::env::var("GOOGLE_SEARCH_API_KEY").map_err(|_| anyhow::anyhow!("GOOGLE_SEARCH_API_KEY not set"))?;
    let cx = std::env::var("GOOGLE_SEARCH_CX").map_err(|_| anyhow::anyhow!("GOOGLE_SEARCH_CX not set"))?;

    let client = reqwest::Client::new();
    let mut url = reqwest::Url::parse("https://www.googleapis.com/customsearch/v1")
        .map_err(|e| anyhow::anyhow!("Failed to parse URL: {}", e))?;

    url.query_pairs_mut()
        .append_pair("key", &api_key)
        .append_pair("cx", &cx)
        .append_pair("q", query);

    let resp = client.get(url)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        return Err(anyhow::anyhow!("Search API Error: Status: {}, Body: {}", status, text));
    }

    let json: serde_json::Value = resp.json().await?;

    let items = json["items"].as_array().ok_or_else(|| anyhow::anyhow!("No items found in search response"))?;

    let mut result = format!("Search Results for '{}':\n", query);
    for (i, item) in items.iter().take(5).enumerate() {
        let title = item["title"].as_str().unwrap_or("No Title");
        let link = item["link"].as_str().unwrap_or("No Link");
        let snippet = item["snippet"].as_str().unwrap_or("No Snippet").replace('\n', " ");

        result.push_str(&format!("{}. {} ({})\n   {}\n", i + 1, title, link, snippet));
    }

    Ok(result)
}
