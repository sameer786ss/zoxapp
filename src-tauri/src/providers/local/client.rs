//! Local Model Provider using Candle ML Framework
//!
//! Provides local inference using GGUF models via Candle.
//! Supports CUDA (NVIDIA), Metal (Apple), and CPU fallback.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use futures::stream;

use crate::providers::{
    ModelProvider, CompletionResult, ProviderCapabilities, ModelTier, Message
};
use tauri::{AppHandle, Emitter};
use crate::setup::paths::get_model_path;

use candle_core::Device;
use candle_core::quantized::gguf_file::Content;
use candle_transformers::models::quantized_llama::ModelWeights;
use tokenizers::Tokenizer;

/// Chat mode system prompt
const LOCAL_CHAT_TEMPLATE: &str = r#"<|im_start|>system
You are ZOX, a helpful AI coding assistant. Be concise and accurate.
<|im_end|>
"#;

/// Turbo/Agent prompt template with strict XML tool output
const LOCAL_TURBO_TEMPLATE: &str = r#"<|im_start|>system
You are ZOX, an autonomous coding agent. You execute tools via strict XML format.

AVAILABLE TOOLS:
| Tool | Parameters |
|------|------------|
| read_file | <path>file_path</path> |
| write_file | <path>file_path</path><content>file_content</content> |
| list_files | <path>directory</path> |
| search_project | <query>search_term</query> |
| replace_lines | <path>file_path</path><start>line_num</start><end>line_num</end><content>new_content</content> |

RESPONSE FORMAT (STRICT XML):
<thinking>brief reasoning</thinking>
<tool>tool_name</tool>
<params>
<param_name>value</param_name>
</params>

For text responses WITHOUT tools:
<message>Your response here</message>
<|im_end|>
"#;

/// Provider state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderState {
    Unloaded,
    Loading,
    Ready,
    Error,
}

/// Loaded model and tokenizer
struct LoadedModel {
    weights: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
}

/// Local model provider using Candle GGUF
/// Named LocalLlamaProvider for compatibility with existing code
#[derive(Clone)]
pub struct LocalLlamaProvider {
    state: Arc<RwLock<ProviderState>>,
    model: Arc<RwLock<Option<LoadedModel>>>,
    model_path: Arc<RwLock<Option<PathBuf>>>,
    device_name: String,
    app_handle: Option<AppHandle>,
}

impl LocalLlamaProvider {
    /// Create new local provider
    pub fn new(app_handle: Option<AppHandle>) -> Self {
        let device_name = Self::detect_device_name();
        
        println!("[LocalLlamaProvider] Created with device: {}", device_name);
        
        Self {
            state: Arc::new(RwLock::new(ProviderState::Unloaded)),
            model: Arc::new(RwLock::new(None)),
            model_path: Arc::new(RwLock::new(None)),
            device_name,
            app_handle,
        }
    }
    
    /// Create with defaults (no handle initially)
    pub fn with_defaults() -> Self {
        Self::new(None)
    }

    /// Create with handle
    pub fn with_handle(handle: AppHandle) -> Self {
        Self::new(Some(handle))
    }
    
    /// Detect best available device name
    fn detect_device_name() -> String {
        #[cfg(feature = "cuda")]
        {
            if candle_core::utils::cuda_is_available() {
                return "CUDA".to_string();
            }
        }
        
        #[cfg(all(feature = "metal", target_os = "macos"))]
        {
            if candle_core::utils::metal_is_available() {
                return "Metal".to_string();
            }
        }
        
        "CPU".to_string()
    }
    
    /// Get the actual Device enum
    fn get_device() -> candle_core::Result<Device> {
        #[cfg(feature = "cuda")]
        {
            if candle_core::utils::cuda_is_available() {
                return Device::new_cuda(0);
            }
        }
        
        #[cfg(all(feature = "metal", target_os = "macos"))]
        {
            if candle_core::utils::metal_is_available() {
                return Device::new_metal(0);
            }
        }
        
        Ok(Device::Cpu)
    }
    
    /// Get device name
    pub fn device_name(&self) -> &str {
        &self.device_name
    }
    
    /// Check if model is loaded
    pub async fn is_loaded(&self) -> bool {
        let state = *self.state.read().await;
        println!("[LocalLlamaProvider] Checking state: {:?}", state);
        state == ProviderState::Ready
    }
    
