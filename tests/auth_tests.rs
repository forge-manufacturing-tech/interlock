mod common;

use common::TestContext;

#[tokio::test]
async fn test_user_registration_and_login() {
    let mut ctx = TestContext::new();
    
    // Register
    ctx.register().await.expect("Registration should succeed");
    
    // Login
    ctx.login().await.expect("Login should succeed");
    
    // Verify we have a token
    assert!(ctx.token.is_some(), "Should have token after login");
}

#[tokio::test]
async fn test_duplicate_registration_fails() {
    let ctx = TestContext::new();
    
    // First registration should succeed
    ctx.register().await.expect("First registration should succeed");
    
    // Second registration with same email should fail
    let result = ctx.register().await;
    assert!(result.is_err(), "Duplicate registration should fail");
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    let mut ctx = TestContext::new();
    
    // Register
    ctx.register().await.expect("Registration should succeed");
    
    // Try to login with wrong password
    ctx.password = "WrongPassword123!".to_string();
    let result = ctx.login().await;
    
    assert!(result.is_err(), "Login with wrong password should fail");
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let mut ctx = TestContext::new();
    
    // Try to login without registering
    let result = ctx.login().await;
    
    assert!(result.is_err(), "Login for nonexistent user should fail");
}
