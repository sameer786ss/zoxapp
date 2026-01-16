use tauri::command;
use std::fs;

#[command]
pub fn fast_read_file(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[command]
pub fn fast_write_file(path: String, content: String) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

#[command]
pub fn get_file_outline(path: String) -> Result<String, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    
    // Mock Tree-Sitter Logic (Regex Fallback for "No Run" Safety)
    let mut outline = String::new();
    outline.push_str(&format!("Outline for {}:\n", path));
    
    for line in content.lines() {
        let trimmed = line.trim();
        // Simple heuristic for Rust/TS/JS definitions
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || 
           trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") ||
           trimmed.starts_with("impl ") || trimmed.starts_with("export function ") ||
           trimmed.starts_with("export interface ") || trimmed.starts_with("class ") {
            outline.push_str(line);
            outline.push('\n');
        }
    }
    
    if outline.len() <= 20 {
        Ok("No definitions found.".to_string())
    } else {
        Ok(outline)
    }
}