    /// Load a GGUF model from the specified path
    pub async fn load_model(&self, model_path: PathBuf) -> Result<(), String> {
        println!("[LocalLlamaProvider] Loading model from: {:?}", model_path);
        
        // Logic check - if already loaded with same path, skip
        {
            let current_path = self.model_path.read().await;
            let current_state = *self.state.read().await;
            if let Some(p) = current_path.as_ref() {
                if p == &model_path && current_state == ProviderState::Ready {
                    println!("[LocalLlamaProvider] Model already loaded, skipping reload");
                    return Ok(());
                }
            }
        }

        if let Some(app) = &self.app_handle {
            app.emit("model-load-progress", 5).ok();
        }

        if !model_path.exists() {
            return Err(format!("Model file not found: {:?}", model_path));
        }
        
        // Set state to loading
        *self.state.write().await = ProviderState::Loading;
        
        if let Some(app) = &self.app_handle {
            app.emit("model-load-progress", 10).ok();
        }

        // Store model path
        *self.model_path.write().await = Some(model_path.clone());
        
        // Load in blocking task
        let model_path_clone = model_path.clone();
        let app_handle_clone = self.app_handle.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            Self::load_model_sync(model_path_clone, app_handle_clone)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;
        
        match result {
            Ok(loaded) => {
                *self.model.write().await = Some(loaded);
                *self.state.write().await = ProviderState::Ready;
                
                if let Some(app) = &self.app_handle {
                    app.emit("model-load-progress", 100).ok();
                    app.emit("model-load-complete", true).ok();
                }
                
                println!("[LocalLlamaProvider] Model loaded successfully");
                Ok(())
            }
            Err(e) => {
                *self.state.write().await = ProviderState::Error;
                if let Some(app) = &self.app_handle {
                    app.emit("model-load-error", e.clone()).ok();
                }
                Err(e)
            }
        }
    }
    
    /// Synchronous model loading
    fn load_model_sync(model_path: PathBuf, app_handle: Option<AppHandle>) -> Result<LoadedModel, String> {
        // Get device
        let device = Self::get_device()
            .map_err(|e| format!("Failed to get device: {}", e))?;
        
        println!("[LocalLlamaProvider] Using device: {:?}", device);
        
        if let Some(app) = &app_handle {
            app.emit("model-load-progress", 20).ok();
        }

        // Open and read GGUF file
        let file = File::open(&model_path)
            .map_err(|e| format!("Failed to open model file: {}", e))?;
        let mut reader = BufReader::new(file);
        
        // Read GGUF content
        let content = Content::read(&mut reader)
            .map_err(|e| format!("Failed to read GGUF content: {}", e))?;
        
        println!("[LocalLlamaProvider] GGUF content read successfully");
        
        // Load model weights
        let weights = ModelWeights::from_gguf(content, &mut reader, &device)
            .map_err(|e| format!("Failed to load model weights: {}", e))?;
        
        println!("[LocalLlamaProvider] Model weights loaded");
        
        // Load tokenizer - try to find tokenizer.json next to model
        let tokenizer_path = model_path.parent()
            .map(|p| p.join("tokenizer.json"))
            .filter(|p| p.exists());
        
        let tokenizer = if let Some(tok_path) = tokenizer_path {
            println!("[LocalLlamaProvider] Loading tokenizer from: {:?}", tok_path);
            Tokenizer::from_file(&tok_path)
                .map_err(|e| format!("Failed to load tokenizer: {}", e))?
        } else {
            return Err("No tokenizer.json found next to model file. Please provide a tokenizer.".to_string());
        };
        
        Ok(LoadedModel {
            weights,
            tokenizer,
            device,
        })
    }
    
    /// Unload the current model
    pub async fn unload_model(&self) {
        println!("[LocalLlamaProvider] Unloading model");
        *self.model.write().await = None;
        *self.model_path.write().await = None;
        *self.state.write().await = ProviderState::Unloaded;
    }
    
    /// Format messages for the model
    fn format_messages(system_prompt: &str, messages: &[Message], is_turbo: bool) -> String {
        let base_system = if is_turbo { LOCAL_TURBO_TEMPLATE } else { LOCAL_CHAT_TEMPLATE };
        let mut formatted = format!("<|im_start|>system\n{}\n{}\n<|im_end|>\n", base_system.trim(), system_prompt);
        
        for msg in messages {
            let role = match msg.role.as_str() {
                "user" => "user",
                "assistant" => "assistant",
                "system" => continue,
                _ => "user",
            };
            
            formatted.push_str(&format!(
                "<|im_start|>{}\n{}\n<|im_end|>\n",
                role, msg.content
            ));
        }
        
        formatted.push_str("<|im_start|>assistant\n");
        formatted
    }
    
