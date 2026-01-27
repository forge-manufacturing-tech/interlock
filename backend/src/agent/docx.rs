use anyhow::{anyhow, Result};
use docx_rs::{
    Docx, Paragraph, Run, Table, TableCell, TableRow, Pic, AlignmentType, BreakType
};
use loco_rs::prelude::*;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::Deserialize;
use uuid::Uuid;
use crate::models::_entities::blobs;
use crate::storage::get_storage;
use object_store::path::Path as ObjectPath;
use async_trait::async_trait;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocElement {
    Paragraph {
        text: String,
        #[serde(default)]
        style: Option<String>,
    },
    Heading {
        text: String,
        level: usize,
    },
    Table {
        rows: Vec<Vec<String>>,
        #[serde(default)]
        headers: Option<Vec<String>>,
    },
    Image {
        image_id: Uuid,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        caption: Option<String>,
    },
}

#[async_trait]
pub trait ImageFetcher {
    async fn fetch_image(&self, image_id: Uuid) -> Result<Vec<u8>>;
}

pub struct DbImageFetcher<'a> {
    pub ctx: &'a AppContext,
}

#[async_trait]
impl<'a> ImageFetcher for DbImageFetcher<'a> {
    async fn fetch_image(&self, image_id: Uuid) -> Result<Vec<u8>> {
        let blob = blobs::Entity::find_by_id(image_id)
            .one(&self.ctx.db)
            .await
            .map_err(|e| anyhow!("Failed to find image blob: {}", e))?;

        if let Some(blob_record) = blob {
             let storage = get_storage();
             let data = storage.get(&ObjectPath::from(blob_record.storage_key))
                .await?
                .bytes()
                .await?;
             Ok(data.to_vec())
        } else {
            Err(anyhow!("Image blob not found"))
        }
    }
}

pub fn process_text_content(text: &str) -> Paragraph {
    let parser = Parser::new(text);
    let mut p = Paragraph::new();
    let mut is_bold = false;
    let mut is_italic = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Strong => is_bold = true,
                Tag::Emphasis => is_italic = true,
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Strong => is_bold = false,
                TagEnd::Emphasis => is_italic = false,
                _ => {}
            },
            Event::Text(t) => {
                let mut run = Run::new();
                if is_bold {
                    run = run.bold();
                }
                if is_italic {
                    run = run.italic();
                }
                p = p.add_run(run.add_text(t.as_ref()));
            }
            Event::SoftBreak | Event::HardBreak => {
                p = p.add_run(Run::new().add_break(BreakType::TextWrapping));
            }
            _ => {}
        }
    }
    p
}

pub async fn generate_docx_with_fetcher(
    elements: Vec<DocElement>,
    fetcher: &impl ImageFetcher,
) -> Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("output.docx");
    let file = std::fs::File::create(&path)?;

    let mut doc = Docx::new();

    for element in elements {
        match element {
            DocElement::Paragraph { text, style: _ } => {
                // TODO: Apply style if mapped.
                doc = doc.add_paragraph(process_text_content(&text));
            }
            DocElement::Heading { text, level } => {
                 let style = match level {
                    1 => "Heading1",
                    2 => "Heading2",
                    3 => "Heading3",
                    _ => "Heading4",
                };
                let p = process_text_content(&text).style(style);
                doc = doc.add_paragraph(p);
            }
            DocElement::Table { rows, headers } => {
                let mut table_rows = Vec::new();

                // Add Header Row if exists
                if let Some(header_texts) = headers {
                    let mut cells = Vec::new();
                    for h_text in header_texts {
                         let _p = process_text_content(&h_text);
                         // Ideally we would style this bold or add shading
                         let p = Run::new().bold().add_text(h_text);
                         cells.push(TableCell::new().add_paragraph(Paragraph::new().add_run(p)));
                    }
                    table_rows.push(TableRow::new(cells));
                }

                // Add Data Rows
                for row_data in rows {
                    let mut cells = Vec::new();
                    for cell_text in row_data {
                         cells.push(TableCell::new().add_paragraph(process_text_content(&cell_text)));
                    }
                    table_rows.push(TableRow::new(cells));
                }

                doc = doc.add_table(Table::new(table_rows));
            }
            DocElement::Image { image_id, width, height, caption } => {
                match fetcher.fetch_image(image_id).await {
                    Ok(img_data) => {
                        let mut pic = Pic::new(&img_data);

                        if let (Some(w), Some(h)) = (width, height) {
                             pic = pic.size(w, h);
                        }

                        let mut p = Paragraph::new().add_run(Run::new().add_image(pic));
                        if let Some(cap) = caption {
                            p = p.add_run(Run::new().add_break(BreakType::TextWrapping));
                            p = p.add_run(Run::new().add_text(cap));
                        }
                        p = p.align(AlignmentType::Center);
                        doc = doc.add_paragraph(p);
                    },
                    Err(e) => {
                        // Log warning but don't fail entire document
                        eprintln!("Warning: Failed to fetch image {}: {}", image_id, e);
                    }
                }
            }
        }
    }

    doc.build().pack(file)?;

    let mut f = std::fs::File::open(&path)?;
    use std::io::Read;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

pub async fn generate_docx_from_json(
    elements: Vec<DocElement>,
    ctx: &AppContext,
) -> Result<Vec<u8>> {
    let fetcher = DbImageFetcher { ctx };
    generate_docx_with_fetcher(elements, &fetcher).await
}
