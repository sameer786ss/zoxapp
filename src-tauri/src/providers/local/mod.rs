//! Local Provider Module
//!
//! Provides local model inference using GGUF models.
//! Simple implementation: same model for chat and turbo, no cascade.

mod client;

pub use client::LocalLlamaProvider;
