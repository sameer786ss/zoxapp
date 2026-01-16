//! File Downloader with Pause/Resume Support
//!
//! Industry-grade download system with:
//! - Pause/resume using HTTP Range headers
//! - Persistent download state across restarts
//! - Real-time progress events

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub step: String,
    pub percent: f64,
    pub speed_mbps: f64,
    pub eta_seconds: u64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub state: DownloadState,
}

/// Download state for UI
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    Downloading,
    Paused,
    Resuming,
    Completed,
    Error,
}

/// Persistent download state file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStateFile {
    pub url: String,
    pub dest: PathBuf,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub step: String,
    pub is_complete: bool,
}

/// Download error types
#[derive(Debug)]
pub enum DownloadError {
    Network(String),
    Io(String),
    InvalidResponse(String),
    Paused,
    Cancelled,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Network(e) => write!(f, "Network error: {}", e),
            DownloadError::Io(e) => write!(f, "IO error: {}", e),
            DownloadError::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            DownloadError::Paused => write!(f, "Download paused"),
            DownloadError::Cancelled => write!(f, "Download cancelled"),
        }
    }
}

/// Download controller for pause/resume/cancel
#[derive(Clone)]
pub struct DownloadController {
    pub is_paused: Arc<AtomicBool>,
    pub is_cancelled: Arc<AtomicBool>,
    pub downloaded_bytes: Arc<AtomicU64>,
}

