pub mod tools;
pub mod docx;

use loco_rs::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, ColumnTrait};
use crate::models::_entities::messages;
use crate::agent::tools::{excel_to_csv, create_excel, list_files, get_excel_sheets, create_text_file, download_from_url, read_file, create_word_doc, create_pdf_doc, generate_image};


use regex::Regex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub stop_sequences: Vec<String>,
    pub max_output_tokens: Option<i32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
}

#[async_trait]
pub trait AgentTool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String>;
}

pub struct ToolRegistry {
    pub tools: std::collections::HashMap<String, Box<dyn AgentTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }

    pub fn register<T: AgentTool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name(), Box::new(tool));
    }

    pub fn get_tool_descriptions(&self) -> String {
        self.tools.values()
            .enumerate()
            .map(|(i, t)| format!("{}. {}: {}", i + 1, t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Implement default tools
struct ListFilesTool;
#[async_trait]
impl AgentTool for ListFilesTool {
    fn name(&self) -> String { "list_files".to_string() }
    fn description(&self) -> String { "Lists all files available in the current session.".to_string() }
    async fn call(&self, _input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        list_files(session_id, ctx).await
    }
}

struct ExcelSheetsTool;
#[async_trait]
impl AgentTool for ExcelSheetsTool {
    fn name(&self) -> String { "get_excel_sheets".to_string() }
    fn description(&self) -> String { "Returns a list of sheet names in an Excel file. (blob_id: string)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, _session_id: Uuid) -> anyhow::Result<String> {
        let blob_id_str = input["blob_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing blob_id"))?;
        let blob_id = Uuid::parse_str(blob_id_str)?;
        let s = get_excel_sheets(blob_id, ctx).await?;
        Ok(format!("Sheets: {:?}", s))
    }
}

struct ExcelToCsvTool;
#[async_trait]
impl AgentTool for ExcelToCsvTool {
    fn name(&self) -> String { "excel_to_csv".to_string() }
    fn description(&self) -> String { "Converts a specific sheet of an Excel file to CSV text. (blob_id: string, sheet_name: string?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, _session_id: Uuid) -> anyhow::Result<String> {
        let blob_id_str = input["blob_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing blob_id"))?;
        let blob_id = Uuid::parse_str(blob_id_str)?;
        let sheet_name = input["sheet_name"].as_str().map(|s| s.to_string());
        excel_to_csv(blob_id, sheet_name, ctx).await
    }
}

struct ReadFileTool;
#[async_trait]
impl AgentTool for ReadFileTool {
    fn name(&self) -> String { "read_file".to_string() }
    fn description(&self) -> String { "Reads the text content of a file (PDF, DOCX, CSV, or Text). (blob_id: string)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, _session_id: Uuid) -> anyhow::Result<String> {
        let blob_id_str = input["blob_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing blob_id"))?;
        let blob_id = Uuid::parse_str(blob_id_str)?;
        read_file(blob_id, ctx).await
    }
}

struct CreateExcelTool;
#[async_trait]
impl AgentTool for CreateExcelTool {
    fn name(&self) -> String { "create_excel".to_string() }
    fn description(&self) -> String { "Creates a new Excel file. (file_name: string, rows: string[][], replace_existing: bool?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        let file_name = input["file_name"].as_str().unwrap_or("output.xlsx");
        let rows_val = input["rows"].as_array().ok_or_else(|| anyhow::anyhow!("Missing rows"))?;
        let replace_existing = input["replace_existing"].as_bool().unwrap_or(false);
        let mut rows = Vec::new();
        for row_val in rows_val {
            let row: Vec<String> = row_val.as_array().unwrap_or(&vec![]).iter().map(|v| v.as_str().unwrap_or("").to_string()).collect();
            rows.push(row);
        }
        let id = create_excel(file_name, rows, session_id, ctx, replace_existing).await?;
        Ok(format!("Success: New Excel file '{}' created. ID: {}.", file_name, id))
    }
}

