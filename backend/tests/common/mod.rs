use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use uuid::Uuid;

pub const API_URL: &str = "http://localhost:5150";

#[allow(dead_code)]
pub struct TestContext {
    pub client: Client,
    pub email: String,
    pub password: String,
    pub token: Option<String>,
    pub project_id: Option<String>,
}

#[allow(dead_code)]
impl TestContext {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            email: format!("test_{}@example.com", Uuid::new_v4()),
            password: "TestPass123!".to_string(),
            token: None,
            project_id: None,
        }
    }

    pub async fn register(&self) -> Result<(), String> {
        let res = self.client
            .post(format!("{}/api/auth/register", API_URL))
            .json(&json!({
                "email": self.email,
                "password": self.password,
                "name": "Test User"
            }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Registration failed: {}", text));
        }
        Ok(())
    }

    pub async fn login(&mut self) -> Result<(), String> {
        let res = self.client
            .post(format!("{}/api/auth/login", API_URL))
            .json(&json!({
                "email": self.email,
                "password": self.password
            }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Login failed: {}", text));
        }

        let data: Value = res.json().await.map_err(|e| format!("Parse failed: {}", e))?;
        self.token = Some(data["token"].as_str().ok_or("No token in response")?.to_string());
        Ok(())
    }

    pub fn get_token(&self) -> Result<&str, String> {
        self.token.as_deref().ok_or_else(|| "Not logged in".to_string())
    }

    pub async fn create_project(&mut self, name: &str, description: Option<&str>) -> Result<String, String> {
        let token = self.get_token()?;
        let res = self.client
            .post(format!("{}/api/projects", API_URL))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "name": name,
                "description": description
            }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Create project failed: {}", text));
        }

        let data: Value = res.json().await.map_err(|e| format!("Parse failed: {}", e))?;
        let project_id = data["id"].as_str().ok_or("No project ID in response")?.to_string();
        self.project_id = Some(project_id.clone());
        Ok(project_id)
    }

    pub async fn delete_project(&self, project_id: &str) -> Result<(), String> {
        let token = self.get_token()?;
        let res = self.client
            .delete(format!("{}/api/projects/{}", API_URL, project_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() && res.status() != StatusCode::NOT_FOUND {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Delete project failed: {}", text));
        }
        Ok(())
    }

    pub async fn list_projects(&self) -> Result<Vec<Value>, String> {
        let token = self.get_token()?;
        let res = self.client
            .get(format!("{}/api/projects", API_URL))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("List projects failed: {}", text));
        }

        res.json().await.map_err(|e| format!("Parse failed: {}", e))
    }

    pub async fn create_session(&self, title: &str, content: &str, project_id: &str) -> Result<String, String> {
        let token = self.get_token()?;
        let res = self.client
            .post(format!("{}/api/sessions", API_URL))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "title": title,
                "content": content,
                "project_id": project_id
            }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Create session failed: {}", text));
        }

        let data: Value = res.json().await.map_err(|e| format!("Parse failed: {}", e))?;
        Ok(data["id"].as_str().ok_or("No session ID in response")?.to_string())
    }

    pub async fn list_sessions(&self, project_id: Option<&str>) -> Result<Vec<Value>, String> {
        let token = self.get_token()?;
        let url = if let Some(pid) = project_id {
            format!("{}/api/sessions?project_id={}", API_URL, pid)
        } else {
            format!("{}/api/sessions", API_URL)
        };

        let res = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("List sessions failed: {}", text));
        }

        res.json().await.map_err(|e| format!("Parse failed: {}", e))
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), String> {
        let token = self.get_token()?;
        let res = self.client
            .delete(format!("{}/api/sessions/{}", API_URL, session_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() && res.status() != StatusCode::NOT_FOUND {
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Delete session failed: {}", text));
        }
        Ok(())
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup happens in individual tests to ensure proper async context
    }
}
