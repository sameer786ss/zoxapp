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
    /// Validates that the path doesn't escape the workspace using component normalization
    pub fn resolve_path(&self, relative_path: &str) -> Result<PathBuf, String> {
        let path = Path::new(relative_path);
        
        // 1. Normalize the path logic to remove components like '.' and '..'
        // We do this purely logically so we don't depend on the file existing
        let mut components = self.workspace_dir.components().map(|c| c.as_os_str()).collect::<Vec<_>>();
        
        // Handle absolute paths (must start with workspace)
        let path_to_resolve = if path.is_absolute() {
            if path.starts_with(&self.workspace_dir) {
               // It's absolute and potentially inside, but might have '..' later
               // We'll parse its components starting from root
               path
            } else {
               // Absolute but outside? Treat as filename in workspace for safety
               let filename = path.file_name().ok_or("Invalid path")?;
               return Ok(self.workspace_dir.join(filename));
            }
        } else {
            path
        };

        // If it was relative, we essentially join it to workspace
        // If it was absolute-in-workspace, we just process its components
        // But to be consistent, let's just process the relative part if absolute
        
        let path_components = if path_to_resolve.is_absolute() {
             // Strip the workspace prefix first? 
             // Actually, easier: reset components to root and push ALL components of path_to_resolve
             // Then normalize
             components.clear();
             path_to_resolve.components()
        } else {
             path.components()
        };

        for component in path_components {
            match component {
                std::path::Component::Prefix(_) => {
                    // Start over if we hit a prefix (should handle absolute case if logic above allows)
                    // But we handled absolute above.
                }, 
                std::path::Component::RootDir => {
                    // If we encounter root dir in iteration, it means we are resetting to root
                    // This implies an absolute path. 
                    if components.is_empty() {
                         components.push(std::ffi::OsStr::new("/")); // Unix simplified
                    }
                    // If we are appending to workspace, we shouldn't hit RootDir unless path was absolute
                }, 
                std::path::Component::CurDir => {}, // Ignore .
                std::path::Component::ParentDir => {
                    // Pop last component if possible
                    if components.len() > self.workspace_dir.components().count() {
                         components.pop();
                    } else {
                        // Attempt to pop workspace dir part - FORBIDDEN
                        return Err(format!("Path '{}' escapes workspace directory", relative_path));
                    }
                },
                std::path::Component::Normal(c) => {
                    components.push(c);
                }
            }
        }

        // Reassemble
        let mut result = PathBuf::new();
        for c in components {
            result.push(c);
        }

        // Final Verify (Paranoid Check)
        if !result.starts_with(&self.workspace_dir) {
             return Err(format!("Path '{}' resolved to outside workspace", relative_path));
        }

        Ok(result)
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
