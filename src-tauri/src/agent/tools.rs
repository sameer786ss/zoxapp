use serde::Serialize;
use ignore::WalkBuilder;
use crate::agent::workspace::WorkspaceManager;

// --- Tool Trait ---
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> &str;
    fn requires_approval(&self) -> bool { false }
    fn execute(&self, args: &str, workspace: &WorkspaceManager) -> String;
}

// --- Tool Implementations ---

struct ReadFileTool;
impl Tool for ReadFileTool {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read file content from the workspace" }
    fn input_schema(&self) -> &str { r#"{"path": "string"}"# }
    fn execute(&self, args: &str, workspace: &WorkspaceManager) -> String {
        match serde_json::from_str::<serde_json::Value>(args) {
            Ok(v) => {
                let rel_path = v["path"].as_str().unwrap_or(args);
                // Sanitize: strip quotes and whitespace
                let clean_path = rel_path.trim().trim_matches('"');
                
                match workspace.resolve_path(clean_path) {
                    Ok(path) => {
                        std::fs::read_to_string(&path)
                            .unwrap_or_else(|e| format!("Error reading '{}': {}", path.display(), e))
                    },
                    Err(e) => format!("Error: {}", e)
                }
            },
            Err(_) => {
                // Fallback for non-JSON string
                let clean_path = args.trim().trim_matches('"');
                match workspace.resolve_path(clean_path) {
                    Ok(path) => {
                        std::fs::read_to_string(&path)
                            .unwrap_or_else(|e| format!("Error reading '{}': {}", path.display(), e))
                    },
                    Err(e) => format!("Error: {}", e)
                }
            }
        }
    }
}

struct WriteFileTool;
impl Tool for WriteFileTool {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Write content to a file in the workspace" }
    fn input_schema(&self) -> &str { r#"{"path": "string", "content": "string"}"# }
    fn requires_approval(&self) -> bool { true }
    fn execute(&self, args: &str, workspace: &WorkspaceManager) -> String {
        match serde_json::from_str::<serde_json::Value>(args) {
            Ok(v) => {
                let rel_path = match v["path"].as_str() {
                    Some(p) => p,
                    None => return "Error: 'path' field required".to_string()
                };
                let content = match v["content"].as_str() {
                    Some(c) => c,
                    None => return "Error: 'content' field required".to_string()
                };
                
                match workspace.resolve_path(rel_path) {
                    Ok(path) => {
                        println!("[WriteFileTool] Writing to: {}", path.display());
                        // Create parent directories if needed
                        if let Some(parent) = path.parent() {
                            if !parent.exists() {
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    return format!("Error creating directories: {}", e);
                                }
                            }
                        }
                        
                        match std::fs::write(&path, content) {
                            Ok(_) => format!("Successfully wrote {} bytes to {}", content.len(), path.display()),
                            Err(e) => format!("Error writing to '{}': {}", path.display(), e)
                        }
                    },
                    Err(e) => format!("Error: {}", e)
                }
            },
            Err(e) => format!("Invalid JSON args: {} (error: {})", args, e)
        }
    }
}

/// Replace specific lines in a file - for accurate partial edits
struct ReplaceLinesToolInner;
impl Tool for ReplaceLinesToolInner {
    fn name(&self) -> &str { "replace_lines" }
    fn description(&self) -> &str { "Replace specific line range in a file. Lines are 1-indexed." }
    fn input_schema(&self) -> &str { r#"{"path": "string", "start_line": "number", "end_line": "number", "new_content": "string"}"# }
    fn requires_approval(&self) -> bool { true }
    fn execute(&self, args: &str, workspace: &WorkspaceManager) -> String {
        match serde_json::from_str::<serde_json::Value>(args) {
            Ok(v) => {
                let rel_path = match v["path"].as_str() {
                    Some(p) => p,
                    None => return "Error: 'path' field required".to_string()
                };
                let start_line = v["start_line"].as_u64().unwrap_or(1) as usize;
                let end_line = v["end_line"].as_u64().unwrap_or(start_line as u64) as usize;
                let new_content = v["new_content"].as_str().unwrap_or("");
                
                if start_line == 0 || end_line == 0 {
                    return "Error: Lines are 1-indexed, cannot be 0".to_string();
                }
                if start_line > end_line {
                    return "Error: start_line cannot be greater than end_line".to_string();
                }
                
                match workspace.resolve_path(rel_path) {
                    Ok(path) => {
                        // Read the file
                        let content = match std::fs::read_to_string(&path) {
                            Ok(c) => c,
                            Err(e) => return format!("Error reading file: {}", e)
                        };
                        
                        let lines: Vec<&str> = content.lines().collect();
                        let total_lines = lines.len();
                        
                        if start_line > total_lines {
                            return format!("Error: start_line {} exceeds file length {}", start_line, total_lines);
                        }
                        
                        // Build new content
                        let mut result = Vec::new();
                        
                        // Lines before the replacement
                        for i in 0..(start_line - 1) {
                            result.push(lines[i].to_string());
                        }
                        
                        // Add the new content
                        for line in new_content.lines() {
                            result.push(line.to_string());
                        }
                        
                        // Lines after the replacement
                        let end_idx = std::cmp::min(end_line, total_lines);
                        for i in end_idx..total_lines {
                            result.push(lines[i].to_string());
                        }
                        
                        let final_content = result.join("\n");
                        
                        match std::fs::write(&path, &final_content) {
                            Ok(_) => format!(
                                "Replaced lines {}-{} in {}. File now has {} lines.", 
                                start_line, end_idx, path.display(), result.len()
                            ),
                            Err(e) => format!("Error writing file: {}", e)
                        }
                    },
                    Err(e) => format!("Error: {}", e)
                }
            },
            Err(e) => format!("Invalid JSON args: {}", e)
        }
    }
}


struct SearchProjectTool;
impl Tool for SearchProjectTool {
    fn name(&self) -> &str { "search_project" }
    fn description(&self) -> &str { "Search the workspace for text patterns" }
    fn input_schema(&self) -> &str { r#"{"query": "string"}"# }
    fn execute(&self, args: &str, workspace: &WorkspaceManager) -> String {
        match serde_json::from_str::<serde_json::Value>(args) {
            Ok(v) => {
                let query = v["query"].as_str().unwrap_or("");
                
                if query.trim().is_empty() {
                    return "Error: 'query' cannot be empty".to_string();
                }
                
                let path = workspace.get_workspace_dir();
                println!("[SearchTool] Searching in: {} for query: '{}'", path.display(), query);
                
                let walker = WalkBuilder::new(path)
                    .hidden(false)
                    .git_ignore(true)
                    .build();
                
                let mut output = String::new();
                let mut match_count = 0;
                let max_matches = 50;
                let mut files_scanned = 0;
                
                for entry in walker.flatten() {
                    let file_path = entry.path();
                    if !file_path.is_file() { continue; }
                    files_scanned += 1;

                    let relative = file_path.strip_prefix(path).unwrap_or(file_path);
                    let relative_str = relative.to_string_lossy();
                    
                    // Check filename match
                    if relative_str.to_lowercase().contains(&query.to_lowercase()) {
                        if match_count >= max_matches { break; }
                        output.push_str(&format!(
                            "Filename match: {}\n",
                            relative.display()
                        ));
                        match_count += 1;
                    }
                    
                    if let Ok(content) = std::fs::read_to_string(file_path) {
                        for (line_num, line) in content.lines().enumerate() {
                            if match_count >= max_matches { break; }
                            
                            if line.to_lowercase().contains(&query.to_lowercase()) {
                                output.push_str(&format!(
                                    "{}:{}:{}\n",
                                    relative.display(),
                                    line_num + 1,
                                    line.chars().take(100).collect::<String>()
                                ));
                                match_count += 1;
                            }
                        }
                    }
                    if match_count >= max_matches { break; }
                }
                
                println!("[SearchTool] Scanned {} files, found {} matches", files_scanned, match_count);

                if output.is_empty() {
                    format!("No matches found for '{}' (scanned {} files in {})", query, files_scanned, path.display())
                } else {
                    format!("Found {} matches:\n{}", match_count, output)
                }
            },
            Err(e) => format!("Invalid JSON: {}", e)
        }
    }
}

struct ListFilesTool;
impl Tool for ListFilesTool {
    fn name(&self) -> &str { "list_files" }
    fn description(&self) -> &str { "List files and directories" }
    fn input_schema(&self) -> &str { r#"{"path": "string"}"# }
    fn execute(&self, args: &str, workspace: &WorkspaceManager) -> String {
        let rel_path = match serde_json::from_str::<serde_json::Value>(args) {
            Ok(v) => v["path"].as_str().unwrap_or(".").to_string(),
            Err(_) => ".".to_string()
        };
        
        match workspace.resolve_path(&rel_path) {
            Ok(path) => {
                if !path.is_dir() {
                    return format!("Error: '{}' is not a directory", path.display());
                }
                
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let mut output = format!("Contents of {}:\n", path.display());
                        let mut items = Vec::new();
                        
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let file_type = entry.file_type().ok();
                            
                            let prefix = if file_type.map_or(false, |t| t.is_dir()) {
                                "📁 "
                            } else {
                                "📄 "
                            };
                            
                            items.push(format!("{}{}", prefix, name));
                        }
                        
                        items.sort();
                        output.push_str(&items.join("\n"));
                        output
                    }
                    Err(e) => format!("Error reading directory: {}", e)
                }
            }
            Err(e) => format!("Error: {}", e)
        }
    }
}

pub fn get_tool_by_name(name: &str) -> Option<Box<dyn Tool>> {
    match name {
        "read_file" => Some(Box::new(ReadFileTool)),
        "write_file" => Some(Box::new(WriteFileTool)),
        "replace_lines" => Some(Box::new(ReplaceLinesToolInner)),
        "search_project" => Some(Box::new(SearchProjectTool)),
        "list_files" => Some(Box::new(ListFilesTool)),
        _ => None
    }
}

pub fn get_all_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file".into(),
            description: "Read content of a file. Path is relative to workspace.".into(),
            input_schema: r#"{"type": "object", "properties": {"path": {"type": "string"}}}"#.into(),
        },
        ToolDefinition {
            name: "write_file".into(),
            description: "Write content to a file. Creates directories if needed. Requires APPROVAL.".into(),
            input_schema: r#"{"type": "object", "properties": {"path": {"type": "string"}, "content": {"type": "string"}}}"#.into(),
        },
        ToolDefinition {
            name: "replace_lines".into(),
            description: "Replace specific line range in a file. Lines are 1-indexed. Use for precise edits. Requires APPROVAL.".into(),
            input_schema: r#"{"type": "object", "properties": {"path": {"type": "string"}, "start_line": {"type": "number"}, "end_line": {"type": "number"}, "new_content": {"type": "string"}}}"#.into(),
        },
        ToolDefinition {
            name: "search_project".into(),
            description: "Search the workspace for a text pattern.".into(),
            input_schema: r#"{"type": "object", "properties": {"query": {"type": "string"}}}"#.into(),
        },
        ToolDefinition {
            name: "list_files".into(),
            description: "List files and directories in a path.".into(),
            input_schema: r#"{"type": "object", "properties": {"path": {"type": "string"}}}"#.into(),
        },
    ]
}

#[derive(Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: String, 
}
