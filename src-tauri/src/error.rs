//! ZOX Error Types
//! 
//! Centralized error handling using thiserror for type-safe errors.

use std::fmt;

/// Main error type for ZOX operations
#[derive(Debug)]
pub enum ZoxError {
    /// API/Network errors
    Api(ApiError),
    /// Tool execution errors
    Tool(ToolError),
    /// Model/Inference errors
    Model(ModelError),
    /// File system errors
    FileSystem(String),
    /// Configuration errors
    Config(String),
    /// State management errors
    State(String),
}

impl fmt::Display for ZoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoxError::Api(e) => write!(f, "API Error: {}", e),
            ZoxError::Tool(e) => write!(f, "Tool Error: {}", e),
            ZoxError::Model(e) => write!(f, "Model Error: {}", e),
            ZoxError::FileSystem(msg) => write!(f, "File System Error: {}", msg),
            ZoxError::Config(msg) => write!(f, "Config Error: {}", msg),
            ZoxError::State(msg) => write!(f, "State Error: {}", msg),
        }
    }
}

impl std::error::Error for ZoxError {}

/// API-related errors
#[derive(Debug)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiErrorCode {
    RateLimit,
    InvalidKey,
    Network,
    Timeout,
    ServerError,
    Unknown,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {} (retryable: {})", self.code, self.message, self.retryable)
    }
}

impl ApiError {
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::RateLimit,
            message: message.into(),
            retryable: true,
        }
    }

    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::InvalidKey,
            message: message.into(),
            retryable: false,
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::Network,
            message: message.into(),
            retryable: true,
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::Timeout,
            message: message.into(),
            retryable: true,
        }
    }

    pub fn server(message: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::ServerError,
            message: message.into(),
            retryable: true,
        }
    }
}

/// Tool execution errors
#[derive(Debug)]
pub struct ToolError {
    pub tool_name: String,
    pub message: String,
    pub recoverable: bool,
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} (recoverable: {})", self.tool_name, self.message, self.recoverable)
    }
}

impl ToolError {
    pub fn new(tool: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_name: tool.into(),
            message: message.into(),
            recoverable: true,
        }
    }

    pub fn fatal(tool: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_name: tool.into(),
            message: message.into(),
            recoverable: false,
        }
    }
}

/// Model/Inference errors
#[derive(Debug)]
pub struct ModelError {
    pub kind: ModelErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelErrorKind {
    NotLoaded,
    LoadFailed,
    InferenceFailed,
    OutOfMemory,
    InvalidInput,
    Cancelled,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)
    }
}

impl ModelError {
    pub fn not_loaded() -> Self {
        Self {
            kind: ModelErrorKind::NotLoaded,
            message: "Model is not loaded".into(),
        }
    }

    pub fn load_failed(message: impl Into<String>) -> Self {
        Self {
            kind: ModelErrorKind::LoadFailed,
            message: message.into(),
        }
    }

    pub fn inference_failed(message: impl Into<String>) -> Self {
        Self {
            kind: ModelErrorKind::InferenceFailed,
            message: message.into(),
        }
    }

    pub fn out_of_memory() -> Self {
        Self {
            kind: ModelErrorKind::OutOfMemory,
            message: "Out of GPU/CPU memory".into(),
        }
    }
}

// Conversion helpers for backwards compatibility with String errors
impl From<ZoxError> for String {
    fn from(err: ZoxError) -> String {
        err.to_string()
    }
}

impl From<String> for ZoxError {
    fn from(s: String) -> Self {
        ZoxError::State(s)
    }
}

impl From<&str> for ZoxError {
    fn from(s: &str) -> Self {
        ZoxError::State(s.to_string())
    }
}

impl From<std::io::Error> for ZoxError {
    fn from(err: std::io::Error) -> Self {
        ZoxError::FileSystem(err.to_string())
    }
}

/// Result type alias for ZOX operations
pub type ZoxResult<T> = Result<T, ZoxError>;
