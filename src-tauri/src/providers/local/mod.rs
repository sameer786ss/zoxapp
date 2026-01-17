//! Local Provider Module
//!
//! Provides local model inference using GGUF models.
//! Simple implementation: same model for chat and turbo, no cascade.

mod client;

pub use client::{LocalLlamaProvider, is_model_available, get_default_model_path};