impl DownloadController {
    pub fn new() -> Self {
        Self {
            is_paused: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }
}

impl Default for DownloadController {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the download state file path
fn get_state_file_path(step: &str) -> PathBuf {
    let temp_dir = super::paths::get_temp_dir();
    temp_dir.join(format!(".download_state_{}.json", step))
}

/// Save download state for resume
async fn save_download_state(state: &DownloadStateFile) -> Result<(), DownloadError> {
    let path = get_state_file_path(&state.step);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| DownloadError::Io(e.to_string()))?;
    tokio::fs::write(&path, json)
        .await
        .map_err(|e| DownloadError::Io(e.to_string()))?;
    Ok(())
}

/// Load download state for resume
pub async fn load_download_state(step: &str) -> Option<DownloadStateFile> {
    let path = get_state_file_path(step);
    if !path.exists() {
        return None;
    }
    let content = tokio::fs::read_to_string(&path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// Clear download state after completion
async fn clear_download_state(step: &str) -> Result<(), DownloadError> {
    let path = get_state_file_path(step);
    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?;
    }
    Ok(())
}

/// Download a file with progress reporting and pause/resume support
pub async fn download_file_with_resume(
    url: &str,
    dest: &Path,
    step: &str,
    app: &AppHandle,
    controller: &DownloadController,
    resume_bytes: u64,
) -> Result<(), DownloadError> {
    println!("[Downloader] Starting download: {} -> {:?} (resume from {})", url, dest, resume_bytes);

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?;
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| DownloadError::Network(e.to_string()))?;

    // Build request with Range header for resume
    let mut request = client
        .get(url)
        .header("User-Agent", "ZOX-Agent/1.0");

    if resume_bytes > 0 {
        request = request.header("Range", format!("bytes={}-", resume_bytes));
        println!("[Downloader] Resuming from byte {}", resume_bytes);
    }

    let response = request
        .send()
        .await
        .map_err(|e| DownloadError::Network(e.to_string()))?;

    // Handle response status
    let status = response.status();
    if !status.is_success() && status.as_u16() != 206 {
        return Err(DownloadError::InvalidResponse(format!(
            "HTTP {}: {}",
            status,
            status.canonical_reason().unwrap_or("Unknown")
        )));
    }

    // Calculate total bytes
    let content_length = response.content_length().unwrap_or(0);
    let total_bytes = if status.as_u16() == 206 {
        // Partial content - add resume offset
        resume_bytes + content_length
    } else {
        content_length
    };

    let mut downloaded_bytes = resume_bytes;
    let start_time = std::time::Instant::now();

    // Open file for append if resuming
    let mut file = if resume_bytes > 0 {
        let f = OpenOptions::new()
            .write(true)
            .append(true)
            .open(dest)
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?;
        f
    } else {
        File::create(dest)
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?
    };

    // Seek to end for append
    if resume_bytes > 0 {
        file.seek(std::io::SeekFrom::End(0))
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?;
    }

    // Stream the response
    let mut stream = response.bytes_stream();
    let mut last_progress_time = std::time::Instant::now();

    while let Some(chunk_result) = stream.next().await {
        // Check for cancellation
        if controller.is_cancelled() {
            println!("[Downloader] Download cancelled");
            return Err(DownloadError::Cancelled);
        }

        // Check for pause
        if controller.is_paused() {
            // Save state for resume
            let state = DownloadStateFile {
                url: url.to_string(),
                dest: dest.to_path_buf(),
                total_bytes,
                downloaded_bytes,
                step: step.to_string(),
                is_complete: false,
            };
            save_download_state(&state).await?;

            // Emit paused event
            let progress = DownloadProgress {
                step: step.to_string(),
                percent: if total_bytes > 0 { (downloaded_bytes as f64 / total_bytes as f64) * 100.0 } else { 0.0 },
                speed_mbps: 0.0,
                eta_seconds: 0,
                downloaded_bytes,
                total_bytes,
                state: DownloadState::Paused,
            };
            app.emit("download-progress", &progress).ok();

            println!("[Downloader] Download paused at {} bytes", downloaded_bytes);
            return Err(DownloadError::Paused);
        }

        let chunk = chunk_result.map_err(|e| DownloadError::Network(e.to_string()))?;

        file.write_all(&chunk)
            .await
            .map_err(|e| DownloadError::Io(e.to_string()))?;

        downloaded_bytes += chunk.len() as u64;
        controller.downloaded_bytes.store(downloaded_bytes, Ordering::SeqCst);

        // Emit progress every 100ms to avoid spam
        if last_progress_time.elapsed().as_millis() >= 100 {
            last_progress_time = std::time::Instant::now();

            let elapsed = start_time.elapsed().as_secs_f64();
            let actual_downloaded = downloaded_bytes - resume_bytes;
            let speed_bps = if elapsed > 0.0 { actual_downloaded as f64 / elapsed } else { 0.0 };
            let speed_mbps = speed_bps / (1024.0 * 1024.0);

            let percent = if total_bytes > 0 {
                (downloaded_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let remaining_bytes = total_bytes.saturating_sub(downloaded_bytes);
            let eta_seconds = if speed_bps > 0.0 {
                (remaining_bytes as f64 / speed_bps) as u64
            } else {
                0
            };

            let progress = DownloadProgress {
                step: step.to_string(),
                percent,
                speed_mbps,
                eta_seconds,
                downloaded_bytes,
                total_bytes,
                state: DownloadState::Downloading,
            };

            app.emit("download-progress", &progress).ok();
        }
    }

    file.flush()
        .await
        .map_err(|e| DownloadError::Io(e.to_string()))?;

    // Clear state file
    clear_download_state(step).await?;

    // Final progress event
    let progress = DownloadProgress {
        step: step.to_string(),
        percent: 100.0,
        speed_mbps: 0.0,
        eta_seconds: 0,
        downloaded_bytes: total_bytes,
        total_bytes,
        state: DownloadState::Completed,
    };
    app.emit("download-progress", &progress).ok();

    println!(
        "[Downloader] Download complete: {} bytes in {:.1}s",
        downloaded_bytes,
        start_time.elapsed().as_secs_f64()
    );

    Ok(())
}

/// Simple download without pause/resume (backwards compatible)
pub async fn download_file(
    url: &str,
    dest: &Path,
    step: &str,
    app: &AppHandle,
) -> Result<(), DownloadError> {
    let controller = DownloadController::new();
    download_file_with_resume(url, dest, step, app, &controller, 0).await
}

/// Get the download URL for GPU binaries based on type
pub fn get_binaries_url(gpu_type: &str) -> &'static str {
    match gpu_type {
        "nvidia" => "https://huggingface.co/sameer786ss/sapiler-engine-dlls/resolve/main/nvidia-cuda-12.0.zip",
        "amd" => "https://huggingface.co/sameer786ss/sapiler-engine-dlls/resolve/main/amd-vulkan.zip",
        _ => "https://huggingface.co/sameer786ss/sapiler-engine-dlls/resolve/main/cpu-fallback.zip",
    }
}

/// Get the model download URL
pub fn get_model_url() -> &'static str {
    "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q6_k.gguf?download=true"
}
