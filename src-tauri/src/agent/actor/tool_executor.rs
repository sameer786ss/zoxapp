//! Tool Executor Module
//!
//! Handles tool execution and user approval flow.

use std::time::Duration;
use serde_json::Value;
use tauri::Emitter;
use tokio::sync::oneshot;

use crate::agent::tools::{get_tool_by_name};
use crate::agent::memory::Message;

use super::core::AgentActor;

impl AgentActor {
    /// Handle tool execution including approval flow
    pub async fn handle_tool_execution(&mut self, tool: &str, parameters: &Value) -> Option<String> {
        let start = std::time::Instant::now();
        self.emit_status(&format!("Executing: {}", tool)).await;
        tracing::info!(tool = tool, params = %parameters, "Tool call");

        if let Some(tool_impl) = get_tool_by_name(tool) {
            let params_str = parameters.to_string();
            
            if tool_impl.requires_approval() {
                self.emit_status(&format!("Waiting approval: {}", tool)).await;
                
                let approved = self.wait_for_approval(tool, &params_str).await;
                
                if !approved {
                    tracing::info!(tool = tool, "Tool denied by user");
                    
                    self.context.add_message(Message {
                        role: "user".to_string(),
                        content: format!("<observation>User DENIED the {} tool. Acknowledge this gracefully and ask what they would like to do instead. Do not retry the tool.</observation>", tool),
                    });
                    
                    return Some("__DENIED__".to_string());
                }
            }
            
            // Emit file access event
            if tool == "read_file" || tool == "write_file" || tool == "replace_lines" || tool == "list_files" {
                if let Some(path) = parameters.get("path").and_then(|p| p.as_str()) {
                    let action = if tool == "write_file" || tool == "replace_lines" { "write" } else { "read" };
                    self.app_handle.emit("agent-file-access", serde_json::json!({
                        "action": action,
                        "path": path
                    })).ok();
                }
            }
            
            // Execute with timeout
            let result = tokio::time::timeout(
                Duration::from_secs(30),
                tokio::task::spawn_blocking({
                    let tool_name = tool.to_string();
                    let params = params_str.clone();
                    let workspace = self.workspace.clone();
                    move || {
                        match get_tool_by_name(&tool_name) {
                            Some(tool_impl) => tool_impl.execute(&params, &workspace),
                            None => format!("Error: Tool {} not found during execution", tool_name)
                        }
                    }
                })
            ).await;
            
            let elapsed = start.elapsed();
            tracing::info!(tool = tool, duration_ms = elapsed.as_millis(), "Tool execution complete");
            
            match result {
                Ok(Ok(output)) => Some(output),
                Ok(Err(e)) => Some(format!("Tool execution error: {}", e)),
                Err(_) => Some("Tool execution timed out after 30 seconds".to_string()),
            }
        } else {
            Some(format!("Error: Tool '{}' not found", tool))
        }
    }
    
    /// Wait for user approval
    pub async fn wait_for_approval(&self, name: &str, args: &str) -> bool {
        tracing::debug!(tool = name, "Waiting for approval");
        
        // Create oneshot channel for this approval
        let (tx, rx) = oneshot::channel::<bool>();
        
        // Store sender in shared state
        {
            let mut approval = self.approval_state.write();
            approval.pending = Some(tx);
        }
        
        // Emit approval request to frontend
        self.app_handle.emit("agent-approval-request", serde_json::json!({
            "tool": name,
            "parameters": args
        })).ok();
        
        // Wait for response
        match rx.await {
            Ok(approved) => {
                tracing::debug!(tool = name, approved = approved, "Approval response received");
                approved
            }
            Err(_) => {
                tracing::warn!(tool = name, "Approval channel dropped - assuming denied");
                false
            }
        }
    }
}
