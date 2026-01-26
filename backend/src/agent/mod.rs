pub mod tools;

use loco_rs::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::agent::tools::{excel_to_csv, create_excel, list_files, get_excel_sheets, create_text_file, download_from_url};

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
    user_query: &str,
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

    let system_prompt = format!(r#"You are an industrial data assistant specialized in the tech transfer process and making sense of loose files.

GUIDELINES:
1. Identify target files from the 'Available files' list below using their IDs.
2. ALWAYS use `excel_to_csv(blob_id, sheet_name)` to read the contents of an Excel file before attempting to process it. You cannot "see" the file otherwise.
3. If you need to know the sheet names first, use `get_excel_sheets(blob_id)`.
4. After reading the context, process the data internally.
5. To generate a result, use `create_excel(file_name, rows)` or `create_text_file(file_name, content)`.
6. IMPORTANT: The tool returns a NEW blob ID. Mention this ID in your Final Answer so the user knows a new file was generated.

TOOLS:
1. list_files(): Lists all files available in the current session.
2. get_excel_sheets(blob_id: string): Returns a list of sheet names in an Excel file.
3. excel_to_csv(blob_id: string, sheet_name: string?): Converts a specific sheet of an Excel file to CSV text.
4. create_excel(file_name: string, rows: string[][]): Creates a new Excel file and saves it to the session.
5. create_text_file(file_name: string, content: string): Saves text content (like CSV, notes, or code) as a file in the session.
6. download_from_url(url: string, file_name: string): Downloads a file from a URL and saves it to the session.

Respond to the user requests and generate files as necessary to help them.

Available files:
{}

Begin!"#, blobs_str);

    let mut history: Vec<GeminiContent> = vec![
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: Some(format!("{}\n\nQuestion: {}", system_prompt, user_query)),
            }],
        }
    ];

    for _ in 0..10 { // Increased cycles for more complex tasks
        let request = GeminiRequest {
            contents: history.clone(),
            generation_config: Some(GenerationConfig {
                stop_sequences: vec!["Observation:".to_string()],
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

        println!("AI Thought: {}", ai_text);
        
        history.push(GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart {
                text: Some(ai_text.clone()),
            }],
        });

        if ai_text.contains("Final Answer:") {
            return Ok(ai_text);
        }

        let action_line_opt = ai_text.lines().find(|l: &&str| l.starts_with("Action:"));
        let input_line_opt = ai_text.lines().find(|l: &&str| l.starts_with("Action Input:"));

        if let (Some(action_line), Some(input_line)) = (action_line_opt, input_line_opt) {
            let full_action = action_line.replace("Action:", "").trim().to_string();
            // Fuzzy match: take the first word as the action name
            let action = full_action.split_whitespace().next().unwrap_or("").to_string();
            
            let input_str_owned = input_line.replace("Action Input:", "");
            let input_str = input_str_owned.trim();
            let input_val: serde_json::Value = serde_json::from_str(input_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse Action Input JSON: {}. Input was: {}", e, input_str))?;

            let observation = match action.as_str() {
                "list_files" => {
                    list_files(session_id, ctx).await?
                }
                "get_excel_sheets" => {
                    let blob_id_str = input_val["blob_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing blob_id"))?;
                    let blob_id = Uuid::parse_str(blob_id_str)?;
                    let sheets = get_excel_sheets(blob_id, ctx).await?;
                    format!("Sheets: {:?}", sheets)
                }
                "excel_to_csv" => {
                    let blob_id_str = input_val["blob_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing blob_id"))?;
                    let blob_id = Uuid::parse_str(blob_id_str)?;
                    let sheet_name = input_val["sheet_name"].as_str().map(|s| s.to_string());
                    excel_to_csv(blob_id, sheet_name, ctx).await?
                }
                "create_excel" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("output.xlsx");
                    let rows_val = input_val["rows"].as_array().ok_or_else(|| anyhow::anyhow!("Missing rows"))?;
                    let mut rows = Vec::new();
                    for row_val in rows_val {
                        let row: Vec<String> = row_val.as_array().unwrap().iter().map(|v| v.as_str().unwrap_or("").to_string()).collect();
                        rows.push(row);
                    }
                    let new_blob_id = create_excel(file_name, rows, session_id, ctx).await?;
                    format!("New Excel file created successfully. File Name: {}, ID: {}. Inform the user they can download this file now.", file_name, new_blob_id)
                }
                "create_text_file" => {
                    let file_name = input_val["file_name"].as_str().unwrap_or("output.txt");
                    let content = input_val["content"].as_str().ok_or_else(|| anyhow::anyhow!("Missing content"))?;
                    let new_blob_id = create_text_file(file_name, content, session_id, ctx).await?;
                    format!("New text file created successfully. File Name: {}, ID: {}. Inform the user they can download this file now.", file_name, new_blob_id)
                }
                "download_from_url" => {
                    let url = input_val["url"].as_str().ok_or_else(|| anyhow::anyhow!("Missing url"))?;
                    let file_name = input_val["file_name"].as_str().unwrap_or("downloaded_file");
                    let new_blob_id = download_from_url(url, file_name, session_id, ctx).await?;
                    format!("File downloaded successfully. File Name: {}, ID: {}. Inform the user they can download this file now.", file_name, new_blob_id)
                }
                _ => format!("Unknown action: {}", action),
            };

            history.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: Some(format!("Observation: {}", observation)),
                }],
            });
        } else {
            // If no action and no final answer, the AI is likely just chatting or asking a clarifying question.
            // valid response for the user.
            return Ok(ai_text);
        }
    }

    // This part should theoretically be unreachable now if we return in all paths, 
    // but just in case the loop finishes exactly at 10 without return:
    Err(anyhow::anyhow!("Agent loop limit exceeded"))
}
