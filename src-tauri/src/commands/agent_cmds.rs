use tauri::{State, AppHandle};
use crate::state::app_state::AppState;
use crate::agent::actor::AgentCommand;
use crate::agent::history::{HistoryManager, ConversationMeta};

/// Start an agent task with the given prompt and mode
#[tauri::command]
pub async fn start_agent_task(
    task: String,
    is_turbo: bool,
    state: State<'_, AppState>,
    _app: AppHandle
) -> Result<(), String> {
    let mode = if is_turbo { "turbo" } else { "chat" };
    println!("[Command] start_agent_task: mode={}, prompt={}", mode, &task[..task.len().min(50)]);
    
    let tx = state.agent_tx.lock().await;
    tx.send(AgentCommand::StartTask { 
        prompt: task, 
        mode: mode.to_string() 
    })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;
    Ok(())
}

/// Cancel the current agent task
#[tauri::command]
pub async fn cancel_agent_task(
    state: State<'_, AppState>
) -> Result<(), String> {
    println!("[Command] cancel_agent_task");
    
    state.send_approval(false);
    
    let tx = state.agent_tx.lock().await;
    tx.send(AgentCommand::Cancel)
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;
    Ok(())
}

/// Send user feedback for tool approval
#[tauri::command]
pub async fn send_user_feedback(
    approved: bool,
    state: State<'_, AppState>
) -> Result<(), String> {
    println!("[Command] send_user_feedback: approved={}", approved);
    
    if state.send_approval(approved) {
        Ok(())
    } else {
        println!("[Command] Warning: No pending approval to respond to");
        Ok(())
    }
}

/// Read a file from the workspace
#[tauri::command]
pub async fn read_workspace_file(
    path: String,
    state: State<'_, AppState>,
    _app: AppHandle
) -> Result<String, String> {
    let workspace = state.get_workspace()
        .ok_or_else(|| "Workspace not initialized".to_string())?;
    
    let abs_path = workspace.resolve_path(&path)?;
    
    std::fs::read_to_string(&abs_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Save a file to the workspace
#[tauri::command]
pub async fn save_workspace_file(
    path: String,
    content: String,
    state: State<'_, AppState>,
    _app: AppHandle
) -> Result<(), String> {
    let workspace = state.get_workspace()
        .ok_or_else(|| "Workspace not initialized".to_string())?;
    
    let abs_path = workspace.resolve_path(&path)?;
    
    if let Some(parent) = abs_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }
    }
    
    std::fs::write(&abs_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

/// List all chat conversations
#[tauri::command]
pub async fn list_conversations(
    state: State<'_, AppState>
) -> Result<Vec<ConversationMeta>, String> {
    let workspace = state.get_workspace()
        .ok_or_else(|| "Workspace not initialized".to_string())?;
    
    let history_manager = HistoryManager::new(workspace.get_history_dir());
    Ok(history_manager.list_conversations())
}

/// Delete a conversation
#[tauri::command]
pub async fn delete_conversation(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let workspace = state.get_workspace()
        .ok_or_else(|| "Workspace not initialized".to_string())?;
    
    let history_manager = HistoryManager::new(workspace.get_history_dir());
    history_manager.delete_conversation(&id)
}

/// Load a specific conversation by ID
#[tauri::command]
pub async fn load_conversation(
    id: String,
    state: State<'_, AppState>
) -> Result<crate::agent::history::Conversation, String> {
    let workspace = state.get_workspace()
        .ok_or_else(|| "Workspace not initialized".to_string())?;
    
    let history_manager = HistoryManager::new(workspace.get_history_dir());
    history_manager.load_conversation(&id)
        .ok_or_else(|| format!("Conversation {} not found", id))
}