struct CreateWordDocTool;
#[async_trait]
impl AgentTool for CreateWordDocTool {
    fn name(&self) -> String { "create_word_doc".to_string() }
    fn description(&self) -> String { "Creates a new Word document. (file_name: string, content: string, image_id: string?, replace_existing: bool?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        let file_name = input["file_name"].as_str().unwrap_or("output.docx");
        let content = input["content"].as_str().ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let image_id = input["image_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
        let replace_existing = input["replace_existing"].as_bool().unwrap_or(false);
        let id = create_word_doc(file_name, content, image_id, session_id, ctx, replace_existing).await?;
        Ok(format!("Success: New Word doc '{}' created. ID: {}.", file_name, id))
    }
}

struct CreatePdfDocTool;
#[async_trait]
impl AgentTool for CreatePdfDocTool {
    fn name(&self) -> String { "create_pdf_doc".to_string() }
    fn description(&self) -> String { "Creates a new PDF document. (file_name: string, content: string, replace_existing: bool?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        let file_name = input["file_name"].as_str().unwrap_or("output.pdf");
        let content = input["content"].as_str().ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let replace_existing = input["replace_existing"].as_bool().unwrap_or(false);
        let id = create_pdf_doc(file_name, content, session_id, ctx, replace_existing).await?;
        Ok(format!("Success: New PDF '{}' created. ID: {}.", file_name, id))
    }
}

struct CreateTextFileTool;
#[async_trait]
impl AgentTool for CreateTextFileTool {
    fn name(&self) -> String { "create_text_file".to_string() }
    fn description(&self) -> String { "Creates a text file. (file_name: string, content: string, replace_existing: bool?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        let file_name = input["file_name"].as_str().unwrap_or("output.txt");
        let content = input["content"].as_str().ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let replace_existing = input["replace_existing"].as_bool().unwrap_or(false);
        let id = create_text_file(file_name, content, session_id, ctx, replace_existing).await?;
        Ok(format!("Success: New text file '{}' created. ID: {}.", file_name, id))
    }
}

struct DownloadFromUrlTool;
#[async_trait]
impl AgentTool for DownloadFromUrlTool {
    fn name(&self) -> String { "download_from_url".to_string() }
    fn description(&self) -> String { "Downloads a file from a URL. (url: string, file_name: string, replace_existing: bool?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        let url = input["url"].as_str().ok_or_else(|| anyhow::anyhow!("Missing url"))?;
        let file_name = input["file_name"].as_str().unwrap_or("downloaded_file");
        let replace_existing = input["replace_existing"].as_bool().unwrap_or(false);
        let id = download_from_url(url, file_name, session_id, ctx, replace_existing).await?;
        Ok(format!("Success: File downloaded as '{}'. ID: {}.", file_name, id))
    }
}

struct GenerateImageTool;
#[async_trait]
impl AgentTool for GenerateImageTool {
    fn name(&self) -> String { "generate_image".to_string() }
    fn description(&self) -> String { "Generates an image based on the prompt. (prompt: string, file_name: string, replace_existing: bool?)".to_string() }
    async fn call(&self, input: serde_json::Value, ctx: &AppContext, session_id: Uuid) -> anyhow::Result<String> {
        let file_name = input["file_name"].as_str().unwrap_or("generated_image.png");
        let prompt = input["prompt"].as_str().ok_or_else(|| anyhow::anyhow!("Missing prompt"))?;
        let replace_existing = input["replace_existing"].as_bool().unwrap_or(false);
        let id = generate_image(prompt, file_name, session_id, ctx, replace_existing).await?;
        Ok(format!("Success: Image '{}' generated. ID: {}.", file_name, id))
    }
}

pub fn get_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(ListFilesTool);
    registry.register(ExcelSheetsTool);
    registry.register(ExcelToCsvTool);
    registry.register(ReadFileTool);
    registry.register(CreateExcelTool);
    registry.register(CreateWordDocTool);
    registry.register(CreatePdfDocTool);
    registry.register(CreateTextFileTool);
    registry.register(DownloadFromUrlTool);
    registry.register(GenerateImageTool);
    registry
}

