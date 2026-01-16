pub mod actor;
pub mod parser;
pub mod memory;
pub mod rag;
pub mod summarizer;
pub mod tools;
pub mod workspace;
pub mod history;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub temperature: f32,
}
