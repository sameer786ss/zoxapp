//! Path utilities for setup locations
//!
//! All files are stored in %APPDATA%/zox/

use std::path::PathBuf;

/// Get the binaries directory: %APPDATA%/zox/binaries
pub fn get_binaries_dir() -> PathBuf {
    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("zox").join("binaries")
}

/// Get the models directory: %APPDATA%/zox/models
pub fn get_models_dir() -> PathBuf {
    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("zox").join("models")
}

/// Get the path to the main model file
pub fn get_model_path() -> PathBuf {
    get_models_dir().join("model.gguf")
}

/// Get the path to the tokenizer file
pub fn get_tokenizer_path() -> PathBuf {
    get_models_dir().join("tokenizer.json")
}

/// Get the path for a temp download file
pub fn get_temp_download_path(filename: &str) -> PathBuf {
    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("zox").join("temp").join(filename)
}

/// Get the temp directory
pub fn get_temp_dir() -> PathBuf {
    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("zox").join("temp")
}

/// Get the path to llama.dll (Windows) or libllama.so (Linux)
#[cfg(target_os = "windows")]
pub fn get_llama_lib_path() -> PathBuf {
    get_binaries_dir().join("llama.dll")
}

#[cfg(target_os = "linux")]
pub fn get_llama_lib_path() -> PathBuf {
    get_binaries_dir().join("libllama.so")
}

#[cfg(target_os = "macos")]
pub fn get_llama_lib_path() -> PathBuf {
    get_binaries_dir().join("libllama.dylib")
}

/// Get the path to ggml-cuda.dll for NVIDIA GPU acceleration
#[cfg(target_os = "windows")]
pub fn get_cuda_lib_path() -> PathBuf {
    get_binaries_dir().join("ggml-cuda.dll")
}

/// Add binaries directory to system PATH for DLL loading
/// This must be called before any llama-cpp-2 initialization
pub fn add_binaries_to_path() -> std::io::Result<()> {
    let binaries_dir = get_binaries_dir();
    
    if !binaries_dir.exists() {
        return Ok(()); // Nothing to add if dir doesn't exist
    }
    
    // Get current PATH
    let current_path = std::env::var("PATH").unwrap_or_default();
    
    // Collect all directories to add (main dir + subdirs)
    let mut dirs_to_add = vec![binaries_dir.clone()];
    
    // Add any subdirectories (in case of nested extraction or cuda libs)
    if let Ok(entries) = std::fs::read_dir(&binaries_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                dirs_to_add.push(entry.path());
            }
        }
    }
    
    // Build new PATH with binaries dirs prepended
    let mut new_path_parts = Vec::new();
    for dir in &dirs_to_add {
        let dir_str = dir.to_string_lossy();
        if !current_path.contains(dir_str.as_ref()) {
            new_path_parts.push(dir_str.to_string());
        }
    }
    
    if new_path_parts.is_empty() {
        return Ok(()); // Already in PATH
    }
    
    #[cfg(target_os = "windows")]
    let new_path = format!("{};{}", new_path_parts.join(";"), current_path);
    
    #[cfg(not(target_os = "windows"))]
    let new_path = format!("{}:{}", new_path_parts.join(":"), current_path);
    
    std::env::set_var("PATH", &new_path);
    
    for dir in &dirs_to_add {
        println!("[Paths] Added {} to PATH", dir.display());
    }
    Ok(())
}

/// Setup CUDA environment variables for bundled binaries
/// This ensures Candle uses our bundled CUDA libraries instead of system ones
pub fn setup_cuda_environment() -> std::io::Result<()> {
    let binaries_dir = get_binaries_dir();
    
    if !binaries_dir.exists() {
        return Ok(());
    }
    
    // Set CUDA_PATH to our binaries directory
    std::env::set_var("CUDA_PATH", &binaries_dir);
    
    // Set CUDA_HOME as well (some libraries check this)
    std::env::set_var("CUDA_HOME", &binaries_dir);
    
    // On Windows, set CUDA lib path
    #[cfg(target_os = "windows")]
    {
        let lib_path = binaries_dir.join("lib");
        if lib_path.exists() {
            std::env::set_var("CUDA_LIB_PATH", &lib_path);
        }
    }
    
    // On Linux, update LD_LIBRARY_PATH
    #[cfg(target_os = "linux")]
    {
        let current = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let new_path = format!("{}:{}", binaries_dir.display(), current);
        std::env::set_var("LD_LIBRARY_PATH", new_path);
    }
    
    // On macOS, update DYLD_LIBRARY_PATH
    #[cfg(target_os = "macos")]
    {
        let current = std::env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
        let new_path = format!("{}:{}", binaries_dir.display(), current);
        std::env::set_var("DYLD_LIBRARY_PATH", new_path);
    }
    
    println!("[Paths] Set CUDA_PATH to: {}", binaries_dir.display());
    Ok(())
}

/// Ensure all required directories exist
pub fn ensure_directories() -> std::io::Result<()> {
    std::fs::create_dir_all(get_binaries_dir())?;
    std::fs::create_dir_all(get_models_dir())?;
    std::fs::create_dir_all(get_temp_download_path("").parent().unwrap())?;
    Ok(())
}

/// Check if setup is complete
pub struct SetupStatus {
    pub binaries_ok: bool,
    pub model_ok: bool,
}

impl SetupStatus {
    pub fn check() -> Self {
        let binaries_dir = get_binaries_dir();
        let model_path = get_model_path();
        let tokenizer_path = get_tokenizer_path();

        // Check if binaries directory has any files
        let binaries_ok = binaries_dir.exists() && 
            std::fs::read_dir(&binaries_dir)
                .map(|mut dir| dir.next().is_some())
                .unwrap_or(false);

        // Check if model AND tokenizer exist
        // Model > 1GB, Tokenizer > 1KB
        let model_ok = model_path.exists() && 
            std::fs::metadata(&model_path)
                .map(|m| m.len() > 1_000_000_000) // > 1GB
                .unwrap_or(false) &&
            tokenizer_path.exists() &&
            std::fs::metadata(&tokenizer_path)
                .map(|m| m.len() > 100) // > 100 bytes
                .unwrap_or(false);

        SetupStatus { binaries_ok, model_ok }
    }

    pub fn is_complete(&self) -> bool {
        self.binaries_ok && self.model_ok
    }
}
