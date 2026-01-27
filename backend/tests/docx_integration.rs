use backend::agent::docx::{DocElement, generate_docx_with_fetcher, ImageFetcher};
use serde_json::from_str;
use uuid::Uuid;
use anyhow::Result;
use async_trait::async_trait;

struct MockFetcher;

#[async_trait]
impl ImageFetcher for MockFetcher {
    async fn fetch_image(&self, _image_id: Uuid) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_deserialize_and_generate_list() {
    let json = r#"[
        {"type":"heading","level":1,"text":"Mass Production Plan: SpinBrush Electric Toothbrush"},
        {"type":"paragraph","text":"This document outlines..."},
        {"type":"list","items":["Phase 1: Pilot...","Phase 2: Ramp-up...","Phase 3: Mass..."]}
    ]"#;

    let elements: Vec<DocElement> = from_str(json).expect("Failed to deserialize");

    let fetcher = MockFetcher;
    let res = generate_docx_with_fetcher(elements, &fetcher).await;

    assert!(res.is_ok(), "Failed to generate docx: {:?}", res.err());
    let data = res.unwrap();
    assert!(!data.is_empty());

    // Check magic number for docx/zip (PK..)
    assert_eq!(&data[0..2], b"PK");
}
