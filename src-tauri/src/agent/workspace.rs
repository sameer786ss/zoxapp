use std::path::{Path, PathBuf};
use notify::{Watcher, RecursiveMode};
use tauri::{AppHandle, Emitter};
use std::fs;

/// Manages the ZOX data directories:
/// - workspace: Agent-edited files
/// - data: RAG/LanceDB storage
/// - history: Chat history
#[derive(Clone)]
pub struct WorkspaceManager {
    /// Root directory for all ZOX data
    root_dir: PathBuf,
    /// Workspace directory for agent-edited files
    workspace_dir: PathBuf,
    /// Data directory for RAG/LanceDB
    data_dir: PathBuf,
    /// History directory for chat conversations
    history_dir: PathBuf,
    app_handle: AppHandle,
}

impl WorkspaceManager {
    pub fn new(app_handle: AppHandle) -> Self {
        // Get app data directory: %APPDATA%\zox on Windows (not zox\ZOX)
        let root_dir = if cfg!(target_os = "windows") {
            // Use APPDATA directly to avoid the qualifier/organization/app nesting
            if let Ok(appdata) = std::env::var("APPDATA") {
                PathBuf::from(appdata).join("zox")
            } else {
                PathBuf::from("./zox_data")
            }
        } else if cfg!(target_os = "macos") {
            // ~/Library/Application Support/zox
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("zox")
        } else {
            // Linux: ~/.local/share/zox
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("zox")
        };

        let workspace_dir = root_dir.join("workspace");
        let data_dir = root_dir.join("data");
        let history_dir = root_dir.join("history");

        // Create directories if they don't exist
        let _ = fs::create_dir_all(&workspace_dir);
        let _ = fs::create_dir_all(&data_dir);
        let _ = fs::create_dir_all(&history_dir);

        println!("[WorkspaceManager] Root: {}", root_dir.display());
        println!("[WorkspaceManager] Workspace: {}", workspace_dir.display());
        println!("[WorkspaceManager] Data: {}", data_dir.display());
        println!("[WorkspaceManager] History: {}", history_dir.display());

        Self {
            root_dir,
            workspace_dir,
            data_dir,
            history_dir,
            app_handle,
        }
    }

    /// Get the root ZOX data directory
    pub fn get_root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Get the workspace directory for agent-edited files
    pub fn get_workspace_dir(&self) -> &Path {
        &self.workspace_dir
    }

    /// Get the data directory for RAG/LanceDB
    pub fn get_data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get the history directory for chat conversations
    pub fn get_history_dir(&self) -> &Path {
        &self.history_dir
    }

    /// Resolve a relative path to an absolute path within the workspace
    /// Validates that the path doesn't escape the workspace
    pub fn resolve_path(&self, relative_path: &str) -> Result<PathBuf, String> {
        // If it's already an absolute path that's within workspace, use it
        let path = Path::new(relative_path);
        
        if path.is_absolute() {
            // Check if it's within our workspace
            if path.starts_with(&self.workspace_dir) {
                return Ok(path.to_path_buf());
            }
            // Otherwise, treat it as just the filename
            let filename = path.file_name()
                .ok_or("Invalid path")?;
            return Ok(self.workspace_dir.join(filename));
        }

        // Resolve relative path
        let full_path = self.workspace_dir.join(relative_path);
        
        // Canonicalize to resolve .. and . (if parent exists)
        let canonical = if full_path.parent().map(|p| p.exists()).unwrap_or(false) {
            full_path.canonicalize().unwrap_or(full_path.clone())
        } else {
            full_path.clone()
        };

        // Security: Ensure path is within workspace
        if !canonical.starts_with(&self.workspace_dir) && 
           !full_path.starts_with(&self.workspace_dir) {
            return Err(format!(
                "Path '{}' is outside workspace directory", 
                relative_path
            ));
        }

        Ok(full_path)
    }

    /// Start watching the workspace for file changes
    pub fn start_watcher(&self) {
        let workspace_dir = self.workspace_dir.clone();
        let app_handle = self.app_handle.clone();

        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            
            let mut watcher = match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("[WorkspaceManager] Failed to create watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&workspace_dir, RecursiveMode::Recursive) {
                eprintln!("[WorkspaceManager] Failed to watch workspace: {}", e);
                return;
            }

            println!("[WorkspaceManager] File watcher started for {}", workspace_dir.display());

            // Process events
            loop {
                match rx.recv() {
                    Ok(event) => {
                        if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                            let paths: Vec<String> = event.paths
                                .iter()
                                .filter_map(|p| p.to_str().map(String::from))
                                .collect();
                            
                            if !paths.is_empty() {
                                let _ = app_handle.emit("workspace-file-change", &paths);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