fn parse_agent_response(response_text: &str) -> Option<(String, serde_json::Value, String)> {
    let action_regex = Regex::new(r"(?i)(?m)^\s*Action:\s*(?P<action>[\w_]+)\s*$").unwrap();

    if let Some(caps) = action_regex.captures(response_text) {
        let action = caps.name("action").unwrap().as_str().trim().to_lowercase();
        let match_end = caps.get(0).unwrap().end();

        let remainder = &response_text[match_end..];

        // Find "Action Input:"
        // We use case-insensitive search logic manually or via regex if needed, but "Action Input:" is the standard prompts
        let input_marker = "Action Input:";
        let input_marker_lower = "action input:";

        let idx_opt = remainder.find(input_marker).or_else(|| remainder.find(input_marker_lower));

        if let Some(idx) = idx_opt {
             let json_start = idx + input_marker.len();
             let json_candidate = &remainder[json_start..];
             let json_candidate_trimmed = json_candidate.trim_start();
             let trim_offset = json_candidate.len() - json_candidate_trimmed.len();
             let absolute_json_start = match_end + json_start + trim_offset;

             // Find end of JSON
             let keywords = ["Action:", "Final Answer:", "Thought:", "Observation:", "---"];
             let mut end_idx = json_candidate_trimmed.len();

             for kw in keywords {
                 if let Some(kw_idx) = json_candidate_trimmed.find(kw) {
                     if kw_idx < end_idx {
                         end_idx = kw_idx;
                     }
                 }
             }

             let json_str = &json_candidate_trimmed[..end_idx].trim();

             // Clean markdown
             let input_clean = json_str.trim_matches('`').trim();
             let input_clean = if input_clean.starts_with("json") {
                &input_clean[4..]
             } else {
                input_clean
             }.trim();

             if let Ok(val) = serde_json::from_str(input_clean) {
                 // Construct truncated history string:
                 // Up to match_end + json_start + trim_offset + end_idx
                 let processed_end = absolute_json_start + end_idx;
                 let processed_text = response_text[..processed_end].to_string();
                 return Some((action, val, processed_text));
             }
        }
    }
    None
}

