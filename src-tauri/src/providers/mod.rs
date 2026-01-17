//! Provider Abstraction Layer
//! 
//! Defines the ModelProvider trait and supporting types for both cloud and local backends.

pub mod cloud;
pub mod local;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::agent::memory::Message;

/// Provider capabilities - what features this provider supports
#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    /// Whether the provider supports tool/function calling
    pub supports_tools: bool,
    /// Whether the provider supports streaming responses
    pub supports_streaming: bool,
    /// Whether the provider uses model cascade (routing + failover)
    pub supports_cascade: bool,
    /// Whether the provider supports background summarization
    pub supports_summarization: bool,
    /// Maximum context window in tokens
    pub max_context_tokens: usize,
}

/// Result of a model completion
pub enum CompletionResult {
    /// Streaming response - yields chunks
    Stream(std::pin::Pin<Box<dyn futures::Stream<Item = Result<String, String>> + Send>>),
}

/// Tool definition for providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: String,
}

/// Model tier for cascade routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// 1B - Fast classifier
    Router,
    /// 4B - Basic chat
    BasicChat,
    /// 12B - Complex chat + fallback
    AdvancedChat,
    /// 27B - Agent with tools
    Agent,
    /// 2B - Background summarizer
    Summarizer,
    /// Local - Offline model
    Local,
}

impl ModelTier {
    pub fn model_name(&self) -> &'static str {
        match self {
            ModelTier::Router => "gemma-3-1b-it",
            ModelTier::BasicChat => "gemma-3-4b-it",
            ModelTier::AdvancedChat => "gemma-3-12b-it",
            ModelTier::Agent => "gemma-3-27b-it",
            // Gemma 3 doesn't have 2B - use 4B for summarization
            ModelTier::Summarizer => "gemma-3-4b-it",
            ModelTier::Local => "local-gguf",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            ModelTier::Router => "1B",
            ModelTier::BasicChat => "4B",
            ModelTier::AdvancedChat => "12B",
            ModelTier::Agent => "27B",
            ModelTier::Summarizer => "2B",
            ModelTier::Local => "Local",
        }
    }
}

/// Complexity classification from router
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Complexity {
    /// Simple request - use 4B
    Simple,
    /// Complex request - use 12B (chat) or 27B (turbo)
    Complex,
}

/// The main provider trait - all backends implement this
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Get provider capabilities
    fn capabilities(&self) -> ProviderCapabilities;
    
    /// Get provider name for logging/display
    fn name(&self) -> &str;
    
    /// Generate streaming completion for chat mode (no tools)
    async fn chat(
        &self, 
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String>;
    
    /// Generate streaming completion for agent/turbo mode (with tools)
    async fn agent(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String>;
    
    /// Classify request complexity (for routing)
    /// Returns None if routing not supported
    async fn classify(&self, _input: &str) -> Option<Complexity> {
        None // Default: no routing
    }
    
    /// Summarize conversation (for context compression)
    /// Returns None if summarization not supported
    async fn summarize(&self, _messages: &[Message]) -> Option<String> {
        None // Default: no summarization
    }
    
    /// Get current active model tier (for UI display)
    fn active_model(&self) -> Option<ModelTier> {
        None
    }
}
