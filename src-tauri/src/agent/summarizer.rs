//! Context Summarizer
//! 
//! Combines RAG retrieval with the 2B summarizer to provide
//! compact, relevant context for model prompts.

#![allow(dead_code)]

use crate::agent::memory::Message;
use crate::agent::rag::ContextManager;
use crate::providers::cloud::GemmaClient;
use crate::providers::ModelTier;

/// Context summarizer that uses RAG + 2B model
pub struct ContextSummarizer {
    client: GemmaClient,
}

impl ContextSummarizer {
    pub fn new(api_keys: Vec<String>) -> Self {
        Self {
            client: GemmaClient::new(api_keys, ModelTier::Summarizer),
        }
    }
    
    /// Generate a context summary using RAG-retrieved chunks
    /// 
    /// Flow:
    /// 1. Search RAG for relevant context chunks
    /// 2. Combine with recent messages
    /// 3. Summarize using 2B model
    pub async fn summarize_for_query(
        &self,
        query: &str,
        recent_messages: &[Message],
        rag: Option<&ContextManager>,
    ) -> Result<String, String> {
        let mut context_parts = Vec::new();
        
        // 1. Get RAG context if available
        if let Some(rag_manager) = rag {
            if let Ok(rag_context) = rag_manager.get_relevant_context(query, 3).await {
                if !rag_context.is_empty() {
                    context_parts.push(format!("Relevant history:\n{}", rag_context));
                }
            }
        }
        
        // 2. Add recent messages (last 3)
        let recent: Vec<String> = recent_messages.iter()
            .rev()
            .take(3)
            .rev()
            .map(|m| {
                let short: String = m.content.chars().take(100).collect();
                format!("{}: {}", m.role, short)
            })
            .collect();
        
        if !recent.is_empty() {
            context_parts.push(format!("Recent:\n{}", recent.join("\n")));
        }
        
        // 3. If we have context, summarize it
        if context_parts.is_empty() {
            return Ok(String::new());
        }
        
        let combined = context_parts.join("\n\n");
        
        // Use 2B to summarize
        match self.client.summarize(&self.to_messages(&combined)).await {
            Ok(summary) => Ok(summary),
            Err(_) => {
                // Fallback: just return truncated combined context
                Ok(combined.chars().take(300).collect())
            }
        }
    }
    
    /// Convert context string to messages format for summarizer
    fn to_messages(&self, context: &str) -> Vec<Message> {
        vec![Message {
            role: "user".to_string(),
            content: context.to_string(),
        }]
    }
    
    /// Summarize conversation messages directly (without RAG)
    pub async fn summarize_messages(&self, messages: &[Message]) -> Result<String, String> {
        self.client.summarize(messages).await
    }
}
