mod common;

use common::TestContext;

async fn setup() -> TestContext {
    let mut ctx = TestContext::new();
    ctx.register().await.expect("Setup: registration failed");
    ctx.login().await.expect("Setup: login failed");
    ctx
}

async fn cleanup(ctx: &TestContext, session_ids: Vec<String>) {
    for session_id in session_ids {
        let _ = ctx.delete_session(&session_id).await;
    }
    if let Some(ref project_id) = ctx.project_id {
        let _ = ctx.delete_project(project_id).await;
    }
}

#[tokio::test]
async fn test_create_session() {
    let mut ctx = setup().await;
    
    let project_id = ctx.create_project("Test Project", None)
        .await
        .expect("Should create project");
    
    let session_id = ctx.create_session("Test Session", "println!(\"Hello\");", &project_id)
        .await
        .expect("Should create session");
    
    assert!(!session_id.is_empty(), "Session ID should not be empty");
    
    cleanup(&ctx, vec![session_id]).await;
}

#[tokio::test]
async fn test_list_sessions_for_project() {
    let mut ctx = setup().await;
    
    let project_id = ctx.create_project("Test Project", None)
        .await
        .expect("Should create project");
    
    let session_id = ctx.create_session("Test Session", "// code", &project_id)
        .await
        .expect("Should create session");
    
    let sessions = ctx.list_sessions(Some(&project_id))
        .await
        .expect("Should list sessions");
    
    assert!(!sessions.is_empty(), "Should have at least one session");
    assert!(
        sessions.iter().any(|s| s["id"].as_str() == Some(&session_id)),
        "Created session should be in list"
    );
    
    cleanup(&ctx, vec![session_id]).await;
}

#[tokio::test]
async fn test_create_session_requires_project() {
    let ctx = setup().await;
    
    // Trying to create session without project should fail
    // This would require modifying the API call, but testing current behavior
    let result = ctx.create_session("Orphan Session", "code", "invalid-uuid-string")
        .await;
    
    assert!(result.is_err(), "Creating session with invalid project should fail");
}

#[tokio::test]
async fn test_sessions_isolated_by_project() {
    let mut ctx = setup().await;
    
    let project1_id = ctx.create_project("Project 1", None)
        .await
        .expect("Should create project 1");
    
    let project2_id = ctx.create_project("Project 2", None)
        .await
        .expect("Should create project 2");
    
    let session1_id = ctx.create_session("Session P1", "code1", &project1_id)
        .await
        .expect("Should create session in project 1");
    
    let session2_id = ctx.create_session("Session P2", "code2", &project2_id)
        .await
        .expect("Should create session in project 2");
    
    // List sessions for project 1
    let p1_sessions = ctx.list_sessions(Some(&project1_id))
        .await
        .expect("Should list project 1 sessions");
    
    assert!(
        p1_sessions.iter().any(|s| s["id"].as_str() == Some(&session1_id)),
        "Project 1 should have session 1"
    );
    assert!(
        !p1_sessions.iter().any(|s| s["id"].as_str() == Some(&session2_id)),
        "Project 1 should not have session 2"
    );
    
    // Cleanup
    let _ = ctx.delete_session(&session1_id).await;
    let _ = ctx.delete_session(&session2_id).await;
    let _ = ctx.delete_project(&project1_id).await;
    let _ = ctx.delete_project(&project2_id).await;
}

#[tokio::test]
async fn test_delete_session() {
    let mut ctx = setup().await;
    
    let project_id = ctx.create_project("Test Project", None)
        .await
        .expect("Should create project");
    
    let session_id = ctx.create_session("Test Session", "code", &project_id)
        .await
        .expect("Should create session");
    
    ctx.delete_session(&session_id)
        .await
        .expect("Should delete session");
    
    let sessions = ctx.list_sessions(Some(&project_id))
        .await
        .expect("Should list sessions");
    
    assert!(
        !sessions.iter().any(|s| s["id"].as_str() == Some(&session_id)),
        "Deleted session should not be in list"
    );
    
    cleanup(&ctx, vec![]).await;
}
