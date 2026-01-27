pub mod tools;
pub mod docx;

use loco_rs::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, ColumnTrait};
use crate::models::_entities::messages;
use crate::agent::tools::{excel_to_csv, create_excel, list_files, get_excel_sheets, create_text_file, download_from_url, read_file, create_word_doc, create_pdf_doc, generate_image, search_internet};


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

pub async fn run_agent_cycle(
    ctx: &AppContext,
    session_id: Uuid,
    _user_query: &str,
    api_key: &str,
    blobs: Vec<(String, String)>,
) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-lite:generateContent?key={}",
        api_key
    );

    let blobs_str = blobs.iter()
        .map(|(id, name)| format!("- {} (ID: {})", name, id))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = format!(r##"You are an industrial data assistant specialized in Tech Transfer and processing technical documents (BOMs, SOPs, Reports).

Available files in this session:
{}

GUIDELINES:
1. You are an AGENT running in a ReAct (Reasoning + Acting) loop.
2. You must achieve the user's goal by using the available tools.
3. You cannot "see" file contents directly. You MUST use tools like `read_file` (for PDF, DOCX, Text) or `excel_to_csv` (for Excel) to inspect them.
4. When you create a result file, you MUST output a "Final Answer" telling the user the file name and that it is ready.
5. If the user asks for a diagram (e.g. Mermaid), you can include the Mermaid code in your response or in a generated text/markdown file.
6. MANDATORY OUTPUTS (unless explicitly told otherwise):
   - ALWAYS generate a CSV summary of any extracted data lists (e.g., BOMs, Part Lists).
   - ALWAYS generate at least 1-2 visual diagrams or illustrations using `generate_image`, related to the subject matter.
   - ALWAYS create a final Word report (`Summary_Report.docx`) aggregating the key findings, data, and context.
7. Use `generate_image` liberally.
8. USE `create_word_doc` for ALL formal documents (Reports, Proposals, Instructions). DO NOT use `create_text_file` for these; use Word (.docx).
   - For COMPLEX documents, especially those with TABLES, you MUST use the JSON DSL format serialized as a string in `content`.
   - JSON DSL Structure: `[ {{ "type": "heading", "level": 1, "text": "Title" }}, {{ "type": "paragraph", "text": "Text with **bold**." }}, {{ "type": "table", "headers": ["H1", "H2"], "rows": [["R1C1", "R1C2"]] }} ]`
   - Only use plain Markdown for very simple text-only documents.
9. Return the result files when ready.


RESPONSE FORMAT:
You MUST format your output strictly as follows:

Thought: [Your reasoning about what to do next]
Action: [The exact name of the tool to use]
Action Input: [A valid JSON object containing the arguments, and ONLY the JSON object]

OR, if you are done:

Final Answer: [Your response to the user, summarizing what you did]

TOOLS:
1. list_files(): Lists all files available in the current session.
2. get_excel_sheets(blob_id: string): Returns a list of sheet names in an Excel file.
3. excel_to_csv(blob_id: string, sheet_name: string?): Converts a specific sheet of an Excel file to CSV text.
4. read_file(blob_id: string): Reads the text content of a file (PDF, DOCX, or Text).
5. create_excel(file_name: string, rows: string[][]): Creates a new Excel file. 'rows' must be a 2D array of strings.
6. create_word_doc(file_name: string, content: string, image_id: string?): Creates a new Word document. optionally embedding an image by ID.
7. create_pdf_doc(file_name: string, content: string): Creates a new PDF document.
8. create_text_file(file_name: string, content: string): Creates a text file.
9. download_from_url(url: string, file_name: string): Downloads a file from a URL.
10. generate_image(prompt: string, file_name: string): Generates an image based on the prompt.
11. search_internet(query: string): Searches the internet for information about parts, components, or items.

EXAMPLES:

Example 1 (Checking a file):
Thought: I need to read the SOP document to understand the process.
Action: read_file
Action Input: {{ "blob_id": "uuid-of-file" }}

Example 2 (Creating result with Tables):
Thought: I need to create a report with a table of data. I will use the JSON DSL for `create_word_doc` to ensure the table is formatted correctly.
Action: create_word_doc
Action Input: {{ "file_name": "Data_Summary.docx", "content": "[{{\"type\":\"heading\",\"level\":1,\"text\":\"Data Summary\"}},{{\"type\":\"paragraph\",\"text\":\"Here is the extracted data:\"}},{{\"type\":\"table\",\"headers\":[\"Item\",\"Value\"],\"rows\":[[\"Part A\",\"10\"],[\"Part B\",\"20\"]]}}]" }}

Example 3 (Done):
Final Answer: I have created the report 'Tech_Transfer_Report.docx'.

Begin!
"##, blobs_str);

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

    // Regex for parsing using multi-line mode
    // Captures "Action: <name>" and "Action Input: <json>"
    let action_regex = Regex::new(r"(?m)^Action:\s*(?P<action>\w+)\s*$").unwrap();
    // Input regex tries to capture the JSON blob following Action Input:
    // We'll look for "Action Input:" and then take everything until the end or next observation/stop.
    // However, simplest is to split by "Action Input:" and parse the remainder.
    
    for cycle in 0..15 {
        println!("Cycle {}/15...", cycle + 1);
        let request = GeminiRequest {
            contents: history.clone(),
            generation_config: Some(GenerationConfig {
                stop_sequences: vec!["Observation:".to_string()],
                max_output_tokens: Some(4096),
                temperature: Some(0.0), // constant 0 temp for deterministic tool use
            }),
        };

        let response_res = client.post(&url)
            .json(&request)
            .send()
            .await?;
        
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
        
        history.push(GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart {
                text: Some(ai_text_clean.to_string()),
            }],
        });

        if ai_text_clean.contains("Final Answer:") {
            return Ok(ai_text_clean.to_string());
        }

        // Parsing Logic
        // 1. Find Action
        if let Some(caps) = action_regex.captures(ai_text_clean) {
            let action = caps.name("action").unwrap().as_str().to_string();
            
            // 2. Find Input
            // We split manually because regex across newlines for unknown JSON content is tricky
            let parts: Vec<&str> = ai_text_clean.split("Action Input:").collect();
            if parts.len() < 2 {
                // Found Action but no Input
                let observation = "Error: Found 'Action:' but missing 'Action Input:'. Please provide the arguments in JSON format.";
                 history.push(GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart { text: Some(format!("Observation: {}", observation)) }],
                });
                continue;
            }

            let input_raw = parts.last().unwrap().trim();
            // Remove markdown code blocks if present (```json ... ```)
            let input_clean = input_raw.trim_matches('`').trim();
            let input_clean = if input_clean.starts_with("json") {
                &input_clean[4..]
            } else {
                input_clean
            }.trim();

            let input_val: serde_json::Value = match serde_json::from_str(input_clean) {
                Ok(v) => v,
                Err(e) => {
                     let observation = format!("Error: Failed to parse JSON arguments: {}. Ensure 'Action Input' is valid JSON.", e);
                     history.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart { text: Some(format!("Observation: {}", observation)) }],
                    });
                    continue;
                }
            };

            let observation = match action.as_str() {
                "list_files" => {
                    list_files(session_id, ctx).await?
                }
                "get_excel_sheets" => {
                    if let Some(blob_id_str) = input_val["blob_id"].as_str() {
                         match Uuid::parse_str(blob_id_str) {
                             Ok(blob_id) => {
                                 match get_excel_sheets(blob_id, ctx).await {
                                     Ok(s) => format!("Sheets: {:?}", s),
                                     Err(e) => format!("Error reading sheets: {}", e),
                                 }
                             },
                             Err(_) => "Error: Invalid UUID for blob_id".to_string()
                         }
                    } else {
                        "Error: Missing 'blob_id' in arguments".to_string()
                    }
                }
                "excel_to_csv" => {
                    if let Some(blob_id_str) = input_val["blob_id"].as_str() {
                         match Uuid::parse_str(blob_id_str) {
                             Ok(blob_id) => {
                                 let sheet_name = input_val["sheet_name"].as_str().map(|s| s.to_string());
                                 match excel_to_csv(blob_id, sheet_name, ctx).await {
                                     Ok(res) => res,
                                     Err(e) => format!("Error converting excel: {}", e)
                                 }
                             },
                             Err(_) => "Error: Invalid UUID for blob_id".to_string()
                         }
                    } else {
                        "Error: Missing 'blob_id' in arguments".to_string()
                    }
                }
                "read_file" => {
                    if let Some(blob_id_str) = input_val["blob_id"].as_str() {
                        match Uuid::parse_str(blob_id_str) {
                            Ok(blob_id) => {
                                match read_file(blob_id, ctx).await {
                                    Ok(content) => content,
                                    Err(e) => format!("Error reading file: {}", e),
                                }
                            },
                             Err(_) => "Error: Invalid UUID for blob_id".to_string()
                        }
                    } else {
                        "Error: Missing 'blob_id' in arguments".to_string()
                    }
                }
                "create_excel" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("output.xlsx");
                    if let Some(rows_val) = input_val["rows"].as_array() {
                         let mut rows = Vec::new();
                         for row_val in rows_val {
                             let row: Vec<String> = row_val.as_array().unwrap_or(&vec![]).iter().map(|v| v.as_str().unwrap_or("").to_string()).collect();
                             rows.push(row);
                         }
                         match create_excel(file_name, rows, session_id, ctx).await {
                             Ok(new_id) => format!("Success: New Excel file '{}' created. ID: {}.", file_name, new_id),
                             Err(e) => format!("Error creating excel: {}", e)
                         }
                    } else {
                        "Error: 'rows' must be an array of arrays".to_string()
                    }
                }
                "create_word_doc" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("output.docx");
                    let image_id = input_val["image_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
                    
                    if let Some(content) = input_val["content"].as_str() {
                        match create_word_doc(file_name, content, image_id, session_id, ctx).await {
                            Ok(new_id) => format!("Success: New Word doc '{}' created. ID: {}.", file_name, new_id),
                            Err(e) => format!("Error creating word doc: {}", e)
                        }
                    } else {
                         "Error: Missing 'content' argument".to_string()
                    }
                }
                "create_pdf_doc" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("output.pdf");
                    if let Some(content) = input_val["content"].as_str() {
                        match create_pdf_doc(file_name, content, session_id, ctx).await {
                            Ok(new_id) => format!("Success: New PDF '{}' created. ID: {}.", file_name, new_id),
                            Err(e) => format!("Error creating PDF: {}", e)
                        }
                    } else {
                         "Error: Missing 'content' argument".to_string()
                    }
                }
                "create_text_file" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("output.txt");
                     if let Some(content) = input_val["content"].as_str() {
                         match create_text_file(file_name, content, session_id, ctx).await {
                             Ok(new_id) => format!("Success: New text file '{}' created. ID: {}.", file_name, new_id),
                             Err(e) => format!("Error creating file: {}", e)
                         }
                     } else {
                         "Error: Missing 'content' argument".to_string()
                     }
                }
                "download_from_url" => {
                    if let Some(url) = input_val["url"].as_str() {
                        let file_name = input_val["file_name"].as_str().unwrap_or("downloaded_file");
                        match download_from_url(url, file_name, session_id, ctx).await {
                            Ok(new_id) => format!("Success: File downloaded as '{}'. ID: {}.", file_name, new_id),
                            Err(e) => format!("Error downloading: {}", e)
                        }
                    } else {
                         "Error: Missing 'url' argument".to_string()
                    }
                }
                "generate_image" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("generated_image.png");
                    if let Some(prompt) = input_val["prompt"].as_str() {
                        match generate_image(prompt, file_name, session_id, ctx).await {
                            Ok(new_id) => format!("Success: Image '{}' generated. ID: {}.", file_name, new_id),
                            Err(e) => format!("Error generating image: {}", e)
                        }
                    } else {
                         "Error: Missing 'prompt' argument".to_string()
                    }
                }
                "search_internet" => {
                    if let Some(query) = input_val["query"].as_str() {
                        match search_internet(query).await {
                            Ok(res) => res,
                            Err(e) => format!("Error searching: {}", e),
                        }
                    } else {
                        "Error: Missing 'query' argument".to_string()
                    }
                }
                _ => format!("Error: Unknown action '{}'. Available tools are: list_files, get_excel_sheets, excel_to_csv, read_file, create_excel, create_word_doc, create_pdf_doc, create_text_file, download_from_url, generate_image, search_internet.", action),
            };

            println!("Observation: {}", observation);

            history.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: Some(format!("Observation: {}", observation)),
                }],
            });

        } else {
             // Model didn't output an action. If it didn't output Final Answer either (checked above), it's just chatting.
             // We can just return the text as the final turn.
             return Ok(ai_text_clean.to_string());
        }
    }

    Ok("Terminated: Maximum agent cycles reached without Final Answer.".to_string())
}

pub async fn process_session_queue(ctx: AppContext, session_id: Uuid) -> anyhow::Result<()> {
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
        let result = match run_agent_cycle(&ctx, session_id, &task, &api_key, blobs_context).await {
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
        // Create ActiveModel with just the ID and fields to update
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
        
        // Use update(db) which filters by primary key (id)
        active_update.update(&ctx.db).await?;
    }
    
    Ok(())
}
