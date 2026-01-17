use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs;
use std::path::Path;

use crate::agent::memory::Message;

/// A chat conversation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub mode: String,
}

impl Conversation {
    pub fn new(mode: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: "New Chat".to_string(),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            mode: mode.to_string(),
        }
    }

    /// Generate a title from the first user message
    pub fn generate_title(&mut self) {
        if let Some(first_user_msg) = self.messages.iter().find(|m| m.role == "user") {
            // Take first 50 chars of first user message as title
            let title: String = first_user_msg.content
                .chars()
                .take(50)
                .collect();
            self.title = if title.len() >= 50 {
                format!("{}...", title)
            } else {
                title
            };
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
        
        // Auto-generate title from first user message
        if self.title == "New Chat" {
            self.generate_title();
        }
    }
}

/// Metadata for conversation list (without full messages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMeta {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub mode: String,
}

impl From<&Conversation> for ConversationMeta {
    fn from(conv: &Conversation) -> Self {
        Self {
            id: conv.id.clone(),
            title: conv.title.clone(),
            created_at: conv.created_at,
            updated_at: conv.updated_at,
            message_count: conv.messages.len(),
            mode: conv.mode.clone(),
        }
    }
}

/// Manages chat history persistence
pub struct HistoryManager {
    history_dir: std::path::PathBuf,
}

impl HistoryManager {
    pub fn new(history_dir: &Path) -> Self {
        Self {
            history_dir: history_dir.to_path_buf(),
        }
    }

    /// List all conversations (metadata only)
    pub fn list_conversations(&self) -> Vec<ConversationMeta> {
        let mut conversations = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.history_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(conv) = serde_json::from_str::<Conversation>(&content) {
                            conversations.push(ConversationMeta::from(&conv));
                        }
                    }
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        conversations
    }

    /// Load a conversation by ID
    pub fn load_conversation(&self, id: &str) -> Option<Conversation> {
        let path = self.history_dir.join(format!("{}.json", id));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                return serde_json::from_str(&content).ok();
            }
        }
        None
    }

    /// Save a conversation
    pub fn save_conversation(&self, conversation: &Conversation) -> Result<(), String> {
        let path = self.history_dir.join(format!("{}.json", conversation.id));
        
        let json = serde_json::to_string_pretty(conversation)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(&path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }

    /// Delete a conversation
    pub fn delete_conversation(&self, id: &str) -> Result<(), String> {
        let path = self.history_dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete: {}", e))?;
        }
        Ok(())
    }
}
