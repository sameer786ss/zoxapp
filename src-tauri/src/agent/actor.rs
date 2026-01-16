use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use futures::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::agent::parser::{StreamingParser, StreamEvent, ParsedResponse, ResponseParser};
use crate::agent::memory::{ContextWindow, Message};
use crate::agent::tools::{get_all_tools, get_tool_by_name};
use crate::agent::workspace::WorkspaceManager;
use crate::agent::rag::ContextManager;
use crate::agent::history::{HistoryManager, Conversation};
use crate::state::app_state::{ApprovalState, ConnectionMode};
use crate::providers::{ModelProvider, CompletionResult, ProviderCapabilities};
use crate::providers::cloud::CloudProvider;
use crate::providers::local::LocalLlamaProvider;

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

/// System prompts - XML format for Gemma 3 compatibility
pub mod prompts {
    /// System prompt for Chat mode (no tools, conversational)
    pub const CHAT_SYSTEM_PROMPT: &str = r#"You are a helpful AI coding assistant. Respond naturally and conversationally.

When responding:
- Use code examples when helpful
- Explain concepts step by step
- Be direct and friendly

For your responses, you may optionally wrap text in <message> tags but plain text is also fine.

You are knowledgeable in many programming languages including Rust, TypeScript, Python, JavaScript, and more."#;

    /// System prompt for Turbo/Agent mode (with tools) - XML structured
    pub const TURBO_SYSTEM_PROMPT: &str = r#"You are ZOX, a coding agent. You execute tools using XML tags.

## AVAILABLE TOOLS:

| Tool | Parameters | Description |
|------|------------|-------------|
| read_file | path | Read file contents |
| write_file | path, content | Create/overwrite file |
| replace_lines | path, start_line, end_line, new_content | Edit line range |
| search_project | query | Search in codebase |
| list_files | path | List directory |

## RESPONSE FORMAT:

To use a tool, respond with XML tags:

<thinking>brief reason for action</thinking>
<tool>tool_name</tool>
<params>
<path>file/path</path>
<content>value</content>
</params>

To respond without tools:

<message>Your response text here</message>

## RULES:
- One tool per response
- Wait for OBSERVATION before continuing
- Keep thinking brief (1 sentence)
- No markdown code blocks
- Be direct and efficient"#;
}

/// The Agent Actor - runs the ReAct loop with provider abstraction
pub struct AgentActor {
    rx: mpsc::Receiver<AgentCommand>,
    app_handle: AppHandle,
    context: ContextWindow,
    provider: Box<dyn ModelProvider>,
    workspace: WorkspaceManager,
    approval_state: Arc<RwLock<ApprovalState>>,
    config: AgentConfig,
    mode: String,
    cancelled: bool,
    /// API keys for background summarization
    api_keys: Vec<String>,
    /// Cached context summary
    context_summary: Option<String>,
    /// RAG context manager for semantic search
    rag: Option<Arc<ContextManager>>,
    /// Current conversation ID for RAG storage
    conversation_id: String,
    /// History manager for saving/loading conversations
    history_manager: HistoryManager,
    /// Current active conversation
    current_conversation: Conversation,
    /// Streaming parser for real-time tool detection
    streaming_parser: StreamingParser,
}

