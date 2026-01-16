//! Local Model Provider using Candle ML Framework
//!
//! Provides local inference using GGUF models via Candle.
//! Supports CUDA (NVIDIA), Metal (Apple), and CPU fallback.
//!
//! Binaries are loaded from %APPDATA%/zox/binaries/ to avoid system dependencies.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use futures::stream::{self, Stream};
use std::pin::Pin;

use crate::providers::{ModelProvider, CompletionResult, ProviderCapabilities, ModelTier, Message};
use crate::setup::paths::get_model_path;

use candle_core::{Device, Tensor, DType};
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

RULES:
1. ONE tool per response
2. Wait for OBSERVATION before continuing
3. Keep thinking brief (1 sentence)
4. ALWAYS wrap responses in XML tags
5. NO markdown code blocks
6. NEVER use JSON - XML ONLY
7. For messages without tools, ALWAYS use <message> tags
<|im_end|>
"#;


/// Local provider configuration
#[derive(Clone)]
pub struct LocalConfig {
    pub model_path: PathBuf,
    pub ctx_size: u32,
    pub max_tokens: usize,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            model_path: get_model_path(),
            ctx_size: 2048,
            max_tokens: 2048,
        }
    }
}

/// Model loading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
    Error,
}

/// Device type for GPU acceleration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Cpu,
    Cuda,
    Metal,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Cpu => write!(f, "CPU"),
            DeviceType::Cuda => write!(f, "CUDA"),
            DeviceType::Metal => write!(f, "Metal"),
        }
    }
}

/// Local provider using Candle for GGUF models
pub struct CandleProvider {
    config: LocalConfig,
    state: Arc<RwLock<ModelState>>,
    load_progress: Arc<AtomicU8>,
    is_generating: Arc<AtomicBool>,
    device_type: DeviceType,
    device: Arc<RwLock<Option<Device>>>,
    model: Arc<RwLock<Option<ModelWeights>>>,
    tokenizer: Arc<RwLock<Option<Tokenizer>>>,
}

