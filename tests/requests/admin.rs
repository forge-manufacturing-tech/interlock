use interlock::{app::App, models::users};
use loco_rs::testing::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_admin_auto_assignment() {
    request::<App, _, _>(|request, ctx| async move {
        let email = "admin@loco.com";
        let payload = serde_json::json!({
            "name": "Admin User",
            "email": email,
            "password": "password"
        });

        // Register first user
        let response = request.post("/api/auth/register").json(&payload).await;
        assert_eq!(response.status_code(), 200);

        let user = users::Model::find_by_email(&ctx.db, email).await.unwrap();
        assert_eq!(user.role, "admin");

        // Register second user
        let email2 = "editor@loco.com";
        let payload2 = serde_json::json!({
            "name": "Editor User",
            "email": email2,
            "password": "password"
        });
        let response2 = request.post("/api/auth/register").json(&payload2).await;
        assert_eq!(response2.status_code(), 200);

        let user2 = users::Model::find_by_email(&ctx.db, email2).await.unwrap();
        assert_eq!(user2.role, "user");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_admin_access_control() {
    request::<App, _, _>(|request, ctx| async move {
        // Create Admin
        let admin_email = "admin@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Admin", "email": admin_email, "password": "password"
        })).await;

        // Login Admin
        let user = users::Model::find_by_email(&ctx.db, admin_email).await.unwrap();
        if let Some(token) = user.email_verification_token {
            request.get(&format!("/api/auth/verify/{}", token)).await;
        }
        let response = request.post("/api/auth/login").json(&serde_json::json!({
            "email": admin_email,
            "password": "password"
        })).await;
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let admin_token = json["token"].as_str().unwrap().to_string();

        // Create Editor
        let editor_email = "editor@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Editor", "email": editor_email, "password": "password"
        })).await;

        // Login Editor
        let user = users::Model::find_by_email(&ctx.db, editor_email).await.unwrap();
        if let Some(token) = user.email_verification_token {
            request.get(&format!("/api/auth/verify/{}", token)).await;
        }
        let response = request.post("/api/auth/login").json(&serde_json::json!({
            "email": editor_email,
            "password": "password"
        })).await;
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let editor_token = json["token"].as_str().unwrap().to_string();

        // Admin can list users
        let response = request.get("/api/admin/users")
            .add_header("Authorization", &format!("Bearer {}", admin_token))
            .await;
        assert_eq!(response.status_code(), 200);

        // Editor cannot list users
        let response = request.get("/api/admin/users")
            .add_header("Authorization", &format!("Bearer {}", editor_token))
            .await;
        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_group_management() {
    request::<App, _, _>(|request, ctx| async move {
        // Create Admin
        let admin_email = "admin@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Admin", "email": admin_email, "password": "password"
        })).await;

        // Login Admin
        let user = users::Model::find_by_email(&ctx.db, admin_email).await.unwrap();
        if let Some(token) = user.email_verification_token {
            request.get(&format!("/api/auth/verify/{}", token)).await;
        }
        let response = request.post("/api/auth/login").json(&serde_json::json!({
            "email": admin_email,
            "password": "password"
        })).await;
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let admin_token = json["token"].as_str().unwrap().to_string();

        // Create Group
        let response = request.post("/api/admin/groups")
            .add_header("Authorization", &format!("Bearer {}", admin_token))
            .json(&serde_json::json!({"name": "Test Group"}))
            .await;
        assert_eq!(response.status_code(), 200);
        let group_json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let group_id = group_json["id"].as_i64().unwrap();

        // Create another user
        let user_email = "user@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "User", "email": user_email, "password": "password"
        })).await;
        let user = users::Model::find_by_email(&ctx.db, user_email).await.unwrap();

        // Add user to group
        let response = request.post(&format!("/api/admin/groups/{}/users", group_id))
            .add_header("Authorization", &format!("Bearer {}", admin_token))
            .json(&serde_json::json!({"user_id": user.id}))
            .await;
        assert_eq!(response.status_code(), 200);

        // Verify association
        use interlock::models::_entities::users_groups;
        let count = users_groups::Entity::find()
            .filter(users_groups::Column::UserId.eq(user.id))
            .filter(users_groups::Column::GroupId.eq(group_id))
            .count(&ctx.db)
            .await
            .unwrap();
        assert_eq!(count, 1);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_viewer_restrictions() {
    request::<App, _, _>(|request, ctx| async move {
        // Create Admin
        let admin_email = "admin@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Admin", "email": admin_email, "password": "password"
        })).await;

        // Login Admin
        let user = users::Model::find_by_email(&ctx.db, admin_email).await.unwrap();
        if let Some(token) = user.email_verification_token {
            request.get(&format!("/api/auth/verify/{}", token)).await;
        }
        let response = request.post("/api/auth/login").json(&serde_json::json!({
            "email": admin_email,
            "password": "password"
        })).await;
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let admin_token = json["token"].as_str().unwrap().to_string();

        // Create Viewer
        let viewer_email = "viewer@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Viewer", "email": viewer_email, "password": "password"
        })).await;

        // Update role to viewer
        let viewer = users::Model::find_by_email(&ctx.db, viewer_email).await.unwrap();
        request.put(&format!("/api/admin/users/{}", viewer.id))
            .add_header("Authorization", &format!("Bearer {}", admin_token))
            .json(&serde_json::json!({"role": "viewer"}))
            .await;

        // Login Viewer
        let user = users::Model::find_by_email(&ctx.db, viewer_email).await.unwrap();
        if let Some(token) = user.email_verification_token {
            request.get(&format!("/api/auth/verify/{}", token)).await;
        }
        let response = request.post("/api/auth/login").json(&serde_json::json!({
            "email": viewer_email,
            "password": "password"
        })).await;
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let viewer_token = json["token"].as_str().unwrap().to_string();

        // Try to create project
        let response = request.post("/api/projects")
            .add_header("Authorization", &format!("Bearer {}", viewer_token))
            .json(&serde_json::json!({"name": "Project"}))
            .await;
        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_delete_user() {
    request::<App, _, _>(|request, ctx| async move {
        // Create Admin
        let admin_email = "admin@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Admin", "email": admin_email, "password": "password"
        })).await;

        // Login Admin
        let user = users::Model::find_by_email(&ctx.db, admin_email).await.unwrap();
        if let Some(token) = user.email_verification_token {
            request.get(&format!("/api/auth/verify/{}", token)).await;
        }
        let response = request.post("/api/auth/login").json(&serde_json::json!({
            "email": admin_email,
            "password": "password"
        })).await;
        let json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let admin_token = json["token"].as_str().unwrap().to_string();

        // Create User to Delete
        let user_email = "delete_me@loco.com";
        request.post("/api/auth/register").json(&serde_json::json!({
            "name": "Delete Me", "email": user_email, "password": "password"
        })).await;

        let user_to_delete = users::Model::find_by_email(&ctx.db, user_email).await.unwrap();
        let user_pid = user_to_delete.pid;

        // Delete User
        let response = request.delete(&format!("/api/admin/users/{}", user_pid))
            .add_header("Authorization", &format!("Bearer {}", admin_token))
            .await;
        assert_eq!(response.status_code(), 200);

        // Verify User Deleted
        let result = users::Model::find_by_pid(&ctx.db, &user_pid.to_string()).await;
        assert!(result.is_err());
    })
    .await;
}
