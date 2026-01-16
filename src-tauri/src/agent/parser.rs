//! XML Response Parser for Gemma 3
//! 
//! Aggressively finds XML tags for tools and content.
//! Gemma 3 outputs cleaner XML than JSON (less "chatty").

use serde_json::Value;
use std::collections::HashMap;

/// Parsed result from the model's response
#[derive(Debug, Clone)]
pub enum ParsedResponse {
    /// Plain text response (no tool call)
    Text(String),
    
    /// Tool call with optional thinking
    ToolCall {
        thinking: Option<String>,
        tool: String,
        parameters: Value,
    },
    
    /// Mixed: some text followed by a tool call
    TextThenTool {
        text: String,
        thinking: Option<String>,
        tool: String,
        parameters: Value,
    },
}

/// Aggressive XML parser for Gemma 3 responses
/// Searches for <tool>, <thinking>, <message> tags
pub struct ResponseParser;

impl ResponseParser {
    /// Parse a complete response string
    /// Aggressively searches for XML tool calls
    pub fn parse(response: &str) -> ParsedResponse {
        let cleaned = Self::clean_response(response);
        
        // Try to find tool XML in the response
        if let Some(tool_data) = Self::find_tool_xml(&cleaned) {
            let (before_xml, thinking, tool_name, params) = tool_data;
            
            if before_xml.trim().is_empty() {
                // Pure tool response
                ParsedResponse::ToolCall {
                    thinking,
                    tool: tool_name,
                    parameters: params,
                }
            } else {
                // Text before tool
                ParsedResponse::TextThenTool {
                    text: before_xml.trim().to_string(),
                    thinking,
                    tool: tool_name,
                    parameters: params,
                }
            }
        } else {
            // No tool call found - extract message content or use cleaned text
            let text = Self::extract_message_content(&cleaned)
                .unwrap_or(cleaned);
            
            // Final cleanup: strip any remaining XML-like tags
            let final_text = Self::strip_remaining_tags(&text);
            ParsedResponse::Text(final_text)
        }
    }
    
    /// Check if the response contains a tool call
    pub fn has_tool_call(response: &str) -> bool {
        let cleaned = Self::clean_response(response);
        Self::find_tool_xml(&cleaned).is_some()
    }
    
    /// Extract just the tool name if present
    pub fn extract_tool_name(response: &str) -> Option<String> {
        let cleaned = Self::clean_response(response);
        Self::find_tool_xml(&cleaned).map(|(_, _, name, _)| name)
    }
    
    /// Clean up model response - remove markdown fences, thinking tags, and extract message
    fn clean_response(response: &str) -> String {
        let mut cleaned = response.to_string();
        
        // Remove markdown code fences
        let fences = ["```xml", "```XML", "```json", "```JSON", "```", "~~~xml", "~~~"];
        for fence in fences {
            cleaned = cleaned.replace(fence, "");
        }
        
        // Strip <thinking>...</thinking> tags completely
        while let (Some(start), Some(end)) = (cleaned.find("<thinking>"), cleaned.find("</thinking>")) {
            if start < end {
                let before = &cleaned[..start];
                let after = &cleaned[end + 11..]; // 11 = "</thinking>".len()
                cleaned = format!("{}{}", before.trim(), after.trim());
            } else {
                break;
            }
        }
        
        // Normalize whitespace
        cleaned.trim().to_string()
    }
    
    /// Find XML tool call in response
    /// Returns (text_before, thinking, tool_name, parameters) if found
    fn find_tool_xml(response: &str) -> Option<(String, Option<String>, String, Value)> {
        // Look for <tool> tag
        let tool_start = response.find("<tool>")?;
        let tool_end = response.find("</tool>")?;
        
        if tool_start >= tool_end {
            return None;
        }
        
        let tool_name = response[tool_start + 6..tool_end].trim().to_string();
        if tool_name.is_empty() {
            return None;
        }
        
        let before_xml = &response[..tool_start];
        
        // Extract thinking if present
        let thinking = Self::extract_tag_content(response, "thinking");
        
        // Extract parameters - try <params> or <parameters>
        let params_str = Self::extract_tag_content(response, "params")
            .or_else(|| Self::extract_tag_content(response, "parameters"))
            .unwrap_or_default();
        
        // Parse parameters into Value
        let parameters = Self::parse_params(&params_str);
        
        Some((before_xml.to_string(), thinking, tool_name, parameters))
    }
    
    /// Extract content between <tag> and </tag>
    fn extract_tag_content(response: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);
        
        let start = response.find(&open_tag)?;
        let end = response.find(&close_tag)?;
        
        if start >= end {
            return None;
        }
        
