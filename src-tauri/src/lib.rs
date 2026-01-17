#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)] // Many utilities are used conditionally or reserved for future use

mod agent;
mod commands;
mod error;
mod hardware;
mod providers;
mod setup;
mod state;

pub use error::{ZoxError, ZoxResult};

/// Library entry point for mobile platforms
/// Desktop uses main.rs directly
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;
    use state::app_state::{AppState, ApprovalState, ConnectionMode};
    use std::sync::Arc;
    use parking_lot::RwLock;
    use crate::agent::actor::spawn_agent;

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Load API keys from environment
            let keys = load_api_keys_for_mobile();
            
            if keys.is_empty() {
                eprintln!("[WARNING] No API keys found for mobile!");
            }

            // Create shared approval state
            let approval_state = Arc::new(RwLock::new(ApprovalState::new()));

            // Spawn Agent Actor
            let (agent_tx, workspace) = spawn_agent(
                app.handle().clone(), 
                keys,
                approval_state.clone(),
                ConnectionMode::Cloud,  // Default to cloud mode
            );

            // Create and manage state with shared approval_state
            let state = AppState::new(agent_tx, approval_state);
            state.set_workspace(workspace);
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Agent commands
            commands::agent_cmds::start_agent_task,
            commands::agent_cmds::cancel_agent_task,
            commands::agent_cmds::send_user_feedback,
            commands::agent_cmds::read_workspace_file,
            commands::agent_cmds::save_workspace_file,
            commands::agent_cmds::list_conversations,
            commands::agent_cmds::delete_conversation,
            commands::agent_cmds::load_conversation,
            commands::agent_cmds::export_conversation,
            // Setup commands
            commands::setup_cmds::detect_gpu_cmd,
            commands::setup_cmds::check_setup_status,
            commands::setup_cmds::download_binaries,
            commands::setup_cmds::download_model,
            commands::setup_cmds::set_connection_mode,
            commands::setup_cmds::pause_download,
            commands::setup_cmds::resume_download,
            commands::setup_cmds::cancel_download,
            // Update commands
            commands::update_cmds::check_for_updates,
            commands::update_cmds::download_update,
            commands::update_cmds::install_update,
            commands::update_cmds::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Load API keys for mobile (simplified - typically from secure storage)
fn load_api_keys_for_mobile() -> Vec<String> {
    if let Ok(keys_str) = std::env::var("GEMINI_API_KEYS") {
        keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![]
    }
}

