//! Setup Commands
//!
//! Tauri commands for GPU detection, binary/model download, and mode switching.

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::hardware::{detect_gpu, GpuInfo};
use crate::setup::downloader::{
    download_file_with_resume, get_binaries_url, get_model_url, get_tokenizer_url,
    load_download_state, DownloadController,
};
use crate::setup::extractor::{cleanup_temp_file, extract_zip};
use crate::setup::paths::{ensure_directories, get_binaries_dir, get_model_path, get_tokenizer_path, get_temp_download_path, SetupStatus};
use crate::state::app_state::AppState;

/// Global download controller for pause/resume
static DOWNLOAD_CONTROLLER: Lazy<RwLock<Option<DownloadController>>> = Lazy::new(|| RwLock::new(None));

/// GPU detection result for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDetectionResult {
    pub gpu_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vram_mb: Option<u64>,
}

impl From<GpuInfo> for GpuDetectionResult {
    fn from(info: GpuInfo) -> Self {
        GpuDetectionResult {
            gpu_type: info.gpu_type.as_str().to_string(),
            name: info.name,
            vram_mb: info.vram_mb,
        }
    }
}

/// Setup status for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatusResult {
    pub binaries_ok: bool,
    pub model_ok: bool,
}

impl From<SetupStatus> for SetupStatusResult {
    fn from(status: SetupStatus) -> Self {
        SetupStatusResult {
            binaries_ok: status.binaries_ok,
            model_ok: status.model_ok,
        }
    }
}

/// Detect GPU hardware
#[tauri::command]
pub fn detect_gpu_cmd() -> Result<GpuDetectionResult, String> {
    println!("[Command] detect_gpu");
    let info = detect_gpu();
    Ok(info.into())
}

/// Check if setup is complete (binaries and model downloaded)
#[tauri::command]
pub fn check_setup_status() -> Result<SetupStatusResult, String> {
    println!("[Command] check_setup_status");
    let status = SetupStatus::check();
    println!("[Command] Setup status: binaries={}, model={}", status.binaries_ok, status.model_ok);
    Ok(status.into())
}

/// Download GPU-specific binaries with pause/resume support
#[tauri::command]
pub async fn download_binaries(gpu_type: String, app: AppHandle) -> Result<(), String> {
    println!("[Command] download_binaries: gpu_type={}", gpu_type);

    // Ensure directories exist
    ensure_directories().map_err(|e| format!("Failed to create directories: {}", e))?;

    // Create and store controller
    let controller = DownloadController::new();
    {
        let mut ctrl = DOWNLOAD_CONTROLLER.write();
        *ctrl = Some(controller.clone());
    }

    let url = get_binaries_url(&gpu_type);
    let temp_zip = get_temp_download_path("binaries.zip");
    let dest_dir = get_binaries_dir();

    // Check for existing download state
    let resume_bytes = if let Some(state) = load_download_state("binaries").await {
        if !state.is_complete && temp_zip.exists() {
            state.downloaded_bytes
        } else {
            0
        }
    } else {
        0
    };

    // Download the ZIP file with resume support
    match download_file_with_resume(url, &temp_zip, "binaries", &app, &controller, resume_bytes).await {
        Ok(()) => {
            // Extract to binaries directory
            extract_zip(&temp_zip, &dest_dir).map_err(|e| format!("Extraction failed: {}", e))?;
            cleanup_temp_file(&temp_zip).ok();
            println!("[Command] Binaries download complete");
            Ok(())
        }
        Err(crate::setup::downloader::DownloadError::Paused) => {
            println!("[Command] Binaries download paused");
            Err("Download paused".to_string())
        }
        Err(crate::setup::downloader::DownloadError::Cancelled) => {
            cleanup_temp_file(&temp_zip).ok();
            println!("[Command] Binaries download cancelled");
            Err("Download cancelled".to_string())
        }
        Err(e) => Err(format!("Download failed: {}", e)),
    }
}