pub async fn run_agent_cycle(
    ctx: &AppContext,
    session_id: Uuid,
    user_query: &str,
    api_key: &str,
    blobs: Vec<(String, String)>,
    registry: &ToolRegistry,
) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:generateContent?key={}",
        api_key
    );

    let blobs_str = blobs.iter()
        .map(|(id, name)| format!("- {} (ID: {})", name, id))
        .collect::<Vec<_>>()
        .join("\n");

    let tool_descriptions = registry.get_tool_descriptions();

    let system_prompt = format!(r##"You are an industrial data assistant specialized in Tech Transfer and processing technical documents (BOMs, SOPs, Reports).

Available files in this session:
{}

GUIDELINES:
1. You are an AGENT running in a ReAct (Reasoning + Acting) loop. 
2. You must achieve the user's goal by using the available tools.
3. You cannot "see" file contents directly. You MUST use tools like `read_file` (for PDF, DOCX, Text) or `excel_to_csv` (for Excel) to inspect them.
4. IMAGE GENERATION: Use `generate_image` to create technical illustrations, flowcharts, or visual aids. 
   - When you call `generate_image`, the tool returns an ID (UUID) in the observation.
   - You can then use this ID as the `image_id` argument in `create_word_doc` to embed that image into the document.
5. UPDATING FILES: If you need to update a file that might already exist (e.g. "metadata.json" or a draft report), always set `replace_existing: true` in the tool arguments. This prevents creating duplicate files with the same name.
6. If the user asks for a diagram (e.g. Mermaid), you can include the Mermaid code in your response or in a generated text/markdown file.
7. MANDATORY OUTPUTS:
   - Follow the specific instructions provided in the user's current request.
   - If the user asks for files, create them.
   - If the user asks for analysis, provide it.
   - **ACTION BIAS**: If the request is vague, MAKE REASONABLE ASSUMPTIONS and proceed with generation. Do not stop to ask for clarification unless absolutely necessary.
8. COMPOSING DOCUMENTS: You can use multiple tools in sequence. For example, read session files -> analyze data -> `generate_image` for a visual -> `create_word_doc` using the generated image ID.
9. USE `create_word_doc` for ALL formal documents (Reports, Proposals, Instructions). DO NOT use `create_text_file` for these; use Word (.docx).
   - For COMPLEX documents, especially those with TABLES, you MUST use the JSON DSL format serialized as a string in `content`.
   - JSON DSL Structure: `[ {{ "type": "heading", "level": 1, "text": "Title" }}, {{ "type": "paragraph", "text": "Text with **bold**." }}, {{ "type": "table", "headers": ["H1", "H2"], "rows": [["R1C1", "R1C2"]] }} ]`
   - Only use plain Markdown for very simple text-only documents.


RESPONSE FORMAT:
You MUST format your output as follows.

To use a tool:
Thought: I need to [reasoning]
Action: [tool_name]
Action Input: {{ [valid_json_arguments] }}

To provide the final response to the user:
Final Answer: [your response]

IMPORTANT:
1. "Action:" must match a tool name exactly (e.g. `read_file`).
2. "Action Input:" must be a valid JSON object. Do not add markdown backticks.
3. When you provide a "Final Answer", the agent loop stops. Ensure you have completed the task.

TOOLS:
{}

Begin!
"##, blobs_str, tool_descriptions);

    let messages = messages::Entity::find()
        .filter(messages::Column::SessionId.eq(session_id))
        .order_by_asc(messages::Column::CreatedAt)
        .all(&ctx.db)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch messages: {}", e))?;

    let mut history: Vec<GeminiContent> = Vec::new();

    // 1. System Prompt
    history.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart {
            text: Some(system_prompt),
        }],
    });

    // 2. Model Acknowledgement
    history.push(GeminiContent {
        role: "model".to_string(),
        parts: vec![GeminiPart {
            text: Some("Understood. I will interpret the user's request and use the tools step-by-step.".to_string()),
        }],
    });

    // 3. Chat Context
    for msg in messages {
        let role = if msg.role == "assistant" { "model" } else { "user" };
        history.push(GeminiContent {
            role: role.to_string(),
            parts: vec![GeminiPart {
                text: Some(msg.content),
            }],
        });
    }

    // Ensure the current user query is at the end of history
    // This handles cases where the DB fetch missed the latest insert or if we want to ensure the prompt is active
    let last_content_matches = history.last()
        .map(|m| m.role == "user" && m.parts.first().and_then(|p| p.text.as_ref()) == Some(&user_query.to_string()))
        .unwrap_or(false);

    if !last_content_matches {
        println!("Appended missing user query to history (DB fetch lag or mismatch).");
        history.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: Some(user_query.to_string()),
            }],
        });
    }

    println!("Agent History Length: {}. Last role: {:?}", history.len(), history.last().map(|m| &m.role));

    for cycle in 0..25 {
        println!("Cycle {}/25...", cycle + 1);
        let request = GeminiRequest {
            contents: history.clone(),
            generation_config: Some(GenerationConfig {
                stop_sequences: vec!["Observation:".to_string()],
                max_output_tokens: Some(4096),
                temperature: Some(0.0), // constant 0 temp for deterministic tool use
            }),
        };

        let mut retry_count = 0;
        let response_res = loop {
            let res = client.post(&url)
                .json(&request)
                .send()
                .await?;

            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if retry_count < 3 {
                    retry_count += 1;
                    println!("Gemini API 429 Too Many Requests. Retrying {}/3 in 10 seconds...", retry_count);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            }
            break res;
        };

        let status = response_res.status();
        let response_text: String = response_res.text().await?;
        
        if !status.is_success() {
            return Err(anyhow::anyhow!("Gemini API error ({}): {}", status, response_text));
        }

        let response: GeminiResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse Gemini response: {}. Body: {}", e, response_text))?;

        let ai_text = response.candidates.get(0)
            .and_then(|c| c.content.parts.get(0))
            .and_then(|p| p.text.clone())
            .ok_or_else(|| anyhow::anyhow!("Empty response from Gemini. Body: {}", response_text))?;

        // Cleanup: Trim whitespace
        let ai_text_clean = ai_text.trim();
        println!("AI Thought: {}", ai_text_clean);
        
        // 0. Check for Final Answer explicitly
        if let Some(pos) = ai_text_clean.find("Final Answer:") {
           history.push(GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart {
                    text: Some(ai_text_clean.to_string()),
                }],
            });
           let answer = ai_text_clean[pos + 13..].trim().to_string();
           return Ok(answer);
        }

        // Parsing Logic
        if let Some((action, input_val, processed_text)) = parse_agent_response(ai_text_clean) {
            // Push only the processed text (action + input) to history
            // This ensures that if there were multiple actions, subsequent ones are "forgotten"
            // and the agent will re-generate them in the next turn.
            history.push(GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart {
                    text: Some(processed_text),
                }],
            });

            let observation = if let Some(tool) = registry.tools.get(&action) {
                match tool.call(input_val, ctx, session_id).await {
                    Ok(res) => res,
                    Err(e) => format!("Error calling tool {}: {}", action, e),
                }
            } else {
                format!("Error: Unknown action '{}'. Available tools are: {:?}", action, registry.tools.keys().collect::<Vec<_>>())
            };

            println!("Observation: {}", observation);

            history.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: Some(format!("Observation: {}", observation)),
                }],
            });

        } else {
             // If parsing fails but we didn't return above (no Final Answer), we log the whole text
             history.push(GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart {
                    text: Some(ai_text_clean.to_string()),
                }],
            });

            if ai_text_clean.contains("Action:") || ai_text_clean.contains("action:") {
             // Loose match for Action but regex failed (likely formatting)
             let observation = "Error: I detected 'Action:' but the format was incorrect. usage:\nAction: <tool_name>\nAction Input: <json>";
             history.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: Some(format!("Observation: {}", observation)),
                }],
            });
            continue;
        } else {
            // No action, No Final Answer (but maybe implicit).
            // If the model just outputs thought/chat without Action or Final Answer, we force it to conform.
             let observation = "System Warning: You did not provide a 'Final Answer:' or an 'Action:'. Please format your response strictly. If you are done or asking a question, use 'Final Answer:'.";
             history.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: Some(format!("Observation: {}", observation)),
                }],
            });
            continue;
        }
        } // End else
    }

    Ok("Terminated: Maximum agent cycles reached without Final Answer.".to_string())
}

