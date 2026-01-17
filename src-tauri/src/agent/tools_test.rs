//! Tool Tests
//!
//! Unit tests for agent tool implementations.

#[cfg(test)]
mod tests {
    use crate::agent::tools::{get_all_tools, get_tool_by_name, Tool};
    use crate::agent::workspace::WorkspaceManager;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    fn create_test_workspace() -> WorkspaceManager {
        // Create a minimal workspace for testing
        WorkspaceManager::empty()
    }
    
    #[test]
    fn test_get_all_tools_returns_expected_tools() {
        let tools = get_all_tools();
        
        assert!(!tools.is_empty(), "Should have at least one tool");
        
        // Check expected tools exist
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"read_file"), "Should have read_file tool");
        assert!(tool_names.contains(&"write_file"), "Should have write_file tool");
        assert!(tool_names.contains(&"list_files"), "Should have list_files tool");
        assert!(tool_names.contains(&"search_project"), "Should have search_project tool");
    }
    
    #[test]
    fn test_get_tool_by_name_found() {
        let tool = get_tool_by_name("read_file");
        assert!(tool.is_some(), "read_file tool should exist");
        
        let tool = tool.unwrap();
        assert_eq!(tool.name(), "read_file");
        assert!(!tool.description().is_empty());
        assert!(!tool.input_schema().is_empty());
    }
    
    #[test]
    fn test_get_tool_by_name_not_found() {
        let tool = get_tool_by_name("nonexistent_tool");
        assert!(tool.is_none(), "Nonexistent tool should return None");
    }
    
    #[test]
    fn test_read_file_tool_requires_approval() {
        let tool = get_tool_by_name("read_file").unwrap();
        assert!(!tool.requires_approval(), "read_file should not require approval");
    }
    
    #[test]
    fn test_write_file_tool_requires_approval() {
        let tool = get_tool_by_name("write_file").unwrap();
        assert!(tool.requires_approval(), "write_file should require approval");
    }
    
    #[test]
    fn test_replace_lines_tool_requires_approval() {
        let tool = get_tool_by_name("replace_lines").unwrap();
        assert!(tool.requires_approval(), "replace_lines should require approval");
    }
    
    #[test]
    fn test_read_file_invalid_json() {
        let tool = get_tool_by_name("read_file").unwrap();
        let workspace = create_test_workspace();
        
        let result = tool.execute("not valid json", &workspace);
        assert!(result.contains("Error") || result.contains("error"), 
                "Should return error for invalid JSON");
    }
    
    #[test]
    fn test_read_file_missing_path() {
        let tool = get_tool_by_name("read_file").unwrap();
        let workspace = create_test_workspace();
        
        let result = tool.execute(r#"{}"#, &workspace);
        assert!(result.contains("Error") || result.contains("error") || result.contains("path"), 
                "Should return error for missing path");
    }
    
    #[test]
    fn test_list_files_invalid_path() {
        let tool = get_tool_by_name("list_files").unwrap();
        let workspace = create_test_workspace();
        
        let result = tool.execute(r#"{"path": "/nonexistent/path/that/does/not/exist"}"#, &workspace);
        // Should either error or return empty list
        assert!(!result.is_empty(), "Should return some response");
    }
    
    #[test]
    fn test_search_project_requires_query() {
        let tool = get_tool_by_name("search_project").unwrap();
        let workspace = create_test_workspace();
        
        let result = tool.execute(r#"{}"#, &workspace);
        assert!(result.contains("Error") || result.contains("error") || result.contains("query"),
                "Should return error for missing query");
    }
    
    #[test]
    fn test_tool_descriptions_not_empty() {
        for tool_def in get_all_tools() {
            if let Some(tool) = get_tool_by_name(&tool_def.name) {
                assert!(!tool.description().is_empty(), 
                        "Tool {} should have a description", tool_def.name);
                assert!(!tool.input_schema().is_empty(),
                        "Tool {} should have an input schema", tool_def.name);
            }
        }
    }
}
