#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)] // Many utilities are used conditionally or reserved for future use

mod agent;
mod commands;
mod hardware;
mod providers;
mod setup;
mod state;

use tauri::{Manager, Emitter};
use state::app_state::{AppState, ApprovalState, ConnectionMode};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::agent::actor::spawn_agent;

#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

fn main() {
    // 0. Add binaries to PATH before any Candle initialization
    // This ensures DLLs in %APPDATA%/zox/binaries are discoverable
    if let Err(e) = setup::paths::add_binaries_to_path() {
        eprintln!("[WARNING] Failed to add binaries to PATH: {}", e);
    }
    
    // 0b. Setup CUDA environment for bundled libraries
    if let Err(e) = setup::paths::setup_cuda_environment() {
        eprintln!("[WARNING] Failed to setup CUDA environment: {}", e);
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // 1. Load API Keys from environment variables (with fallback for dev)
            let keys = load_api_keys();
            
            if keys.is_empty() {
                eprintln!("[WARNING] No API keys found! Set GEMINI_API_KEYS environment variable.");
                eprintln!("[WARNING] Format: GEMINI_API_KEYS=key1,key2,key3");
            }

            // 2. Create shared approval state - this Arc is shared between AppState and AgentActor
            let approval_state = Arc::new(RwLock::new(ApprovalState::new()));

            // 3. Spawn Agent Actor with the shared approval state
            let (agent_tx, workspace) = spawn_agent(
                app.handle().clone(), 
                keys,
                approval_state.clone(),  // Clone Arc, not the inner value
                ConnectionMode::Cloud,   // Default to cloud mode on startup
            );

            // 4. Create AppState with the SAME approval_state Arc
            let state = AppState::new(agent_tx, approval_state);
            state.set_workspace(workspace);
            
            app.manage(state);

            // 5. Apply Windows 11 Mica effect for Fluent Design
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = apply_mica(&window, Some(true));
                }
            }

            // 6. Emit app-ready event for splash screen
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                // Small delay to ensure frontend is ready
                std::thread::sleep(std::time::Duration::from_millis(100));
                handle.emit("app-ready", true).ok();
            });

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
            // File commands
            commands::file_cmds::fast_read_file,
            commands::file_cmds::fast_write_file,
            commands::file_cmds::get_file_outline,
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
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                std::process::exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Load API keys from environment variable or fallback to defaults for development
fn load_api_keys() -> Vec<String> {
    // Try to load from environment variable first
    if let Ok(keys_str) = std::env::var("GEMINI_API_KEYS") {
        let keys: Vec<String> = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if !keys.is_empty() {
            println!("[Config] Loaded {} API keys from environment", keys.len());
            return keys;
        }
    }
    
    // Fallback for development - read from config file if exists
    if let Ok(config_path) = std::env::current_dir() {
        let config_file = config_path.join(".env.local");
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                for line in content.lines() {
                    if line.starts_with("GEMINI_API_KEYS=") {
                        let keys: Vec<String> = line
                            .trim_start_matches("GEMINI_API_KEYS=")
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        
                        if !keys.is_empty() {
                            println!("[Config] Loaded {} API keys from .env.local", keys.len());
                            return keys;
                        }
                    }
                }
            }
        }
    }
    
    // DEVELOPMENT FALLBACK - Remove in production
    #[cfg(debug_assertions)]
    {
        println!("[Config] Using development fallback API keys");
        return vec![
            "AIzaSyAyLQgqmO0PzbePQzCD5S7slHa_Z-jLMHU".into(),
            "AIzaSyA_7Bnzeiq8eBYKAm_16pTkyQEvdAC6KDg".into(),
            "AIzaSyCGUick6BcbreCGBKg8pgL4LiyTYw9hlx8".into(),
        ];
    }
    
    #[cfg(not(debug_assertions))]
    {
        vec![]
    }
}
