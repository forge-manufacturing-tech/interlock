use interlock::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn test_model() {
    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();
}

#[test]
fn test_serialization() {
    use interlock::models::_entities::sessions::Model;
    use chrono::{Utc, TimeZone};
    use uuid::Uuid;

    let item = Model {
        created_at: Utc.timestamp_opt(0, 0).unwrap().into(),
        updated_at: Utc.timestamp_opt(0, 0).unwrap().into(),
        id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        title: Some("test".to_string()),
        content: Some("content".to_string()),
        project_id: Some(Uuid::new_v4()),
        pending_tasks: serde_json::json!([]),
        status: "active".to_string(),
    };

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"id\":\"11111111-1111-1111-1111-111111111111\""), "JSON was: {}", json);
}
