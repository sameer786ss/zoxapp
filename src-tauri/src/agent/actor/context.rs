//! Context Management Module
//!
//! Handles RAG initialization, message storage, and context optimization.

use std::sync::Arc;
use tauri::Manager;

use crate::agent::memory::{ContextWindow, Message};
use crate::agent::rag::ContextManager;
use crate::agent::history::Conversation;

use super::core::AgentActor;

impl AgentActor {
    /// Initialize RAG system (called lazily on first use)
    pub async fn ensure_rag(&mut self) -> Option<Arc<ContextManager>> {
        if self.rag.is_some() {
            return self.rag.clone();
        }
        
        if let Ok(data_dir) = self.app_handle.path().app_data_dir() {
            let rag_dir = data_dir.join("rag");
            if std::fs::create_dir_all(&rag_dir).is_ok() {
                match ContextManager::new(&rag_dir).await {
                    Ok(cm) => {
                        tracing::info!("RAG initialized");
                        let arc = Arc::new(cm);
                        self.rag = Some(arc.clone());
                        return Some(arc);
                    }
                    Err(e) => {
                        tracing::warn!("RAG init failed: {}", e);
                    }
                }
            }
        }
        
        None
    }
    
    /// Store a message in both context window and RAG
    pub async fn store_message(&mut self, role: &str, content: &str) {
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
        };
        
        self.context.add_message(msg);
        
        if let Some(rag) = &self.rag {
            let _ = rag.store_message(&self.conversation_id, role, content).await;
        }
    }
    
    /// Store a tool result in RAG
    pub async fn store_tool_result(&mut self, tool_name: &str, result: &str) {
        if let Some(rag) = &self.rag {
            let _ = rag.store_tool_result(tool_name, result).await;
        }
    }
    
    /// Add a message to both context and conversation history, then save
    pub fn save_and_persist_message(&mut self, role: &str, content: &str) {
        self.context.add_message(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        
        self.current_conversation.add_message(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        
        if let Err(e) = self.history_manager.save_conversation(&self.current_conversation) {
            tracing::error!("Failed to save conversation: {}", e);
        }
    }
    
    /// Start a new conversation
    pub fn start_new_conversation(&mut self, mode: &str) {
        self.context = ContextWindow::new(self.config.context_window_size);
        self.current_conversation = Conversation::new(mode);
        self.conversation_id = self.current_conversation.id.clone();
        self.streaming_parser.reset();
        tracing::info!(id = %self.conversation_id, "Started new conversation");
    }
    
    /// Build optimized messages using RAG semantic search
    pub async fn build_optimized_messages(&mut self) -> Vec<Message> {
        let msg_count = self.context.len();
        
        // If few messages, send all
        if msg_count <= 12 {
            return self.context.get_history().to_vec();
        }
        
        // Get current query for RAG search
        let current_query = self.context.get_history()
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        
        let mut optimized = Vec::new();
        
        // Try RAG search
        if let Some(rag) = self.ensure_rag().await {
            if let Ok(relevant) = rag.search(&current_query, 5).await {
                if !relevant.is_empty() {
                    optimized.push(Message {
                        role: "user".to_string(),
                        content: format!("[Relevant Context]\n{}\n[End Context]", relevant.join("\n---\n")),
                    });
                    optimized.push(Message {
                        role: "model".to_string(),
                        content: "I've reviewed the relevant context.".to_string(),
                    });
                    tracing::debug!(chunks = relevant.len(), "Added RAG chunks to context");
                }
            }
        }
        
        // Always include last 8 messages
        let all_messages = self.context.get_history();
        let recent: Vec<Message> = all_messages.iter()
            .rev()
            .take(8)
            .rev()
            .cloned()
            .collect();
        
        optimized.extend(recent);
        
        tracing::debug!(from = msg_count, to = optimized.len(), "Context optimized");
        
        optimized
    }
    
    /// Get or create context summary
    pub async fn get_or_create_summary(&mut self) -> Option<String> {
        use crate::providers::cloud::GemmaClient;
        use crate::providers::ModelTier;
        
        if !self.provider.capabilities().supports_summarization {
            return None;
        }
        
        if self.context.len() < 6 {
            return self.context_summary.clone();
        }
        
        if self.context_summary.is_some() && self.context.len() < 10 {
            return self.context_summary.clone();
        }
        
        tracing::info!("Generating context summary");
        let _ = self.app_handle.emit("context-summary-pending", true);
        
        let all_messages = self.context.get_history();
        let to_summarize = if all_messages.len() > 3 {
            &all_messages[..all_messages.len() - 3]
        } else {
            return None;
        };
        
        let summarizer = GemmaClient::new(self.api_keys.clone(), ModelTier::Summarizer);
        
        match summarizer.summarize(to_summarize).await {
            Ok(summary) => {
                tracing::info!(len = summary.len(), "Summary generated");
                self.context_summary = Some(summary.clone());
                let _ = self.app_handle.emit("context-summary", summary.clone());
                let _ = self.app_handle.emit("context-summary-pending", false);
                Some(summary)
            }
            Err(e) => {
                tracing::warn!("Summarization failed: {}", e);
                let _ = self.app_handle.emit("context-summary-pending", false);
                None
            }
        }
    }
}