        let content = response[start + open_tag.len()..end].trim().to_string();
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }
    
    /// Extract <message> content for plain text responses
    fn extract_message_content(response: &str) -> Option<String> {
        Self::extract_tag_content(response, "message")
            .or_else(|| Self::extract_tag_content(response, "response"))
            .or_else(|| Self::extract_tag_content(response, "content"))
    }
    
    /// Strip any remaining XML-like tags that shouldn't be shown to user
    fn strip_remaining_tags(text: &str) -> String {
        let mut result = text.to_string();
        
        // Tags that should never appear in final output
        let tags_to_strip = ["thinking", "message", "response", "content", "output"];
        
        for tag in tags_to_strip {
            let open = format!("<{}>", tag);
            let close = format!("</{}>", tag);
            result = result.replace(&open, "").replace(&close, "");
        }
        
        result.trim().to_string()
    }
    
    /// Parse parameter string into JSON Value
    /// Supports nested XML tags or key="value" pairs
    fn parse_params(params_str: &str) -> Value {
        let mut map: HashMap<String, Value> = HashMap::new();
        
        // Try parsing nested XML tags like <path>file.txt</path>
        let mut remaining = params_str;
        while let Some(tag_start) = remaining.find('<') {
            let tag_name_end = remaining[tag_start..].find('>')
                .map(|i| tag_start + i)
                .unwrap_or(remaining.len());
            
            let tag_name = &remaining[tag_start + 1..tag_name_end];
            if tag_name.starts_with('/') || tag_name.is_empty() {
                remaining = &remaining[tag_name_end.min(remaining.len() - 1) + 1..];
                continue;
            }
            
            let close_tag = format!("</{}>", tag_name);
            if let Some(close_pos) = remaining.find(&close_tag) {
                let value_start = tag_name_end + 1;
                if value_start < close_pos {
                    let value = remaining[value_start..close_pos].trim();
                    
                    // Try to parse as number or use as string
                    if let Ok(num) = value.parse::<i64>() {
                        map.insert(tag_name.to_string(), Value::Number(num.into()));
                    } else if let Ok(num) = value.parse::<f64>() {
                        map.insert(tag_name.to_string(), Value::Number(serde_json::Number::from_f64(num).unwrap_or(0.into())));
                    } else {
                        map.insert(tag_name.to_string(), Value::String(value.to_string()));
                    }
                }
                remaining = &remaining[close_pos + close_tag.len()..];
            } else {
                remaining = &remaining[tag_name_end.min(remaining.len() - 1) + 1..];
            }
        }
        
        if map.is_empty() {
            Value::Object(Default::default())
        } else {
            serde_json::to_value(map).unwrap_or(Value::Object(Default::default()))
        }
    }
    
    /// Find the matching closing brace (kept for compatibility)
    pub fn find_matching_brace(s: &str) -> Option<usize> {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        for (i, c) in s.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        
        None
    }
    
    /// Validate tool name is not empty or whitespace-only
    pub fn is_valid_tool_name(name: &str) -> bool {
        !name.trim().is_empty()
    }
}

/// Streaming parser for incremental processing
/// Supports both chat mode (text/message) and turbo mode (tool calls)
pub struct StreamingParser {
    buffer: String,
    emitted_text: usize,
    is_turbo_mode: bool,
    tool_detected: bool,
}

/// Events emitted during streaming
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Text content (safe to display)
    Text(String),
    
    /// Tool call complete with parameters (only emitted when fully parsed)
    ToolCallComplete {
        thinking: Option<String>,
        tool: String,
        parameters: Value,
    },
}

