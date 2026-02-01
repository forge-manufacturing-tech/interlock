use interlock::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_openapi_ui() {
    request::<App, _, _>(|request, _ctx| async move {
        // Test Swagger UI HTML
        let response = request.get("/api-docs/ui").await;
        assert_eq!(response.status_code(), 200);
        assert!(response.text().contains("SwaggerUI"));

        // Test OpenAPI JSON
        let response = request.get("/api-docs/openapi.json").await;
        assert_eq!(response.status_code(), 200);
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        assert!(json["openapi"].as_str().is_some());
    })
    .await;
}
