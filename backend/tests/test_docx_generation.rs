use backend::agent::docx::{DocElement, generate_docx_with_fetcher, ImageFetcher};
use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

struct MockImageFetcher;

#[async_trait]
impl ImageFetcher for MockImageFetcher {
    async fn fetch_image(&self, _image_id: Uuid) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_generate_docx_with_table() {
    let json_content = r#"
    [
        { "type": "heading", "level": 1, "text": "Test Report" },
        { "type": "paragraph", "text": "This is a **bold** paragraph." },
        { "type": "table", "headers": ["Header 1", "Header 2"], "rows": [["Row 1 Col 1", "Row 1 Col 2"], ["Row 2 Col 1", "Row 2 Col 2"]] }
    ]
    "#;

    let elements: Vec<DocElement> = serde_json::from_str(json_content).expect("Failed to parse JSON");
    let fetcher = MockImageFetcher;

    let result = generate_docx_with_fetcher(elements, &fetcher).await;

    assert!(result.is_ok(), "Failed to generate DOCX: {:?}", result.err());
    let bytes = result.unwrap();
    assert!(bytes.len() > 0);
    println!("Generated DOCX size: {} bytes", bytes.len());
}