    /// Generate text (blocking)
    fn generate_sync(
        model: &mut LoadedModel,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String, String> {
        use candle_core::Tensor;
        use rand::SeedableRng;
        
        // Tokenize input
        let encoding = model.tokenizer.encode(prompt, true)
            .map_err(|e| format!("Tokenization failed: {}", e))?;
        let input_ids = encoding.get_ids();
        
        // Create input tensor
        let mut tokens: Vec<u32> = input_ids.to_vec();
        let mut generated_text = String::new();
        
        // Get special tokens
        let eos_token_id = model.tokenizer.token_to_id("<|im_end|>")
            .or_else(|| model.tokenizer.token_to_id("</s>"))
            .unwrap_or(2);
        
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        
        for i in 0..max_tokens {
            // Create tensor from tokens
            let context_size = tokens.len().min(2048);
            let start = tokens.len().saturating_sub(context_size);
            let context = &tokens[start..];
            
            let input = Tensor::new(context, &model.device)
                .map_err(|e| format!("Failed to create tensor: {}", e))?
                .unsqueeze(0)
                .map_err(|e| format!("Failed to unsqueeze: {}", e))?;
            
            // Forward pass
            let logits = model.weights.forward(&input, i)
                .map_err(|e| format!("Forward pass failed: {}", e))?;
            
            // Get last token logits
            let logits = logits.squeeze(0)
                .map_err(|e| format!("Squeeze failed: {}", e))?;
            let seq_len = logits.dim(0).map_err(|e| format!("Dim error: {}", e))?;
            let logits = logits.get(seq_len - 1)
                .map_err(|e| format!("Get last failed: {}", e))?;
            
            // Apply temperature and sample
            let next_token = if temperature <= 0.0 {
                // Greedy
                logits.argmax(0)
                    .map_err(|e| format!("Argmax failed: {}", e))?
                    .to_scalar::<u32>()
                    .map_err(|e| format!("To scalar failed: {}", e))?
            } else {
                // Sample with temperature
                let logits = (&logits / temperature as f64)
                    .map_err(|e| format!("Div failed: {}", e))?;
                let probs = candle_nn::ops::softmax(&logits, 0)
                    .map_err(|e| format!("Softmax failed: {}", e))?;
                let probs_vec: Vec<f32> = probs.to_vec1()
                    .map_err(|e| format!("To vec failed: {}", e))?;
                
                // Weighted sampling
                use rand::distributions::{Distribution, WeightedIndex};
                let dist = WeightedIndex::new(&probs_vec)
                    .map_err(|e| format!("WeightedIndex failed: {}", e))?;
                dist.sample(&mut rng) as u32
            };
            
            // Check for EOS
            if next_token == eos_token_id {
                break;
            }
            
            tokens.push(next_token);
            
            // Decode new token
            if let Some(text) = model.tokenizer.decode(&[next_token], false).ok() {
                generated_text.push_str(&text);
            }
        }
        
        Ok(generated_text)
    }
    
    /// Internal generation method
    /// Internal generation method
    async fn generate(&self, system_prompt: &str, messages: &[Message], is_turbo: bool) -> Result<String, String> {
        // Log current state
        let current_state = *self.state.read().await;
        println!("[LocalLlamaProvider] generate called. State: {:?}, Model loaded? {}", 
            current_state, 
            self.model.read().await.is_some()
        );

        if !self.is_loaded().await {
            // Check if model is actually loaded in memory despite state
            if self.model.read().await.is_some() {
                 println!("[LocalLlamaProvider] State is {:?} but model is in memory. Setting state to Ready.", current_state);
                 *self.state.write().await = ProviderState::Ready;
            } else {
                // Auto-load if default model exists
                let default_path = get_default_model_path();
                if default_path.exists() {
                    println!("[LocalLlamaProvider] Auto-loading default model...");
                    self.load_model(default_path).await?;
                } else {
                    return Err("Model not loaded. Please download the model first.".to_string());
                }
            }
        }
        
        let prompt = Self::format_messages(system_prompt, messages, is_turbo);
        let model_arc = self.model.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut model_guard = futures::executor::block_on(model_arc.write());
            if let Some(ref mut loaded) = *model_guard {
                Self::generate_sync(loaded, &prompt, 2048, 0.7)
            } else {
                Err("Model not loaded".to_string())
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }
}

impl Default for LocalLlamaProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl ModelProvider for LocalLlamaProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_tools: true,
            supports_streaming: false,
            supports_cascade: false,
            supports_summarization: false,
            max_context_tokens: 4096,
        }
    }
    
    fn name(&self) -> &str {
        "LocalLlama"
    }
    
    async fn chat(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        let text = self.generate(system_prompt, messages, false).await?;
        
        // Return as single-chunk stream - use Box::pin for Unpin
        let stream = stream::once(async move { Ok(text) });
        Ok(CompletionResult::Stream(Box::pin(stream)))
    }
    
    async fn agent(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        let text = self.generate(system_prompt, messages, true).await?;
        
        // Return as single-chunk stream - use Box::pin for Unpin
        let stream = stream::once(async move { Ok(text) });
        Ok(CompletionResult::Stream(Box::pin(stream)))
    }
    
    fn active_model(&self) -> Option<ModelTier> {
        Some(ModelTier::Local)
    }
}

/// Get default model path for offline mode
pub fn get_default_model_path() -> PathBuf {
    get_model_path()
}

/// Check if a local model is available
pub fn is_model_available() -> bool {
    get_default_model_path().exists()
}