/// Download the AI model with pause/resume support
#[tauri::command]
pub async fn download_model(app: AppHandle) -> Result<(), String> {
    println!("[Command] download_model");

    // Ensure directories exist
    ensure_directories().map_err(|e| format!("Failed to create directories: {}", e))?;

    // Create and store controller
    let controller = DownloadController::new();
    {
        let mut ctrl = DOWNLOAD_CONTROLLER.write();
        *ctrl = Some(controller.clone());
    }

    // 1. Download Model
    let model_url = get_model_url();
    let model_dest = get_model_path();

    // Check for existing download state
    let resume_bytes = if let Some(state) = load_download_state("model").await {
        if !state.is_complete && model_dest.exists() {
            state.downloaded_bytes
        } else {
            0
        }
    } else {
        0
    };

    println!("[Command] Starting model download");
    match download_file_with_resume(model_url, &model_dest, "model", &app, &controller, resume_bytes).await {
        Ok(()) => {
            println!("[Command] Model download complete");
            // 2. Download Tokenizer
            let tok_url = get_tokenizer_url();
            let tok_dest = get_tokenizer_path();
            
            println!("[Command] Starting tokenizer download");
            // Simple download for tokenizer (it's small, no resume needed really, but consistent)
            match download_file_with_resume(tok_url, &tok_dest, "tokenizer", &app, &controller, 0).await {
                Ok(()) => {
                    println!("[Command] Tokenizer download complete");
                    app.emit("setup-complete", ()).ok();
                    Ok(())
                }
                Err(e) => Err(format!("Tokenizer download failed: {}", e)),
            }
        }
        Err(crate::setup::downloader::DownloadError::Paused) => {
            println!("[Command] Model download paused");
            Err("Download paused".to_string())
        }
        Err(crate::setup::downloader::DownloadError::Cancelled) => {
            // Don't delete partial model - allow resume
            println!("[Command] Model download cancelled");
            Err("Download cancelled".to_string())
        }
        Err(e) => Err(format!("Download failed: {}", e)),
    }
}

/// Pause active download
#[tauri::command]
pub fn pause_download() -> Result<(), String> {
    println!("[Command] pause_download");
    let ctrl = DOWNLOAD_CONTROLLER.read();
    if let Some(controller) = ctrl.as_ref() {
        controller.pause();
        Ok(())
    } else {
        Err("No active download to pause".to_string())
    }
}

/// Resume paused download (returns current progress)
#[tauri::command]
pub fn resume_download() -> Result<u64, String> {
    println!("[Command] resume_download");
    let ctrl = DOWNLOAD_CONTROLLER.read();
    if let Some(controller) = ctrl.as_ref() {
        controller.resume();
        Ok(controller.downloaded_bytes.load(std::sync::atomic::Ordering::SeqCst))
    } else {
        Err("No download to resume".to_string())
    }
}

/// Cancel active download
#[tauri::command]
pub fn cancel_download() -> Result<(), String> {
    println!("[Command] cancel_download");
    let ctrl = DOWNLOAD_CONTROLLER.read();
    if let Some(controller) = ctrl.as_ref() {
        controller.cancel();
        Ok(())
    } else {
        Err("No active download to cancel".to_string())
    }
}

/// Set connection mode (cloud or offline)
/// This triggers model loading/unloading and switches the agent provider
#[tauri::command]
pub async fn set_connection_mode(
    mode: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::agent::actor::AgentCommand;
    
    println!("[Command] set_connection_mode: mode={}", mode);

    match mode.as_str() {
        "offline" => {
            // Check if setup is complete
            let status = SetupStatus::check();
            if !status.is_complete() {
                return Err("Setup not complete. Please download binaries and model first.".to_string());
            }

            // Emit initial progress - actor.rs will emit real progress during load
            app.emit("model-load-progress", 5).ok();

            // Send command to agent to switch provider
            // The agent will await the actual model load and emit real progress events
            let tx = state.agent_tx.lock().await;
            tx.send(AgentCommand::SetConnectionMode { is_offline: true })
                .await
                .map_err(|e| format!("Failed to send mode switch: {}", e))?;

            // Update local state
            {
                let mut conn_mode = state.connection_mode.write();
                *conn_mode = crate::state::app_state::ConnectionMode::Offline;
            }

            // Final events - model-load-complete will be emitted by actor.rs when done
            app.emit("connection-mode-changed", "offline").ok();
        }
        "cloud" => {
            // Send command to agent to switch provider
            let tx = state.agent_tx.lock().await;
            tx.send(AgentCommand::SetConnectionMode { is_offline: false })
                .await
                .map_err(|e| format!("Failed to send mode switch: {}", e))?;

            // Update local state
            {
                let mut conn_mode = state.connection_mode.write();
                *conn_mode = crate::state::app_state::ConnectionMode::Cloud;
            }

            // Emit completion immediately - cloud doesn't need loading
            app.emit("model-load-progress", 0).ok();
            app.emit("model-load-complete", "unloaded").ok();
            app.emit("connection-mode-changed", "cloud").ok();
        }
        _ => {
            return Err(format!("Invalid mode: {}", mode));
        }
    }

    Ok(())
}
