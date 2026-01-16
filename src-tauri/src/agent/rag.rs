//! RAG (Retrieval Augmented Generation) System
//! 
//! Simplified in-memory implementation with semantic search.
//! Uses fastembed for embeddings and cosine similarity for search.

use std::path::Path;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use fastembed::TextEmbedding;

/// A stored context chunk with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub chunk_type: String,
    pub source: String,
    pub timestamp: i64,
}

/// In-memory context store
struct ContextStore {
    chunks: Vec<ContextChunk>,
}

impl ContextStore {
    fn new() -> Self {
        Self { chunks: Vec::new() }
    }
    
    fn add(&mut self, chunk: ContextChunk) {
        self.chunks.push(chunk);
    }
    
    fn search(&self, query_embedding: &[f32], limit: usize) -> Vec<&ContextChunk> {
        // Calculate cosine similarity for all chunks
        let mut scored: Vec<(&ContextChunk, f32)> = self.chunks
            .iter()
            .map(|chunk| {
                let similarity = cosine_similarity(query_embedding, &chunk.embedding);
                (chunk, similarity)
            })
            .collect();
        
        // Sort by similarity (highest first)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top results
        scored.into_iter()
            .take(limit)
            .map(|(chunk, _)| chunk)
            .collect()
    }
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

/// RAG Context Manager with in-memory storage
pub struct ContextManager {
    store: Mutex<ContextStore>,
    embedder: Mutex<TextEmbedding>,
    initialized: bool,
}

impl ContextManager {
    /// Initialize the RAG context manager
    pub async fn new(_data_dir: &Path) -> Result<Self, String> {
        println!("[RAG] Initializing embedder...");
        
        // Initialize embedding model (all-MiniLM-L6-v2)
        let embedder = TextEmbedding::try_new(Default::default())
            .map_err(|e| format!("Failed to initialize embedder: {}", e))?;
        
        println!("[RAG] Context manager initialized (in-memory)");
        
        Ok(Self {
            store: Mutex::new(ContextStore::new()),
            embedder: Mutex::new(embedder),
            initialized: true,
        })
    }
    
    /// Check if RAG is properly initialized
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
    
    /// Embed text using the embedding model
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        if text.trim().is_empty() {
            return Ok(vec![]);
        }
        
        let mut embedder = self.embedder.lock();
        let embeddings = embedder.embed(vec![text.to_string()], None)
            .map_err(|e| format!("Embedding failed: {}", e))?;
        
        if embeddings.is_empty() {
            return Err("No embeddings generated".to_string());
        }
        
        Ok(embeddings[0].clone())
    }
    
    /// Store a chunk in the context
    pub async fn store_chunk(
        &self,
        content: &str,
        chunk_type: &str,
        source: &str,
    ) -> Result<(), String> {
        if content.trim().is_empty() || content.len() < 10 {
            return Ok(()); // Skip trivial content
        }
        
        // Truncate for embedding (models have limits)
        let truncated: String = content.chars().take(512).collect();
        
        // Generate embedding
        let embedding = self.embed(&truncated)?;
        if embedding.is_empty() {
            return Ok(());
        }
        
        let chunk = ContextChunk {
            id: uuid::Uuid::new_v4().to_string(),
            content: truncated,
            embedding,
            chunk_type: chunk_type.to_string(),
            source: source.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        // Store in memory
        let mut store = self.store.lock();
        store.add(chunk);
        
        println!("[RAG] Stored {} chunk ({} chars, {} total)", 
            chunk_type, content.len(), store.chunks.len());
        
        Ok(())
    }
    
    /// Store a message in the context
    pub async fn store_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), String> {
        let source = format!("{}:{}", conversation_id, role);
        self.store_chunk(content, "message", &source).await
    }
    
    /// Store a tool result
    pub async fn store_tool_result(
        &self,
        tool_name: &str,
        result: &str,
    ) -> Result<(), String> {
        self.store_chunk(result, "tool_result", tool_name).await
    }
    
    /// Search for relevant context given a query
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, String> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        
        // Embed query
        let query_embedding = self.embed(query)?;
        if query_embedding.is_empty() {
            return Ok(vec![]);
        }
        
        // Search in store
        let store = self.store.lock();
        let results = store.search(&query_embedding, limit);
        
        let contents: Vec<String> = results
            .iter()
            .map(|chunk| chunk.content.clone())
            .collect();
        
        println!("[RAG] Search found {} results for: {}", 
            contents.len(), 
            query.chars().take(30).collect::<String>());
        
        Ok(contents)
    }
    
    /// Get summarized context for a query
    pub async fn get_relevant_context(
        &self,
        query: &str,
        max_chunks: usize,
    ) -> Result<String, String> {
        let chunks = self.search(query, max_chunks).await?;
        
        if chunks.is_empty() {
            return Ok(String::new());
        }
        
        Ok(chunks.join("\n---\n"))
    }
}

/// Lazy-initialized global RAG manager
pub struct RagManager {
    context_manager: Option<ContextManager>,
}

impl RagManager {
    pub fn new() -> Self {
        Self {
            context_manager: None,
        }
    }
    
    /// Initialize the RAG system
    pub async fn initialize(&mut self, data_dir: &Path) -> Result<(), String> {
        match ContextManager::new(data_dir).await {
            Ok(cm) => {
                self.context_manager = Some(cm);
                Ok(())
            }
            Err(e) => {
                eprintln!("[RAG] Failed to initialize: {}", e);
                Err(e)
            }
        }
    }
    
    /// Get reference to context manager
    pub fn get(&self) -> Option<&ContextManager> {
        self.context_manager.as_ref()
    }
    
    /// Check if RAG is initialized
    pub fn is_initialized(&self) -> bool {
        self.context_manager.is_some()
    }
}

impl Default for RagManager {
    fn default() -> Self {
        Self::new()
    }
}