impl AgentActor {
    pub fn new(
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
                println!("[AgentActor] Using LOCAL provider (offline mode)");
                Box::new(LocalLlamaProvider::with_defaults())
            }
            ConnectionMode::Cloud => {
                println!("[AgentActor] Using CLOUD provider");
                Box::new(CloudProvider::new(api_keys.clone()))
            }
        };
        
        // Initialize Workspace Manager
        let workspace = WorkspaceManager::new(app_handle.clone());
        workspace.start_watcher();
        
        // Initialize History Manager with history directory
        let history_dir = app_handle.path().app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("history");
        if let Err(e) = std::fs::create_dir_all(&history_dir) {
            eprintln!("[AgentActor] Failed to create history directory: {}", e);
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
    
    /// Initialize RAG system (called lazily on first use)
    async fn ensure_rag(&mut self) -> Option<Arc<ContextManager>> {
        if self.rag.is_some() {
            return self.rag.clone();
        }
        
        // Get data directory
        if let Ok(data_dir) = self.app_handle.path().app_data_dir() {
            let rag_dir: std::path::PathBuf = data_dir.join("rag");
            if std::fs::create_dir_all(&rag_dir).is_ok() {
                match ContextManager::new(&rag_dir).await {
                    Ok(cm) => {
                        println!("[AgentActor] RAG initialized");
                        let arc = Arc::new(cm);
                        self.rag = Some(arc.clone());
                        return Some(arc);
                    }
                    Err(e) => {
                        println!("[AgentActor] RAG init failed: {}", e);
                    }
                }
            }
        }
        
        None
    }
    
    /// Store a message in both context window and RAG
    async fn store_message(&mut self, role: &str, content: &str) {
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
        };
        
        // Add to context window
        self.context.add_message(msg);
        
        // Also store in RAG for semantic search (async, non-blocking)
        if let Some(rag) = &self.rag {
            let _ = rag.store_message(&self.conversation_id, role, content).await;
        }
    }
    
    /// Store a tool result in RAG
    async fn store_tool_result(&mut self, tool_name: &str, result: &str) {
        if let Some(rag) = &self.rag {
            let _ = rag.store_tool_result(tool_name, result).await;
        }
    }
    
    /// Add a message to both context and conversation history, then save
    fn save_and_persist_message(&mut self, role: &str, content: &str) {
        // Add to context window for LLM
        self.context.add_message(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        
        // Add to conversation history for persistence
        self.current_conversation.add_message(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        
        // Save to disk (fire and forget)
        if let Err(e) = self.history_manager.save_conversation(&self.current_conversation) {
            eprintln!("[AgentActor] Failed to save conversation: {}", e);
        }
    }
    
    /// Start a new conversation (clears context and creates new conversation)
    fn start_new_conversation(&mut self, mode: &str) {
        self.context = ContextWindow::new(self.config.context_window_size);
        self.current_conversation = Conversation::new(mode);
        self.conversation_id = self.current_conversation.id.clone();
        self.streaming_parser.reset();
        println!("[AgentActor] Started new conversation: {}", self.conversation_id);
    }
    
    /// Get provider capabilities for feature gating
    fn get_capabilities(&self) -> ProviderCapabilities {
        self.provider.capabilities()
    }
    
    /// Check if current provider supports tools (for turbo mode)
    fn supports_tools(&self) -> bool {
        self.provider.capabilities().supports_tools
    }
    
    /// Extract thinking content from response (for displaying in UI)
    fn extract_thinking(response: &str) -> Option<String> {
        let start = response.find("<thinking>")?;
        let end = response.find("</thinking>")?;
        
        if start < end {
            let content = &response[start + 10..end]; // 10 = "<thinking>".len()
            if !content.trim().is_empty() {
                return Some(content.trim().to_string());
            }
        }
        None
    }
    
    /// Clean response for display - strips XML tags, extracts message content
    fn clean_for_display(response: &str) -> String {
        let mut result = response.to_string();
        
        // Strip <thinking>...</thinking> completely
        while let (Some(start), Some(end)) = (result.find("<thinking>"), result.find("</thinking>")) {
            if start < end {
                let before = &result[..start];
                let after = &result[end + 11..]; // 11 = "</thinking>".len()
                result = format!("{}{}", before, after);
            } else {
                break;
            }
        }
        
        // Extract <message>...</message> content if present
        if let (Some(start), Some(end)) = (result.find("<message>"), result.find("</message>")) {
            if start < end {
                return result[start + 9..end].trim().to_string(); // 9 = "<message>".len()
            }
        }
        
        // Strip any remaining tags
        let tags = ["<tool>", "</tool>", "<params>", "</params>", "<path>", "</path>", 
                    "<content>", "</content>", "<message>", "</message>", 
                    "<response>", "</response>", "<output>", "</output>"];
        for tag in tags {
            result = result.replace(tag, "");
        }
        
        result.trim().to_string()
    }

    pub async fn run(mut self) {
        println!("[AgentActor] Actor started with {} provider", self.provider.name());
        
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                AgentCommand::StartTask { prompt, mode } => {
                    self.mode = mode;
                    self.cancelled = false;
                    println!("[AgentActor] Starting task in {} mode: {}", self.mode, prompt);
                    
                    if self.mode == "turbo" {
                        self.execute_react_loop(prompt).await;
                    } else {
                        self.execute_chat_mode(prompt).await;
                    }
                    
                    // Note: Summarization is now done on-demand via build_optimized_messages()
                }
                AgentCommand::Cancel => {
                    println!("[AgentActor] Task cancelled");
                    self.cancelled = true;
                    self.emit_status("Cancelled").await;
                    
                    // Also reject any pending approval via the shared state
                    let mut approval = self.approval_state.write();
                    if let Some(sender) = approval.pending.take() {
                        let _ = sender.send(false);
                    }
                }
                AgentCommand::UserFeedback { approved } => {
                    // NOTE: This should rarely be hit since approval now goes
                    // directly through the oneshot channel via AppState.
                    // But we keep it as a fallback.
                    println!("[AgentActor] Received UserFeedback via command channel (legacy): {}", approved);
                    let mut approval = self.approval_state.write();
                    if let Some(sender) = approval.pending.take() {
                        let _ = sender.send(approved);
                    }
                }
                AgentCommand::SetConnectionMode { is_offline } => {
                    println!("[AgentActor] Switching connection mode: offline={}", is_offline);
                    
                    if is_offline {
                        // Switch to local provider
                        self.provider = Box::new(LocalLlamaProvider::with_defaults());
                        println!("[AgentActor] Switched to LOCAL provider");
                        self.emit_status("Switched to offline mode").await;
                    } else {
                        // Switch to cloud provider
                        self.provider = Box::new(CloudProvider::new(self.api_keys.clone()));
                        println!("[AgentActor] Switched to CLOUD provider");
                        self.emit_status("Switched to cloud mode").await;
                    }
                }
            }
        }
    }
    
    /// Run background summarization if provider supports it
    /// Returns the summary or None
    async fn get_or_create_summary(&mut self) -> Option<String> {
        if !self.provider.capabilities().supports_summarization {
            return None;
        }
        
        // Only summarize if we have enough messages and no cached summary
        if self.context.len() < 6 {
            return self.context_summary.clone();
        }
        
        // Return cached summary if it's still fresh (less than 4 new messages since last summary)
        // For now, we regenerate every time we have > 6 messages
        if self.context_summary.is_some() && self.context.len() < 10 {
            return self.context_summary.clone();
        }
        
        println!("[AgentActor] Generating context summary with 2B...");
        let _ = self.app_handle.emit("context-summary-pending", true);
        
        // Get all messages except last 3 (those will be sent directly)
        let all_messages = self.context.get_history();
        let to_summarize = if all_messages.len() > 3 {
            &all_messages[..all_messages.len() - 3]
        } else {
            return None; // Not enough to summarize
        };
        
        // Create summarizer and get summary
        use crate::providers::cloud::GemmaClient;
        use crate::providers::ModelTier;
        
        let summarizer = GemmaClient::new(self.api_keys.clone(), ModelTier::Summarizer);
        
        match summarizer.summarize(to_summarize).await {
            Ok(summary) => {
                println!("[AgentActor] Summary generated: {}", summary.chars().take(80).collect::<String>());
                self.context_summary = Some(summary.clone());
                let _ = self.app_handle.emit("context-summary", summary.clone());
                let _ = self.app_handle.emit("context-summary-pending", false);
                Some(summary)
            }
            Err(e) => {
                println!("[AgentActor] Summarization failed: {}", e);
                let _ = self.app_handle.emit("context-summary-pending", false);
                None
            }
        }
    }
    
    /// Build optimized messages using RAG semantic search
    /// Under 12 messages: send all
    /// Over 12: RAG search for relevant context + last 8 messages
    async fn build_optimized_messages(&mut self) -> Vec<Message> {
        let msg_count = self.context.len();
        
        // If few messages, send all (no optimization needed)
        if msg_count <= 12 {
            return self.context.get_history().to_vec();
        }
        
        // Get the current query (last user message) for RAG search
        let current_query = self.context.get_history()
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        
        let mut optimized = Vec::new();
        
        // Try RAG search for semantically relevant context
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
                    println!("[AgentActor] Added {} RAG chunks to context", relevant.len());
                }
            }
        }
        
        // Always include last 8 messages for recency
        let all_messages = self.context.get_history();
        let recent: Vec<Message> = all_messages.iter()
            .rev()
            .take(8)
            .rev()
            .cloned()
            .collect();
        
        optimized.extend(recent);
        
        println!("[AgentActor] Context: {} messages -> {} optimized", msg_count, optimized.len());
        
        optimized
    }

    /// Execute the ReAct loop (Turbo Mode) - PRESERVED logic
    async fn execute_react_loop(&mut self, initial_prompt: String) {
        let mut current_step = 0;

        self.emit_status("Thinking...").await;
        
        // Emit active model info
        if let Some(tier) = self.provider.active_model() {
            self.app_handle.emit("active-model-changed", tier.display_name()).ok();
        }

        // Add user message to context and persist
        self.save_and_persist_message("user", &initial_prompt);

        // Loop
        while current_step < self.config.max_steps && !self.cancelled {
            current_step += 1;
            println!("[AgentActor] Step {}/{}", current_step, self.config.max_steps);
            
            // Build tool definitions for prompt (for logging)
            let tools = get_all_tools();
            println!("[AgentActor] Available tools: {}", tools.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", "));

            // Use TURBO_SYSTEM_PROMPT with full tool definitions
            let system_prompt = prompts::TURBO_SYSTEM_PROMPT;
            
            // Use optimized messages
            let messages = self.build_optimized_messages().await;

            // Stream response via provider - CHUNKED streaming (every 100 chars)
            self.streaming_parser.reset(); // Reset parser for new response
            let mut full_response_text = String::new();
            let mut chunk_buffer = String::new();
            const CHUNK_SIZE: usize = 100;
            
            // Emit streaming status
            self.app_handle.emit("agent-streaming", true).ok();
            
            match self.provider.agent(system_prompt, &messages).await {
                Ok(CompletionResult::Stream(mut stream)) => {
                    while let Some(result) = stream.next().await {
                        if self.cancelled {
                            break;
                        }
                        
                        match result {
                            Ok(token) => {
                                // Feed parser for tool detection
                                let events = self.streaming_parser.feed(&token);
                                for event in events {
                                    match event {
                                        StreamEvent::ToolCallStart { .. } => {
                                            self.emit_status("Tool detected...").await
                                        }
                                        _ => {}
                                    }
                                }
                                
                                // Accumulate
                                full_response_text.push_str(&token);
                                chunk_buffer.push_str(&token);
                                
                                // Extract and emit thinking content if present
                                if let Some(thinking) = Self::extract_thinking(&full_response_text) {
                                    self.app_handle.emit("agent-thinking", &thinking).ok();
                                }
                                
                                // Check if this looks like a tool call - if so, don't stream to chat
                                let is_tool_response = full_response_text.contains("<tool>") || 
                                                        full_response_text.contains("write_file") ||
                                                        full_response_text.contains("read_file") ||
                                                        full_response_text.contains("list_files") ||
                                                        full_response_text.contains("search_project") ||
                                                        full_response_text.contains("replace_lines") ||
                                                        full_response_text.contains("run_terminal");
                                
                                // Only emit cleaned chunks for non-tool responses
                                if chunk_buffer.len() >= CHUNK_SIZE && !is_tool_response {
                                    let cleaned = Self::clean_for_display(&full_response_text);
                                    if !cleaned.is_empty() {
                                        self.app_handle.emit("agent-stream-chunk", &cleaned).ok();
                                    }
                                    chunk_buffer.clear();
                                } else if chunk_buffer.len() >= CHUNK_SIZE {
                                    chunk_buffer.clear(); // Clear buffer but don't emit for tool responses
                                }
                            }
                            Err(e) => {
                                println!("[AgentActor] Stream error: {}", e);
                                self.emit_status(&format!("Error: {}", e)).await;
                                self.app_handle.emit("agent-streaming", false).ok();
                                return;
                            }
                        }
                    }
                    
                    // Only emit final chunk if NOT a tool response
                    let is_tool_response = full_response_text.contains("<tool>") || 
                                            ResponseParser::has_tool_call(&full_response_text);
                    if !full_response_text.is_empty() && !is_tool_response {
                        let cleaned = Self::clean_for_display(&full_response_text);
                        if !cleaned.is_empty() {
                            self.app_handle.emit("agent-stream-chunk", &cleaned).ok();
                        }
                    }
                }
                Err(e) => {
                    println!("[AgentActor] Provider error: {}", e);
                    self.emit_status("API Error").await;
                    self.app_handle.emit("agent-streaming", false).ok();
                    return;
                }
            };
            
            self.app_handle.emit("agent-streaming", false).ok();

            if self.cancelled {
                return;
            }

            // Parse Final Response - EXACT same logic
            let final_parsed = self.streaming_parser.finalize();
            
            // Add model response to history and persist
            self.save_and_persist_message("model", &full_response_text);

            match final_parsed {
                ParsedResponse::ToolCall { tool, parameters, thinking: _ } => {
                    let result = self.handle_tool_execution(&tool, &parameters).await;
                    
                    match result {
                        Some(tool_result) if tool_result == "__DENIED__" => {
                            // Tool was denied - let model respond gracefully (one more iteration)
                            // Denial already added to context in handle_tool_execution
                            println!("[AgentActor] Tool denied - letting model respond gracefully");
                            self.emit_status("Responding...").await;
                            // Continue loop - model will respond to denial, then ParsedResponse::Text will stop
                        }
                        Some(tool_result) => {
                            // Tool executed successfully - send observation
                            self.app_handle.emit("agent-tool-result", serde_json::json!({
                                "tool": tool,
                                "parameters": parameters,
                                "result": tool_result
                            })).ok();

                            // Add observation to context in XML format and persist
                            let observation = format!("<observation>{}</observation>", tool_result);
                            self.save_and_persist_message("user", &observation);
                            
                            self.emit_status("Thinking...").await;
                        }
                        None => {
                            // Denied - stop agent
                            println!("[AgentActor] Denied by user");
                            self.emit_status("Denied").await;
                            self.app_handle.emit("agent-stream-end", "denied").ok();
                            return;
                        }
                    }
                }
                ParsedResponse::Text(text) => {
                    println!("[AgentActor] Final Answer: {}", text);
                    
                    // Emit complete message to frontend (single update)
                    self.app_handle.emit("agent-message-complete", serde_json::json!({
                        "role": "model",
                        "content": text
                    })).ok();
                    
                    self.emit_status("Ready").await;
                    self.app_handle.emit("agent-stream-end", "complete").ok();
                    break; 
                }
                ParsedResponse::TextThenTool { text: _, tool, parameters, thinking: _ } => {
                    let result = self.handle_tool_execution(&tool, &parameters).await;
                    
                    match result {
                        Some(tool_result) if tool_result == "__DENIED__" => {
                            // Tool denied - let model respond gracefully
                            println!("[AgentActor] Tool denied - letting model respond gracefully");
                            self.emit_status("Responding...").await;
                        }
                        Some(tool_result) => {
                            self.app_handle.emit("agent-tool-result", serde_json::json!({
                                "tool": tool,
                                "parameters": parameters,
                                "result": tool_result
                            })).ok();

                            // Add observation in XML format
                            let observation = format!("<observation>{}</observation>", tool_result);
                            self.context.add_message(Message {
                                role: "user".to_string(),
                                content: observation,
                            });
                            
                            self.emit_status("Thinking...").await;
                        }
                        None => {
                            // Denied - stop agent
                            println!("[AgentActor] Denied by user");
                            self.emit_status("Denied").await;
                            self.app_handle.emit("agent-stream-end", "denied").ok();
                            return;
                        }
                    }
                }
            }
        }
        
        if current_step >= self.config.max_steps {
            self.emit_status("Max steps reached").await;
            self.app_handle.emit("agent-stream-end", "max_steps").ok();
        }
    }
    
    /// Handle tool execution including approval flow - PRESERVED logic
    async fn handle_tool_execution(&mut self, tool: &str, parameters: &Value) -> Option<String> {
        self.emit_status(&format!("Executing: {}", tool)).await;
        println!("[AgentActor] Tool Call: {} params: {}", tool, parameters);

        if let Some(tool_impl) = get_tool_by_name(tool) {
            let params_str = parameters.to_string();
            
            if tool_impl.requires_approval() {
                self.emit_status(&format!("Waiting approval: {}", tool)).await;
                
                let approved = self.wait_for_approval(tool, &params_str).await;
                
                if !approved {
                    println!("[AgentActor] Tool denied by user - will respond gracefully");
                    
                    // Add denial to context so model can acknowledge gracefully
                    self.context.add_message(Message {
                        role: "user".to_string(),
                        content: format!("<observation>User DENIED the {} tool. Acknowledge this gracefully and ask what they would like to do instead. Do not retry the tool.</observation>", tool),
                    });
                    
                    // Return special marker - model will respond to denial then stop
                    return Some("__DENIED__".to_string());
                }
            }
            
            // Emit file access event for file-related tools
            if tool == "read_file" || tool == "write_file" || tool == "replace_lines" || tool == "list_files" {
                if let Some(path) = parameters.get("path").and_then(|p| p.as_str()) {
                    let action = if tool == "write_file" || tool == "replace_lines" { "write" } else { "read" };
                    self.app_handle.emit("agent-file-access", serde_json::json!({
                        "action": action,
                        "path": path
                    })).ok();
                }
            }
            
            // Execute the tool with timeout protection
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                tokio::task::spawn_blocking({
                    let tool_name = tool.to_string();
                    let params = params_str.clone();
                    let workspace = self.workspace.clone();
                    move || {
                        match get_tool_by_name(&tool_name) {
                            Some(tool_impl) => tool_impl.execute(&params, &workspace),
                            None => Err(format!("Tool {} not found during execution", tool_name).into())
                        }
                    }
                })
            ).await;
            
            match result {
                Ok(Ok(output)) => Some(output),
                Ok(Err(e)) => Some(format!("Tool execution error: {}", e)),
                Err(_) => Some("Tool execution timed out after 30 seconds".to_string()),
            }
        } else {
            Some(format!("Error: Tool '{}' not found", tool))
        }
    }

    /// Execute Chat Mode (No Tools) - Accumulate in Rust, emit complete only
    async fn execute_chat_mode(&mut self, initial_prompt: String) {
        self.emit_status("Thinking...").await;
        
        // Emit active model info
        if let Some(tier) = self.provider.active_model() {
            self.app_handle.emit("active-model-changed", tier.display_name()).ok();
        }

        // Add user message and persist
        self.save_and_persist_message("user", &initial_prompt);

        let system_prompt = prompts::CHAT_SYSTEM_PROMPT;
        
        // Use optimized messages
        let messages = self.build_optimized_messages().await;

        let mut full_response = String::new();
        let mut chunk_buffer = String::new();
        const CHUNK_SIZE: usize = 100;
        
        // Emit streaming status
        self.app_handle.emit("agent-streaming", true).ok();

        match self.provider.chat(system_prompt, &messages).await {
            Ok(CompletionResult::Stream(mut stream)) => {
                while let Some(result) = stream.next().await {
                    if self.cancelled {
                        break;
                    }
                    
                    match result {
                        Ok(token) => {
                            // Accumulate
                            full_response.push_str(&token);
                            chunk_buffer.push_str(&token);
                            
                            // Extract and emit thinking if present
                            if let Some(thinking) = Self::extract_thinking(&full_response) {
                                self.app_handle.emit("agent-thinking", &thinking).ok();
                            }
                            
                            // Emit cleaned chunk when buffer reaches threshold
                            if chunk_buffer.len() >= CHUNK_SIZE {
                                let cleaned = Self::clean_for_display(&full_response);
                                if !cleaned.is_empty() {
                                    self.app_handle.emit("agent-stream-chunk", &cleaned).ok();
                                }
                                chunk_buffer.clear();
                            }
                        }
                        Err(e) => {
                            self.emit_status("Error streaming").await;
                            println!("Stream error: {}", e);
                        }
                    }
                }
                
                // Emit final cleaned chunk
                if !full_response.is_empty() {
                    let cleaned = Self::clean_for_display(&full_response);
                    if !cleaned.is_empty() {
                        self.app_handle.emit("agent-stream-chunk", &cleaned).ok();
                    }
                }
            },
            Err(e) => {
                self.emit_status("Error connecting").await;
                println!("Provider error: {}", e);
                self.app_handle.emit("agent-streaming", false).ok();
                return;
            }
        }
        
        self.app_handle.emit("agent-streaming", false).ok();

        // Add to context and persist
        self.save_and_persist_message("model", &full_response);

        self.emit_status("Ready").await;
        self.app_handle.emit("agent-stream-end", "complete").ok();
    }

    /// Wait for user approval using oneshot channel - PRESERVED
    async fn wait_for_approval(&self, name: &str, args: &str) -> bool {
        // Create a oneshot channel for this approval request
        let (tx, rx) = oneshot::channel::<bool>();
        
        // Store the sender in the approval state
        {
            let mut approval = self.approval_state.write();
            approval.pending = Some(tx);
        }
        
        // Request approval from frontend
        let approval_request = serde_json::json!({
            "tool": name,
            "parameters": args
        });
        
        self.app_handle.emit("agent-approval-request", &approval_request).ok();
        self.emit_status("Waiting Approval...").await;
        
        // Wait for the response
        match rx.await {
            Ok(approved) => {
                println!("[AgentActor] Approval result: {}", approved);
                approved
            }
            Err(_) => {
                // Channel was dropped (e.g., cancelled)
                println!("[AgentActor] Approval channel closed");
                false
            }
        }
    }

    /// Emit a thought/text to the frontend
    async fn emit_thought(&self, text: &str) {
        self.app_handle.emit("agent-thought", text).ok();
    }

    /// Emit status change to the frontend
    async fn emit_status(&self, status: &str) {
        self.app_handle.emit("agent-status", status).ok();
    }
}

/// Spawn the agent actor and return the command sender and workspace
pub fn spawn_agent(
    app: AppHandle, 
    keys: Vec<String>,
    approval_state: Arc<RwLock<ApprovalState>>,
    connection_mode: ConnectionMode,
) -> (mpsc::Sender<AgentCommand>, WorkspaceManager) {
    let (tx, rx) = mpsc::channel(32);
    let app_clone = app.clone();
    let approval_clone = approval_state.clone();
    
    // Create workspace manager first so we can return it
    let workspace = WorkspaceManager::new(app.clone());
    let workspace_clone = workspace.clone();
    
    // Spawn using Tauri's async runtime
    tauri::async_runtime::spawn(async move {
        let mut actor = AgentActor::new(rx, app_clone, keys, approval_clone, connection_mode);
        // Use the same workspace instance
        actor.workspace = workspace_clone;
        actor.run().await;
    });
    
    (tx, workspace)
}
