//! Generic Gemma Client
//! 
//! HTTP client for Gemini API that can work with any model tier.
//! Preserves the exact prompt structure and streaming behavior from the original implementation.

use parking_lot::Mutex;
use std::sync::Arc;
use reqwest::Client;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;

use crate::agent::memory::Message;
use crate::providers::ModelTier;

/// API Key Manager with rotation support
#[derive(Clone)]
pub struct KeyManager {
    keys: Vec<String>,
    current_index: Arc<Mutex<usize>>,
}

impl KeyManager {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            keys,
            current_index: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_current_key(&self) -> String {
        let idx = *self.current_index.lock();
        self.keys.get(idx).cloned().unwrap_or_default()
    }

    pub fn rotate(&self) {
        if self.keys.is_empty() { return; }
        let mut idx = self.current_index.lock();
        *idx = (*idx + 1) % self.keys.len();
        println!("[KeyManager] Rotated to API Key Index: {}", *idx);
    }
}

/// Response structures for JSON parsing
#[derive(Debug, Deserialize)]
struct StreamResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<Content>,
    #[serde(rename = "finishReason")]
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Content {
    parts: Option<Vec<Part>>,
    #[allow(dead_code)]
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Part {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: Option<i32>,
    message: Option<String>,
}

/// Gemma Client for a specific model tier
#[derive(Clone)]
pub struct GemmaClient {
    http: Client,
    key_manager: KeyManager,
    model_tier: ModelTier,
}

impl GemmaClient {
    pub fn new(keys: Vec<String>, tier: ModelTier) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            key_manager: KeyManager::new(keys),
            model_tier: tier,
        }
    }
    
    pub fn tier(&self) -> ModelTier {
        self.model_tier
    }
    
    /// Stream a completion - PRESERVED from original implementation
    /// 
    /// # Arguments
    /// * `system_prompt` - The system instruction
    /// * `messages` - Conversation history
    /// * `is_turbo` - Whether this is turbo/agent mode (affects temperature)
    pub async fn stream_completion(
        &self,
        system_prompt: &str,
        messages: &[Message],
        is_turbo: bool,
    ) -> Result<impl futures::Stream<Item = Result<String, String>>, String> {
        let model_name = self.model_tier.model_name();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse",
            model_name
        );
        let key = self.key_manager.get_current_key();
        let url_with_key = format!("{}&key={}", url, key);

        // Build contents array - EXACT same structure as original
        let mut contents: Vec<Value> = Vec::new();

        // System prompt as first user message
        contents.push(serde_json::json!({
            "role": "user",
            "parts": [{ "text": format!("SYSTEM INSTRUCTION:\n{}\n\nCONFIRM YOU UNDERSTAND BY ACKNOWLEDGING.", system_prompt) }]
        }));
        
        // Model acknowledgment
        contents.push(serde_json::json!({
            "role": "model",
            "parts": [{ "text": "I understand the system instructions. I am ready to act as the AI coding agent." }]
        }));

        // Append conversation history - MAP ROLES to valid Gemma roles (user/model only)
        for msg in messages {
            // Gemma only supports "user" and "model" roles
            // Map: user -> user, model/assistant -> model
            // IMPORTANT: Tool results should be "user" so model sees them as observations
            let gemma_role = match msg.role.as_str() {
                "user" | "tool" => "user",  // Tool results are observations from user perspective
                "model" | "assistant" => "model",
                _ => "user", // Fallback to user for any unknown role
            };
            
            contents.push(serde_json::json!({
                "role": gemma_role,
                "parts": [{ "text": &msg.content }]
            }));
        }

        // Build request config - Gemma 3 doesn't support JSON mode, parser handles JSON extraction
        let generation_config = serde_json::json!({
            "temperature": if is_turbo { 0.4 } else { 0.8 },
            "topP": 0.95,
            "topK": 40,
            "maxOutputTokens": 8192
        });
        
        let request_body = serde_json::json!({
            "contents": contents,
            "generationConfig": generation_config,
            "safetySettings": [
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE" }
            ]
        });

        println!("[GemmaClient] {} sending request with {} messages", 
            self.model_tier.display_name(), messages.len());

        let resp = self.http.post(&url_with_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            
            if status.as_u16() == 429 {
                self.key_manager.rotate();
                return Err("RATE_LIMIT".to_string());
            }
            
            return Err(format!("API Error {}: {}", status, body));
        }

        // Process SSE stream - EXACT same parsing as original
        let stream = resp.bytes_stream().map(|chunk_result| {
            match chunk_result {
                Ok(bytes) => {
                    let s = String::from_utf8_lossy(&bytes).to_string();
                    let mut text_parts = Vec::new();
                    
                    for line in s.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if let Ok(response) = serde_json::from_str::<StreamResponse>(json_str) {
                                if let Some(error) = response.error {
                                    return Err(format!("API Error: {} - {}", 
                                        error.code.unwrap_or(0), 
                                        error.message.unwrap_or_default()));
                                }
                                
                                if let Some(candidates) = response.candidates {
                                    for candidate in candidates {
                                        if let Some(content) = candidate.content {
                                            if let Some(parts) = content.parts {
                                                for part in parts {
                                                    if let Some(text) = part.text {
                                                        text_parts.push(text);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    Ok(text_parts.join(""))
                },
                Err(e) => Err(e.to_string())
            }
        });

        Ok(stream)
    }
    
    /// Quick classification (for 1B router) - MINIMAL prompt for small model
    pub async fn classify(&self, input: &str) -> Result<String, String> {
        // Ultra-simple prompt for 1B - just one line
        let short_input: String = input.chars().take(100).collect();
        let prompt = format!("Classify as SIMPLE or COMPLEX: \"{}\"", short_input);
        
        let model_name = self.model_tier.model_name();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model_name
        );
        let key = self.key_manager.get_current_key();
        let url_with_key = format!("{}?key={}", url, key);
        
        // Minimal request - no system prompt, low tokens
        let request_body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "temperature": 0.0,
                "maxOutputTokens": 5,
            }
        });
        
        let resp = self.http.post(&url_with_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Router request failed: {}", e))?;
        
        if !resp.status().is_success() {
            return Err("Router API error".to_string());
        }
        
        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        
        let text = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("SIMPLE")
            .trim()
            .to_uppercase();
        
        Ok(text)
    }
    
    /// Summarize messages (for 2B summarizer) - MINIMAL prompt for small model
    pub async fn summarize(&self, messages: &[Message]) -> Result<String, String> {
        // Take only last 5 messages, truncate each heavily
        let history: String = messages.iter()
            .rev()
            .take(5)
            .rev()
            .map(|m| {
                let short_content: String = m.content.chars().take(100).collect();
                format!("{}: {}", m.role, short_content)
            })
            .collect::<Vec<_>>()
            .join(" | ");
        
        // Ultra-simple prompt for 2B
        let prompt = format!("Summarize in 2 sentences: {}", history);
        
        let model_name = self.model_tier.model_name();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model_name
        );
        let key = self.key_manager.get_current_key();
        let url_with_key = format!("{}?key={}", url, key);
        
        // Minimal config for small model
        let request_body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 100,
            }
        });
        
        let resp = self.http.post(&url_with_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Summarizer request failed: {}", e))?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            println!("[Summarizer] API Error {}: {}", status, error_text);
            return Err(format!("Summarizer API error: {}", status));
        }
        
        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        
        let text = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();
        
        Ok(text)
    }
}
