//! Cloud Provider Module
//! 
//! Implements the full tiered cascade for Gemini API:
//! - 1B Router for classification
//! - 4B Basic Chat
//! - 12B Advanced Chat + Fallback
//! - 27B Agent
//! - 2B Summarizer

mod client;
mod router;
mod cascade;

pub use client::GemmaClient;
pub use router::ModelRouter;
pub use cascade::CascadeExecutor;

use async_trait::async_trait;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::agent::memory::Message;
use crate::providers::{
    ModelProvider, ProviderCapabilities, CompletionResult, 
    ModelTier, Complexity
};

/// Cloud provider using Gemini API with full cascade
pub struct CloudProvider {
    /// Model router (1B)
    router: ModelRouter,
    /// Cascade executor with all model clients
    cascade: CascadeExecutor,
    /// Current active model for UI display
    active_model: Arc<RwLock<ModelTier>>,
}

impl CloudProvider {
    pub fn new(api_keys: Vec<String>) -> Self {
        let router = ModelRouter::new(api_keys.clone());
        let cascade = CascadeExecutor::new(api_keys);
        
        Self {
            router,
            cascade,
            active_model: Arc::new(RwLock::new(ModelTier::Agent)),
        }
    }
}

#[async_trait]
impl ModelProvider for CloudProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_cascade: true,
            supports_summarization: true,
            max_context_tokens: 128000, // Gemma 3 27B supports 128K context
        }
    }
    
    fn name(&self) -> &str {
        "Gemini Cloud"
    }
    
    async fn chat(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        // Step 1: Route with 1B to determine complexity
        let input = messages.last()
            .map(|m| m.content.as_str())
            .unwrap_or("");
        
        let complexity = self.router.classify(input).await
            .unwrap_or(Complexity::Simple);
        
        // Step 2: Execute with appropriate model
        let tier = match complexity {
            Complexity::Simple => ModelTier::BasicChat,  // 4B
            Complexity::Complex => ModelTier::AdvancedChat, // 12B
        };
        
        *self.active_model.write() = tier;
        
        // Step 3: Execute with failover
        self.cascade.execute_chat(tier, system_prompt, messages).await
    }
    
    async fn agent(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        // Agent mode: Always use 27B with 12B fallback
        *self.active_model.write() = ModelTier::Agent;
        
        self.cascade.execute_agent(system_prompt, messages).await
    }
    
    async fn classify(&self, input: &str) -> Option<Complexity> {
        self.router.classify(input).await.ok()
    }
    
    async fn summarize(&self, messages: &[Message]) -> Option<String> {
        self.cascade.summarize(messages).await.ok()
    }
    
    fn active_model(&self) -> Option<ModelTier> {
        Some(*self.active_model.read())
    }
}
