//! Actor Core Module
//!
//! Main event loop and command handling for the AgentActor.

use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::agent::memory::{ContextWindow, Message};
use crate::agent::workspace::WorkspaceManager;
use crate::agent::rag::ContextManager;
use crate::agent::history::{HistoryManager, Conversation};
use crate::agent::parser::StreamingParser;
use crate::state::app_state::{ApprovalState, ConnectionMode};
use crate::providers::{ModelProvider, ProviderCapabilities};
use crate::providers::cloud::CloudProvider;
use crate::providers::local::LocalLlamaProvider;

use super::prompts;

/// Commands that can be sent to the agent actor
#[derive(Debug)]
pub enum AgentCommand {
    StartTask { prompt: String, mode: String },
    UserFeedback { approved: bool },
    SetConnectionMode { is_offline: bool },
    Cancel,
}

/// Configuration for the agent
pub struct AgentConfig {
    pub max_steps: usize,
    pub context_window_size: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 15,
            context_window_size: 28000,
        }
    }
}

/// Builder for AgentActor
pub struct AgentActorBuilder {
    rx: Option<mpsc::Receiver<AgentCommand>>,
    app_handle: Option<AppHandle>,
    api_keys: Vec<String>,
    approval_state: Option<Arc<RwLock<ApprovalState>>>,
    connection_mode: ConnectionMode,
}

impl AgentActorBuilder {
    pub fn new() -> Self {
        Self {
            rx: None,
            app_handle: None,
            api_keys: vec![],
            approval_state: None,
            connection_mode: ConnectionMode::Cloud,
        }
    }
    
    pub fn receiver(mut self, rx: mpsc::Receiver<AgentCommand>) -> Self {
        self.rx = Some(rx);
        self
    }
    
    pub fn app_handle(mut self, handle: AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }
    
    pub fn api_keys(mut self, keys: Vec<String>) -> Self {
        self.api_keys = keys;
        self
    }
    
    pub fn approval_state(mut self, state: Arc<RwLock<ApprovalState>>) -> Self {
        self.approval_state = Some(state);
        self
    }
    
    pub fn connection_mode(mut self, mode: ConnectionMode) -> Self {
        self.connection_mode = mode;
        self
    }
    
    pub fn build(self) -> Result<AgentActor, String> {
        let rx = self.rx.ok_or("Receiver is required")?;
        let app_handle = self.app_handle.ok_or("AppHandle is required")?;
        let approval_state = self.approval_state.ok_or("ApprovalState is required")?;
        
        Ok(AgentActor::new_internal(
            rx,
            app_handle,
            self.api_keys,
            approval_state,
            self.connection_mode,
        ))
    }
}

impl Default for AgentActorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The Agent Actor - runs the ReAct loop with provider abstraction
pub struct AgentActor {
    pub(super) rx: mpsc::Receiver<AgentCommand>,
    pub(super) app_handle: AppHandle,
    pub(super) context: ContextWindow,
    pub(super) provider: Box<dyn ModelProvider>,
    pub(super) workspace: WorkspaceManager,
    pub(super) approval_state: Arc<RwLock<ApprovalState>>,
    pub(super) config: AgentConfig,
    pub(super) mode: String,
    pub(super) cancelled: bool,
    pub(super) api_keys: Vec<String>,
    pub(super) context_summary: Option<String>,
    pub(super) rag: Option<Arc<ContextManager>>,
    pub(super) conversation_id: String,
    pub(super) history_manager: HistoryManager,
    pub(super) current_conversation: Conversation,
    pub(super) streaming_parser: StreamingParser,
}

impl AgentActor {
    /// Create a builder for AgentActor
    pub fn builder() -> AgentActorBuilder {
        AgentActorBuilder::new()
    }
    
    /// Internal constructor used by builder
    fn new_internal(
        rx: mpsc::Receiver<AgentCommand>,
        app_handle: AppHandle,
        api_keys: Vec<String>,
        approval_state: Arc<RwLock<ApprovalState>>,
        connection_mode: ConnectionMode,
    ) -> Self {
        let config = AgentConfig::default();
        let context = ContextWindow::new(config.context_window_size);
        
        // Create provider based on connection mode
        let provider: Box<dyn ModelProvider> = match connection_mode {
            ConnectionMode::Offline => {
                tracing::info!("Using LOCAL provider (offline mode)");
                Box::new(LocalLlamaProvider::with_handle(app_handle.clone()))
            }
            ConnectionMode::Cloud => {
                tracing::info!("Using CLOUD provider");
                Box::new(CloudProvider::new(api_keys.clone()))
            }
        };
        
        // Initialize Workspace Manager
        let workspace = WorkspaceManager::new(app_handle.clone());
        workspace.start_watcher();
        
        // Initialize History Manager
        let history_dir = app_handle.path().app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("history");
        if let Err(e) = std::fs::create_dir_all(&history_dir) {
            tracing::error!("Failed to create history directory: {}", e);
        }
        let history_manager = HistoryManager::new(&history_dir);
        
        // Create new conversation
        let current_conversation = Conversation::new("chat");
        let conversation_id = current_conversation.id.clone();
        
        // Initialize streaming parser
        let streaming_parser = StreamingParser::new();

        Self {
            rx,
            app_handle,
            context,
            provider,
            workspace,
            approval_state,
            config,
            mode: "chat".to_string(),
            cancelled: false,
            api_keys,
            context_summary: None,
            rag: None,
            conversation_id,
            history_manager,
            current_conversation,
            streaming_parser,
        }
    }
    
    /// Legacy constructor for backwards compatibility
    pub fn new(
        rx: mpsc::Receiver<AgentCommand>,
        app_handle: AppHandle,
        api_keys: Vec<String>,
        approval_state: Arc<RwLock<ApprovalState>>,
        connection_mode: ConnectionMode,
    ) -> Self {
        Self::new_internal(rx, app_handle, api_keys, approval_state, connection_mode)
    }
    
    /// Get provider capabilities
    pub fn get_capabilities(&self) -> ProviderCapabilities {
        self.provider.capabilities()
    }
    
    /// Check if current provider supports tools
    pub fn supports_tools(&self) -> bool {
        self.provider.capabilities().supports_tools
    }
    
    /// Emit status to frontend
    pub async fn emit_status(&self, status: &str) {
        tracing::debug!(status = status, "Emitting status");
        self.app_handle.emit("agent-status", status).ok();
    }
    
    /// Emit thought to frontend
    pub fn emit_thought(&self, text: &str) {
        self.app_handle.emit("agent-thinking", text).ok();
    }
}