impl StreamingParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            emitted_text: 0,
            is_turbo_mode: false,
            tool_detected: false,
        }
    }
    
    /// Create parser in turbo mode (expects tool calls)
    pub fn new_turbo() -> Self {
        Self {
            buffer: String::new(),
            emitted_text: 0,
            is_turbo_mode: true,
            tool_detected: false,
        }
    }
    
    /// Set the parsing mode
    pub fn set_turbo_mode(&mut self, turbo: bool) {
        self.is_turbo_mode = turbo;
    }
    
    /// Feed a chunk of streamed text and get events
    /// 
    /// Chat mode: Emits text incrementally, strips <message> tags
    /// Turbo mode: Waits for complete <tool>...</tool> before emitting
    pub fn feed(&mut self, chunk: &str) -> Vec<StreamEvent> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();
        
        // Clean buffer for analysis
        let cleaned = ResponseParser::clean_response(&self.buffer);
        
        if self.is_turbo_mode {
            // TURBO MODE: Look for complete tool calls only
            self.process_turbo_mode(&cleaned, &mut events);
        } else {
            // CHAT MODE: Emit text incrementally
            self.process_chat_mode(&cleaned, &mut events);
        }
        
        events
    }
    
    /// Process in turbo mode - wait for complete tool tags
    fn process_turbo_mode(&mut self, cleaned: &str, events: &mut Vec<StreamEvent>) {
        // Check if we have BOTH <tool> and </tool>
        let has_tool_start = cleaned.contains("<tool>");
        let has_tool_end = cleaned.contains("</tool>");
        
        if has_tool_start {
            self.tool_detected = true;
        }
        
        if has_tool_start && has_tool_end {
            // Complete tool call - parse and emit
            if let Some((before, thinking, tool, params)) = ResponseParser::find_tool_xml(cleaned) {
                // Emit any text before the tool call
                if !before.trim().is_empty() && self.emitted_text == 0 {
                    events.push(StreamEvent::Text(before.trim().to_string()));
                }
                
                events.push(StreamEvent::ToolCallComplete {
                    thinking,
                    tool,
                    parameters: params,
                });
                
                // Mark as fully emitted
                self.emitted_text = cleaned.len();
            }
        } else if !self.tool_detected && !cleaned.contains('<') {
            // No tool detected yet and no XML starting - safe to emit text
            let new_text = &cleaned[self.emitted_text..];
            if !new_text.is_empty() {
                events.push(StreamEvent::Text(new_text.to_string()));
                self.emitted_text = cleaned.len();
            }
        }
        // If we see '<' but not complete tags, wait for more data
    }
    
    /// Process in chat mode - emit text incrementally
    fn process_chat_mode(&mut self, cleaned: &str, events: &mut Vec<StreamEvent>) {
        // Check for <message> tags
        if cleaned.contains("<message>") && cleaned.contains("</message>") {
            // Extract message content
            if let Some(content) = ResponseParser::extract_tag_content(cleaned, "message") {
                let new_portion = if self.emitted_text < content.len() {
                    &content[self.emitted_text..]
                } else {
                    ""
                };
                
                if !new_portion.is_empty() {
                    events.push(StreamEvent::Text(new_portion.to_string()));
                    self.emitted_text = content.len();
                }
            }
        } else if cleaned.contains("<message>") {
            // Partial <message> - wait for closing tag
            // Don't emit anything yet
        } else if !cleaned.contains('<') {
            // No XML at all - emit plain text
            let new_text = &cleaned[self.emitted_text..];
            if !new_text.is_empty() {
                events.push(StreamEvent::Text(new_text.to_string()));
                self.emitted_text = cleaned.len();
            }
        } else {
            // Some XML starting but not <message> - might be thinking or other
            // For chat mode, strip thinking and emit remaining text
            let stripped = ResponseParser::strip_remaining_tags(cleaned);
            let new_text = if self.emitted_text < stripped.len() {
                &stripped[self.emitted_text..]
            } else {
                ""
            };
            
            if !new_text.is_empty() {
                events.push(StreamEvent::Text(new_text.to_string()));
                self.emitted_text = stripped.len();
            }
        }
    }
    
    /// Get the final parsed result
    pub fn finalize(&self) -> ParsedResponse {
        ResponseParser::parse(&self.buffer)
    }
    
    /// Reset the parser for a new response
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.emitted_text = 0;
        self.tool_detected = false;
    }
    
    /// Get the current buffer content
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }
    
    fn try_parse_complete_xml(response: &str) -> Option<(String, Option<String>, String, Value)> {
        ResponseParser::find_tool_xml(response)
    }
}

impl Default for StreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_text() {
        let response = "Hello, I can help you with that!";
        let parsed = ResponseParser::parse(response);
        assert!(matches!(parsed, ParsedResponse::Text(_)));
    }

    #[test]
    fn test_pure_tool_call() {
        let response = r#"
<thinking>I need to read the file</thinking>
<tool>read_file</tool>
<params><path>test.txt</path></params>"#;
        let parsed = ResponseParser::parse(response);
        match parsed {
            ParsedResponse::ToolCall { thinking, tool, .. } => {
                assert_eq!(tool, "read_file");
                assert!(thinking.is_some());
            }
            _ => panic!("Expected ToolCall"),
        }
    }

    #[test]
    fn test_text_then_tool() {
        let response = r#"I'll read that file for you.
<thinking>User wants file contents</thinking>
<tool>read_file</tool>
<params><path>package.json</path></params>"#;
        let parsed = ResponseParser::parse(response);
        match parsed {
            ParsedResponse::TextThenTool { text, tool, .. } => {
                assert!(text.contains("I'll read"));
                assert_eq!(tool, "read_file");
            }
            _ => panic!("Expected TextThenTool"),
        }
    }

    #[test]
    fn test_message_extraction() {
        let response = "<message>Here is your answer!</message>";
        let parsed = ResponseParser::parse(response);
        match parsed {
            ParsedResponse::Text(text) => {
                assert_eq!(text, "Here is your answer!");
            }
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_params_parsing() {
        let response = r#"
<tool>write_file</tool>
<params>
<path>hello.txt</path>
<content>Hello World</content>
</params>"#;
        let parsed = ResponseParser::parse(response);
        match parsed {
            ParsedResponse::ToolCall { parameters, .. } => {
                assert_eq!(parameters["path"], "hello.txt");
                assert_eq!(parameters["content"], "Hello World");
            }
            _ => panic!("Expected ToolCall"),
        }
    }
}
