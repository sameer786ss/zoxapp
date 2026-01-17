//! Actor Module Index
//!
//! Re-exports all actor submodules.

mod core;
mod context;
mod tool_executor;
pub mod prompts;

pub use core::{AgentActor, AgentActorBuilder, AgentCommand, AgentConfig};

// Spawn agent helper function
use tauri::AppHandle;
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::state::app_state::{ApprovalState, ConnectionMode};
use crate::agent::workspace::WorkspaceManager;

/// Spawn the agent actor and return the command sender
pub fn spawn_agent(
    app_handle: AppHandle,
    api_keys: Vec<String>,
    approval_state: Arc<RwLock<ApprovalState>>,
    connection_mode: ConnectionMode,
) -> (mpsc::Sender<AgentCommand>, WorkspaceManager) {
    let (tx, rx) = mpsc::channel::<AgentCommand>(32);
    
    let actor = AgentActor::builder()
        .receiver(rx)
        .app_handle(app_handle.clone())
        .api_keys(api_keys)
        .approval_state(approval_state)
        .connection_mode(connection_mode)
        .build()
        .expect("Failed to build AgentActor");
    
    let workspace = actor.workspace.clone();
    
    // Spawn actor on async runtime
    tokio::spawn(async move {
        actor.run().await;
    });
    
    (tx, workspace)
}