pub async fn process_session_queue(ctx: AppContext, session_id: Uuid, registry: ToolRegistry) -> anyhow::Result<()> {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::models::_entities::{sessions, messages};

    println!("Processing queue for session {}", session_id);
    
    // Fetch session
    let session = sessions::Entity::find_by_id(session_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
    let tasks_json = session.pending_tasks.clone();
    let mut tasks: Vec<String> = serde_json::from_value(tasks_json)?;
    
    if tasks.is_empty() {
        // Mark complete
        let mut active: sessions::ActiveModel = session.into();
        active.status = Set("completed".to_string());
        active.update(&ctx.db).await?;
        return Ok(());
    }
    
    let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    
    // Loop
    while let Some(task) = tasks.first().cloned() {
        // Re-fetch session to check for cancellation
        let current_session = sessions::Entity::find_by_id(session_id)
            .one(&ctx.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        if current_session.status != "processing" {
            println!("Session {} execution halted (status: {})", session_id, current_session.status);
            return Ok(());
        }

        println!("Processing task: {}", task);
        
        let blobs = crate::models::_entities::blobs::Entity::find()
            .filter(crate::models::_entities::blobs::Column::SessionId.eq(session_id))
            .all(&ctx.db)
            .await?;
        let blobs_context: Vec<(String, String)> = blobs.into_iter()
            .map(|b| (b.id.to_string(), b.file_name))
            .collect();

        // Save USER message for this task
        let user_msg = messages::ActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            role: Set("user".to_string()),
            content: Set(task.clone()),
            ..Default::default()
        };
        user_msg.insert(&ctx.db).await?;

        // Run Agent
        let result = match run_agent_cycle(&ctx, session_id, &task, &api_key, blobs_context, &registry).await {
            Ok(res) => res,
            Err(e) => {
                let error_msg = format!("Error executing task: {}", e);
                // Save Error message
                let asst_msg = messages::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    session_id: Set(session_id),
                    role: Set("assistant".to_string()),
                    content: Set(error_msg),
                    ..Default::default()
                };
                asst_msg.insert(&ctx.db).await?;

                // Set status to error and stop
                let active_update = sessions::ActiveModel {
                    id: Set(session_id),
                    status: Set("error".to_string()),
                    ..Default::default()
                };
                active_update.update(&ctx.db).await?;
                return Ok(());
            }
        };
        
        // Save ASSISTANT message
        let asst_msg = messages::ActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            role: Set("assistant".to_string()),
            content: Set(result),
            ..Default::default()
        };
        asst_msg.insert(&ctx.db).await?;
        
        // Remove task from queue only if SUCCESS
        tasks.remove(0); 
        
        // Update session
        let mut active_update = sessions::ActiveModel {
            id: Set(session_id),
            pending_tasks: Set(serde_json::to_value(&tasks)?),
            ..Default::default()
        };
        
        if tasks.is_empty() {
            active_update.status = Set("completed".to_string());
        } else {
            active_update.status = Set("processing".to_string());
        }
        
        active_update.update(&ctx.db).await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests;
