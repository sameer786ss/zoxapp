//! Hardware Detection Module
//!
//! Provides GPU and system hardware detection for optimal binary selection.

pub mod gpu;

pub use gpu::{GpuInfo, detect_gpu};
