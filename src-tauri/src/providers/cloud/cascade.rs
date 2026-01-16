//! Cascade Executor
//! 
//! Manages multiple model clients and handles failover logic:
//! - Chat: 4B → 12B fallback
//! - Agent: 27B → 12B fallback
//! - Summarizer: 2B (background, no fallback)

use super::client::GemmaClient;
use crate::agent::memory::Message;
use crate::providers::{ModelTier, CompletionResult};

/// Cascade executor with all model clients
pub struct CascadeExecutor {
    /// 4B - Basic chat
    client_4b: GemmaClient,
    /// 12B - Advanced chat + fallback
    client_12b: GemmaClient,
    /// 27B - Agent
    client_27b: GemmaClient,
    /// 2B - Summarizer
    client_2b: GemmaClient,
}

impl CascadeExecutor {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            client_4b: GemmaClient::new(keys.clone(), ModelTier::BasicChat),
            client_12b: GemmaClient::new(keys.clone(), ModelTier::AdvancedChat),
            client_27b: GemmaClient::new(keys.clone(), ModelTier::Agent),
            client_2b: GemmaClient::new(keys, ModelTier::Summarizer),
        }
    }
    
    /// Execute chat request with failover
    pub async fn execute_chat(
        &self,
        tier: ModelTier,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        let primary = match tier {
            ModelTier::BasicChat => &self.client_4b,
            ModelTier::AdvancedChat => &self.client_12b,
            _ => &self.client_4b, // Default to 4B
        };
        
        println!("[Cascade] Trying {} for chat...", primary.tier().display_name());
        
        // Try primary model
        match primary.stream_completion(system_prompt, messages, false).await {
            Ok(stream) => {
                println!("[Cascade] {} succeeded", primary.tier().display_name());
                Ok(CompletionResult::Stream(Box::pin(stream)))
            }
            Err(e) if e == "RATE_LIMIT" => {
                // Failover to 12B
                println!("[Cascade] {} rate limited, failing over to 12B", primary.tier().display_name());
                
                match self.client_12b.stream_completion(system_prompt, messages, false).await {
                    Ok(stream) => {
                        println!("[Cascade] 12B fallback succeeded");
                        Ok(CompletionResult::Stream(Box::pin(stream)))
                    }
                    Err(e) => Err(format!("All models failed: {}", e))
                }
            }
            Err(e) => Err(e)
        }
    }
    
    /// Execute agent request with failover
    pub async fn execute_agent(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        println!("[Cascade] Trying 27B for agent...");
        
        // Try 27B first
        match self.client_27b.stream_completion(system_prompt, messages, true).await {
            Ok(stream) => {
                println!("[Cascade] 27B succeeded");
                Ok(CompletionResult::Stream(Box::pin(stream)))
            }
            Err(e) if e == "RATE_LIMIT" => {
                // Failover to 12B with agent prompt
                println!("[Cascade] 27B rate limited, failing over to 12B");
                
                match self.client_12b.stream_completion(system_prompt, messages, true).await {
                    Ok(stream) => {
                        println!("[Cascade] 12B fallback succeeded");
                        Ok(CompletionResult::Stream(Box::pin(stream)))
                    }
                    Err(e) => Err(format!("All agent models failed: {}", e))
                }
            }
            Err(e) => Err(e)
        }
    }
    
    /// Summarize using 2B (no failover - background task)
    pub async fn summarize(&self, messages: &[Message]) -> Result<String, String> {
        println!("[Cascade] Running 2B summarizer...");
        self.client_2b.summarize(messages).await
    }
}
