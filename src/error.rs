use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum DeputyError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("IO operation failed: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Model API error: {0}")]
    Model(#[from] ModelError),
    
    #[error("Tool execution error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Session management error: {0}")]
    Session(#[from] SessionError),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, DeputyError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorResponse {
    pub error: ApiError,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {reason}")]
    Missing { reason: String },
    
    #[error("Invalid configuration: {reason}")]
    Invalid { reason: String },
    
    #[error("Failed to read configuration: {reason}")]
    ReadFailed { reason: String },
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {reason}")]
    NotFound { reason: String },
    
    #[error("Invalid tool arguments: {reason}")]
    InvalidArguments { reason: String },
    
    #[error("Tool execution failed: {reason}")]
    ExecutionFailed { reason: String },
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session processing failed: {reason}")]
    Processing { reason: String },
    
    #[error("User input error: {reason}")]
    UserInput { reason: String },
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Authentication failed: {reason}")]
    Authentication { reason: String },
    
    #[error("Rate limit exceeded: {reason}")]
    RateLimit { reason: String, retry_after_seconds: Option<u64> },
    
    #[error("API request failed: {reason}")]
    Request { reason: String },
    
    #[error("Network error: {reason}")]
    Network { reason: String },
}

