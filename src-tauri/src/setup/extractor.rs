//! ZIP File Extractor
//!
//! Extracts ZIP archives with proper directory handling.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

/// Extraction error types
#[derive(Debug)]
pub enum ExtractError {
    Io(String),
    Zip(String),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::Io(e) => write!(f, "IO error: {}", e),
            ExtractError::Zip(e) => write!(f, "ZIP error: {}", e),
        }
    }
}

/// Extract a ZIP file to a destination directory
///
/// # Arguments
/// * `zip_path` - Path to the ZIP file
/// * `dest_dir` - Destination directory (will be created if needed)
///
/// # Notes
/// - Flattens nested directory structures (strips top-level folder)
/// - Preserves file permissions where possible
/// - Skips __MACOSX and .DS_Store files
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), ExtractError> {
    println!("[Extractor] Extracting {:?} to {:?}", zip_path, dest_dir);

    // Create destination directory
    fs::create_dir_all(dest_dir).map_err(|e| ExtractError::Io(e.to_string()))?;

    // Open the ZIP file
    let file = File::open(zip_path).map_err(|e| ExtractError::Io(e.to_string()))?;
    
    // Read the entire file into memory for the zip reader
    let mut buffer = Vec::new();
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut buffer).map_err(|e| ExtractError::Io(e.to_string()))?;

    // Parse as ZIP
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(buffer))
        .map_err(|e| ExtractError::Zip(e.to_string()))?;

    // Detect if there's a common top-level folder to strip
    let strip_prefix = detect_common_prefix(&mut archive);
    if let Some(ref prefix) = strip_prefix {
        println!("[Extractor] Stripping prefix: {:?}", prefix);
    }

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ExtractError::Zip(e.to_string()))?;

        let file_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue, // Skip invalid paths
        };

        // Skip macOS metadata files
        let path_str = file_path.to_string_lossy();
        if path_str.contains("__MACOSX") || path_str.contains(".DS_Store") {
            continue;
        }

        // Strip the common prefix if detected
        let relative_path = if let Some(ref prefix) = strip_prefix {
            if let Ok(stripped) = file_path.strip_prefix(prefix) {
                stripped.to_path_buf()
            } else {
                file_path.clone()
            }
        } else {
            file_path.clone()
        };

        // Skip if path is now empty (the top-level folder itself)
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let out_path = dest_dir.join(&relative_path);

        if file.is_dir() {
            // Create directory
            fs::create_dir_all(&out_path).map_err(|e| ExtractError::Io(e.to_string()))?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| ExtractError::Io(e.to_string()))?;
                }
            }

            // Extract file
            let mut out_file =
                File::create(&out_path).map_err(|e| ExtractError::Io(e.to_string()))?;

            io::copy(&mut file, &mut out_file).map_err(|e| ExtractError::Io(e.to_string()))?;

            println!("[Extractor] Extracted: {:?}", relative_path);
        }

        // Set permissions on Unix (no-op on Windows, but good practice)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    println!("[Extractor] Extraction complete");
    Ok(())
}

/// Detect if all files in the ZIP share a common top-level folder
fn detect_common_prefix(archive: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>) -> Option<std::path::PathBuf> {
    if archive.is_empty() {
        return None;
    }

    // Get the first path component of all files
    let mut common_prefix: Option<std::path::PathBuf> = None;
    
    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let path = match file.enclosed_name() {
            Some(p) => p.to_owned(),
            None => continue,
        };

        // Skip macOS metadata
        let path_str = path.to_string_lossy();
        if path_str.contains("__MACOSX") {
            continue;
        }

        // Get first component
        let mut components = path.components();
        if let Some(first) = components.next() {
            let first_path = std::path::PathBuf::from(first.as_os_str());
            
            match &common_prefix {
                None => common_prefix = Some(first_path),
                Some(existing) => {
                    if *existing != first_path {
                        // Files have different top-level folders, don't strip
                        return None;
                    }
                }
            }
        }
    }

    // Only return prefix if it looks like a folder (all files are inside it)
    // Check that there's at least one file with more than just the prefix
    if let Some(ref prefix) = common_prefix {
        for i in 0..archive.len() {
            let file = match archive.by_index(i) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let path = match file.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };
            if path.components().count() > 1 {
                return Some(prefix.clone());
            }
        }
    }

    None
}

/// Clean up a temporary ZIP file after extraction
pub fn cleanup_temp_file(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
