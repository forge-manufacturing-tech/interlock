mod common;

use common::TestContext;

async fn setup() -> TestContext {
    let mut ctx = TestContext::new();
    ctx.register().await.expect("Setup: registration failed");
    ctx.login().await.expect("Setup: login failed");
    ctx
}

async fn cleanup(ctx: &TestContext) {
    if let Some(ref project_id) = ctx.project_id {
        let _ = ctx.delete_project(project_id).await;
    }
}

#[tokio::test]
async fn test_create_project() {
    let mut ctx = setup().await;
    
    let project_id = ctx.create_project("Test Project", Some("Test Description"))
        .await
        .expect("Should create project");
    
    assert!(!project_id.is_empty(), "Project ID should not be empty");
    
    cleanup(&ctx).await;
}

#[tokio::test]
async fn test_list_projects() {
    let mut ctx = setup().await;
    
    let project_id = ctx.create_project("Test Project", None)
        .await
        .expect("Should create project");
    
    let projects = ctx.list_projects()
        .await
        .expect("Should list projects");
    
    assert!(!projects.is_empty(), "Should have at least one project");
    assert!(
        projects.iter().any(|p| p["id"].as_str() == Some(&project_id)),
        "Created project should be in list"
    );
    
    cleanup(&ctx).await;
}

#[tokio::test]
async fn test_delete_project() {
    let mut ctx = setup().await;
    
    let project_id = ctx.create_project("Test Project", None)
        .await
        .expect("Should create project");
    
    ctx.delete_project(&project_id)
        .await
        .expect("Should delete project");
    
    let projects = ctx.list_projects()
        .await
        .expect("Should list projects");
    
    assert!(
        !projects.iter().any(|p| p["id"].as_str() == Some(&project_id)),
        "Deleted project should not be in list"
    );
}

#[tokio::test]
async fn test_create_multiple_projects() {
    let mut ctx = setup().await;
    
    let id1 = ctx.create_project("Project 1", Some("First"))
        .await
        .expect("Should create first project");
    
    let id2 = ctx.create_project("Project 2", Some("Second"))
        .await
        .expect("Should create second project");
    
    assert_ne!(id1, id2, "Project IDs should be different");
    
    let projects = ctx.list_projects()
        .await
        .expect("Should list projects");
    
    assert!(projects.len() >= 2, "Should have at least 2 projects");
    
    // Cleanup
    let _ = ctx.delete_project(&id1).await;
    let _ = ctx.delete_project(&id2).await;
}

#[tokio::test]
async fn test_unauthorized_project_access() {
    let mut ctx1 = setup().await;
    let ctx2 = setup().await;
    
    // Create project with user 1
    let project_id = ctx1.create_project("User1 Project", None)
        .await
        .expect("Should create project");
    
    // User 2 should not see user 1's project
    let projects = ctx2.list_projects()
        .await
        .expect("Should list projects");
    
    assert!(
        !projects.iter().any(|p| p["id"].as_str() == Some(&project_id)),
        "User should not see other user's projects"
    );
    
    // Cleanup
    cleanup(&ctx1).await;
}
