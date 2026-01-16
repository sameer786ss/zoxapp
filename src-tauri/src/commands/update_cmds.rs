//! Update Commands
//!
//! Tauri commands for checking, downloading, and installing updates.
//! Uses tauri-plugin-updater with GitHub releases.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// Update information returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub release_notes: Option<String>,
    pub release_date: Option<String>,
    pub download_size: Option<u64>,
}

/// Download progress event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
    pub speed: f64, // bytes per second
}

/// Update error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

/// Check for available updates
/// Returns UpdateInfo if an update is available, null otherwise
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    println!("[Updater] Checking for updates...");
    
    let updater = app.updater()
        .map_err(|e| format!("Failed to get updater: {}", e))?;
    
    let update = updater.check()
        .await
        .map_err(|e| {
            let error = UpdateError {
                code: "network".to_string(),
                message: e.to_string(),
                retryable: true,
            };
            app.emit("update-error", error).ok();
            format!("Update check failed: {}", e)
        })?;
    
    match update {
        Some(update) => {
            let current_version = app.package_info().version.to_string();
            let info = UpdateInfo {
                version: update.version.clone(),
                current_version,
                release_notes: update.body.clone(),
                release_date: update.date.map(|d| d.to_string()),
                download_size: None, // Not always available
            };
            
            println!("[Updater] Update available: v{}", update.version);
            Ok(Some(info))
        }
        None => {
            println!("[Updater] No updates available");
            Ok(None)
        }
    }
}

/// Download the available update with progress events
#[tauri::command]
pub async fn download_update(app: AppHandle) -> Result<(), String> {
    println!("[Updater] Starting download...");
    
    let updater = app.updater()
        .map_err(|e| format!("Failed to get updater: {}", e))?;
    
    let update = updater.check()
        .await
        .map_err(|e| format!("Update check failed: {}", e))?
        .ok_or("No update available")?;
    
    let app_clone = app.clone();
    let mut downloaded: u64 = 0;
    let start_time = std::time::Instant::now();
    
    // Download with progress tracking
    update.download_and_install(
        move |chunk_len, content_len| {
            downloaded += chunk_len as u64;
            let total = content_len.unwrap_or(0) as u64;
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };
            let percent = if total > 0 { (downloaded as f64 / total as f64) * 100.0 } else { 0.0 };
            
            let progress = UpdateProgress {
                downloaded,
                total,
                percent,
                speed,
            };
            
            app_clone.emit("update-download-progress", progress).ok();
        },
        || {
            // Called before restart - we don't auto-restart, user triggers it
            false
        }
    )
    .await
    .map_err(|e| {
        let error = UpdateError {
            code: "download".to_string(),
            message: e.to_string(),
            retryable: true,
        };
        app.emit("update-error", error).ok();
        format!("Download failed: {}", e)
    })?;
    
    println!("[Updater] Download complete, ready to install");
    Ok(())
}

/// Install the downloaded update and restart the app
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    println!("[Updater] Installing update and restarting...");
    
    let updater = app.updater()
        .map_err(|e| format!("Failed to get updater: {}", e))?;
    
    let update = updater.check()
        .await
        .map_err(|e| format!("Update check failed: {}", e))?
        .ok_or("No update available")?;
    
    // Download and install with restart
    update.download_and_install(
        |_, _| {},
        || {
            // Return true to trigger restart
            true
        }
    )
    .await
    .map_err(|e| {
        let error = UpdateError {
            code: "install".to_string(),
            message: e.to_string(),
            retryable: false,
        };
        app.emit("update-error", error).ok();
        format!("Install failed: {}", e)
    })?;
    
    // If we reach here, restart was triggered
    Ok(())
}

/// Get current app version
#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}