impl CandleProvider {
    pub fn new(config: LocalConfig) -> Self {
        let device_type = Self::detect_device_type();
        println!("[CandleProvider] Detected device: {}", device_type);
        
        Self {
            config,
            state: Arc::new(RwLock::new(ModelState::Unloaded)),
            load_progress: Arc::new(AtomicU8::new(0)),
            is_generating: Arc::new(AtomicBool::new(false)),
            device_type,
            device: Arc::new(RwLock::new(None)),
            model: Arc::new(RwLock::new(None)),
            tokenizer: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default settings
    pub fn with_defaults() -> Self {
        Self::new(LocalConfig::default())
    }

    /// Detect the best available device
    fn detect_device_type() -> DeviceType {
        // Check for CUDA (NVIDIA GPU)
        #[cfg(feature = "cuda")]
        {
            // Set CUDA paths from our binaries directory before checking
            Self::setup_cuda_paths();
            
            if candle_core::utils::cuda_is_available() {
                return DeviceType::Cuda;
            }
        }

        // Check for Metal (Apple Silicon)
        #[cfg(feature = "metal")]
        {
            if candle_core::utils::metal_is_available() {
                return DeviceType::Metal;
            }
        }

        DeviceType::Cpu
    }

    /// Setup CUDA library paths from bundled binaries
    #[cfg(feature = "cuda")]
    fn setup_cuda_paths() {
        use crate::setup::paths::get_binaries_dir;
        
        let binaries_dir = get_binaries_dir();
        if binaries_dir.exists() {
            // Set CUDA_PATH to our binaries directory
            std::env::set_var("CUDA_PATH", &binaries_dir);
            
            // Also add to LD_LIBRARY_PATH on Linux
            #[cfg(target_os = "linux")]
            {
                let current = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
                let new_path = format!("{}:{}", binaries_dir.display(), current);
                std::env::set_var("LD_LIBRARY_PATH", new_path);
            }
            
            println!("[CandleProvider] Set CUDA_PATH to: {}", binaries_dir.display());
        }
    }

    /// Check if model file exists
    pub fn model_exists(&self) -> bool {
        self.config.model_path.exists()
    }

    /// Get current load progress (0-100)
    pub fn get_load_progress(&self) -> u8 {
        self.load_progress.load(Ordering::SeqCst)
    }

    /// Get current model state
    pub async fn get_state(&self) -> ModelState {
        *self.state.read().await
    }

    /// Create the Candle device
    fn create_device(&self) -> Result<Device, String> {
        match self.device_type {
            #[cfg(feature = "cuda")]
            DeviceType::Cuda => {
                Device::new_cuda(0).map_err(|e| format!("CUDA device error: {}", e))
            }
            #[cfg(feature = "metal")]
            DeviceType::Metal => {
                Device::new_metal(0).map_err(|e| format!("Metal device error: {}", e))
            }
            _ => Ok(Device::Cpu),
        }
    }

    /// Load the GGUF model
    pub async fn load_model(&self) -> Result<(), String> {
        let current_state = *self.state.read().await;
        if current_state == ModelState::Loaded {
            return Ok(());
        }
        if current_state == ModelState::Loading {
            return Err("Model is already loading".to_string());
        }

        {
            let mut state = self.state.write().await;
            *state = ModelState::Loading;
        }

        self.load_progress.store(0, Ordering::SeqCst);

        // Check model file exists
        if !self.model_exists() {
            let mut state = self.state.write().await;
            *state = ModelState::Error;
            return Err(format!("Model file not found: {:?}", self.config.model_path));
        }

        self.load_progress.store(10, Ordering::SeqCst);

        let model_path = self.config.model_path.clone();
        let device_type = self.device_type;
        let progress = self.load_progress.clone();

        // Load model in blocking task
        let result = tokio::task::spawn_blocking(move || {
            println!("[CandleProvider] Loading model from {:?}", model_path);

            // Create device
            progress.store(20, Ordering::SeqCst);
            
            let device = match device_type {
                #[cfg(feature = "cuda")]
                DeviceType::Cuda => Device::new_cuda(0).unwrap_or(Device::Cpu),
                #[cfg(feature = "metal")]
                DeviceType::Metal => Device::new_metal(0).unwrap_or(Device::Cpu),
                _ => Device::Cpu,
            };

            println!("[CandleProvider] Using device: {:?}", device);
            progress.store(30, Ordering::SeqCst);

            // Load GGUF model
            let model_content = std::fs::read(&model_path)
                .map_err(|e| format!("Failed to read model file: {}", e))?;

            progress.store(60, Ordering::SeqCst);

            // Parse GGUF and create model weights
            let mut file = std::io::Cursor::new(&model_content);
            let model = ModelWeights::from_gguf(file, &mut file, &device)
                .map_err(|e| format!("Failed to load GGUF model: {}", e))?;

            progress.store(80, Ordering::SeqCst);

            // Load tokenizer from GGUF or use default
            let tokenizer = Self::load_tokenizer_from_gguf(&model_content)
                .unwrap_or_else(|_| Self::create_default_tokenizer());

            progress.store(100, Ordering::SeqCst);

            println!("[CandleProvider] Model loaded successfully!");
            Ok::<(Device, ModelWeights, Tokenizer), String>((device, model, tokenizer))
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        match result {
            Ok((device, model, tokenizer)) => {
                let mut dev_lock = self.device.write().await;
                *dev_lock = Some(device);
                
                let mut model_lock = self.model.write().await;
                *model_lock = Some(model);
                
                let mut tok_lock = self.tokenizer.write().await;
                *tok_lock = Some(tokenizer);
                
                let mut state = self.state.write().await;
                *state = ModelState::Loaded;
                
                Ok(())
            }
            Err(e) => {
                let mut state = self.state.write().await;
                *state = ModelState::Error;
                Err(e)
            }
        }
    }

    /// Load tokenizer from GGUF metadata
    fn load_tokenizer_from_gguf(_data: &[u8]) -> Result<Tokenizer, String> {
        // GGUF models often embed tokenizer - for now use HuggingFace tokenizer
        // In production, parse GGUF metadata for tokenizer vocab
        Err("GGUF tokenizer extraction not implemented".to_string())
    }

    /// Create a default tokenizer (fallback)
    fn create_default_tokenizer() -> Tokenizer {
        // Use a simple BPE tokenizer as fallback
        // In production, this should be the actual model's tokenizer
        Tokenizer::from_pretrained("hf-internal-testing/llama-tokenizer", None)
            .unwrap_or_else(|_| {
                // Ultimate fallback - create minimal tokenizer
                let mut tokenizer = tokenizers::Tokenizer::new(
                    tokenizers::models::bpe::BPE::default()
                );
                tokenizer
            })
    }

    /// Unload the model from memory
    pub async fn unload_model(&self) -> Result<(), String> {
        let current_state = *self.state.read().await;
        if current_state == ModelState::Unloaded {
            return Ok(());
        }

        {
            let mut state = self.state.write().await;
            *state = ModelState::Unloading;
        }

        // Clear model and device
        {
            let mut model_lock = self.model.write().await;
            *model_lock = None;
        }
        {
            let mut dev_lock = self.device.write().await;
            *dev_lock = None;
        }
        {
            let mut tok_lock = self.tokenizer.write().await;
            *tok_lock = None;
        }

        self.load_progress.store(0, Ordering::SeqCst);

        {
            let mut state = self.state.write().await;
            *state = ModelState::Unloaded;
        }

        println!("[CandleProvider] Model unloaded");
        Ok(())
    }

    /// Format messages for the model
    fn format_messages(&self, system_prompt: &str, messages: &[Message]) -> String {
        let mut prompt = system_prompt.to_string();
        
        for msg in messages {
            let role = match msg.role.as_str() {
                "user" => "user",
                "model" | "assistant" => "assistant",
                _ => "user",
            };
            prompt.push_str(&format!("<|im_start|>{}\n{}\n<|im_end|>\n", role, msg.content));
        }
        
        prompt.push_str("<|im_start|>assistant\n");
        prompt
    }

    /// Generate completion using Candle
    async fn generate_completion(&self, prompt: &str) -> Result<Vec<String>, String> {
        let state = *self.state.read().await;
        if state != ModelState::Loaded {
            return Err("Model not loaded".to_string());
        }

        self.is_generating.store(true, Ordering::SeqCst);

        let prompt_owned = prompt.to_string();
        let max_tokens = self.config.max_tokens;
        let is_generating = self.is_generating.clone();
        let model_arc = self.model.clone();
        let device_arc = self.device.clone();
        let tokenizer_arc = self.tokenizer.clone();

        let result = tokio::task::spawn_blocking(move || {
            let model_guard = model_arc.blocking_read();
            let model = model_guard.as_ref().ok_or("Model not available")?;
            
            let device_guard = device_arc.blocking_read();
            let device = device_guard.as_ref().ok_or("Device not available")?;
            
            let tok_guard = tokenizer_arc.blocking_read();
            let tokenizer = tok_guard.as_ref().ok_or("Tokenizer not available")?;

            // Tokenize input
            let encoding = tokenizer.encode(prompt_owned.as_str(), true)
                .map_err(|e| format!("Tokenization failed: {}", e))?;
            
            let tokens: Vec<u32> = encoding.get_ids().to_vec();
            println!("[CandleProvider] Input tokens: {}", tokens.len());

            // Convert to tensor
            let input_tensor = Tensor::new(tokens.as_slice(), device)
                .map_err(|e| format!("Tensor creation failed: {}", e))?;

            let mut generated_tokens = Vec::new();
            let mut generated_text = Vec::new();
            let mut current_tokens = input_tensor.clone();

            // Generation loop
            for _ in 0..max_tokens {
                if !is_generating.load(Ordering::SeqCst) {
                    break;
                }

                // Forward pass
                let logits = model.forward(&current_tokens, 0)
                    .map_err(|e| format!("Forward pass failed: {}", e))?;

                // Sample next token (greedy for now)
                let next_token = logits
                    .argmax(candle_core::D::Minus1)
                    .map_err(|e| format!("Argmax failed: {}", e))?
                    .to_scalar::<u32>()
                    .map_err(|e| format!("Scalar conversion failed: {}", e))?;

                // Check for EOS
                if next_token == 2 || next_token == 0 {
                    break;
                }

                generated_tokens.push(next_token);

                // Decode token
                let text = tokenizer.decode(&[next_token], false)
                    .unwrap_or_default();
                
                generated_text.push(text.clone());

                // Check for stop sequences
                let current_text: String = generated_text.iter().cloned().collect();
                if current_text.contains("<|im_end|>") || current_text.contains("</message>") {
                    break;
                }

                // Update current tokens for next iteration
                current_tokens = Tensor::new(&[next_token], device)
                    .map_err(|e| format!("Token tensor failed: {}", e))?;
            }

            println!("[CandleProvider] Generated {} tokens", generated_tokens.len());
            Ok::<Vec<String>, String>(generated_text)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        self.is_generating.store(false, Ordering::SeqCst);
        result
    }
}

#[async_trait]
impl ModelProvider for CandleProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_cascade: false,
            supports_summarization: false,
            max_context_tokens: self.config.ctx_size as usize,
        }
    }

    fn name(&self) -> &str {
        match self.device_type {
            DeviceType::Cuda => "Local (Candle/CUDA)",
            DeviceType::Metal => "Local (Candle/Metal)",
            DeviceType::Cpu => "Local (Candle/CPU)",
        }
    }

    async fn chat(
        &self,
        _system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        // Ensure model is loaded
        let state = *self.state.read().await;
        if state != ModelState::Loaded {
            self.load_model().await?;
        }

        // Format with chat template
        let prompt = self.format_messages(LOCAL_CHAT_TEMPLATE, messages);
        
        // Generate response
        let chunks = self.generate_completion(&prompt).await?;
        
        // Create stream from chunks
        let stream_chunks: Vec<Result<String, String>> = chunks.into_iter().map(Ok).collect();
        let stream = stream::iter(stream_chunks);
        
        Ok(CompletionResult::Stream(Box::new(Box::pin(stream))))
    }

    async fn agent(
        &self,
        _system_prompt: &str,
        messages: &[Message],
    ) -> Result<CompletionResult, String> {
        // Ensure model is loaded
        let state = *self.state.read().await;
        if state != ModelState::Loaded {
            self.load_model().await?;
        }

        // Format with turbo template
        let prompt = self.format_messages(LOCAL_TURBO_TEMPLATE, messages);
        
        // Generate response
        let chunks = self.generate_completion(&prompt).await?;
        
        // Create stream from chunks
        let stream_chunks: Vec<Result<String, String>> = chunks.into_iter().map(Ok).collect();
        let stream = stream::iter(stream_chunks);
        
        Ok(CompletionResult::Stream(Box::new(Box::pin(stream))))
    }

    fn active_model(&self) -> Option<ModelTier> {
        Some(ModelTier::Local)
    }
}

// Re-export for backwards compatibility
pub type LocalLlamaProvider = CandleProvider;
pub type LocalProvider = CandleProvider;
