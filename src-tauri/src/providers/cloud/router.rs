//! Model Router
//! 
//! Uses Gemma 3 1B to classify requests as SIMPLE or COMPLEX.
//! Fast classification (< 200ms) to determine which model to use.

use super::client::GemmaClient;
use crate::providers::{ModelTier, Complexity};

/// Model router using 1B for classification
pub struct ModelRouter {
    client: GemmaClient,
}

impl ModelRouter {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            client: GemmaClient::new(keys, ModelTier::Router),
        }
    }
    
    /// Classify input complexity
    /// 
    /// Returns:
    /// - Simple: use 4B for chat
    /// - Complex: use 12B for chat, 27B for turbo
    pub async fn classify(&self, input: &str) -> Result<Complexity, String> {
        println!("[Router] Classifying input...");
        
        // Skip classification for very short inputs (greetings)
        if input.len() < 20 {
            println!("[Router] Short input → SIMPLE");
            return Ok(Complexity::Simple);
        }
        
        // Quick heuristics before using the model
        let lower = input.to_lowercase();
        
        // Code-related keywords → COMPLEX
        let complex_keywords = [
            "create", "write", "modify", "edit", "fix", "debug",
            "implement", "build", "code", "function", "class",
            "file", "folder", "directory", "install", "run",
            "error", "bug", "issue", "why", "how does",
            "explain", "analyze", "compare", "difference",
        ];
        
        for keyword in complex_keywords {
            if lower.contains(keyword) {
                println!("[Router] Keyword '{}' → COMPLEX", keyword);
                return Ok(Complexity::Complex);
            }
        }
        
        // Simple patterns → SIMPLE
        let simple_patterns = [
            "hi", "hello", "hey", "thanks", "thank you",
            "bye", "goodbye", "ok", "okay", "yes", "no",
            "what is", "who is", "when",
        ];
        
        for pattern in simple_patterns {
            if lower.starts_with(pattern) || lower == pattern {
                println!("[Router] Pattern '{}' → SIMPLE", pattern);
                return Ok(Complexity::Simple);
            }
        }
        
        // Use 1B model for uncertain cases
        match self.client.classify(input).await {
            Ok(result) => {
                let complexity = if result.contains("COMPLEX") {
                    Complexity::Complex
                } else {
                    Complexity::Simple
                };
                println!("[Router] Model classified → {:?}", complexity);
                Ok(complexity)
            }
            Err(e) => {
                // On error, default to Simple (use 4B)
                println!("[Router] Classification error: {}, defaulting to SIMPLE", e);
                Ok(Complexity::Simple)
            }
        }
    }
}
