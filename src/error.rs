use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PolymarketError {
    #[error("API request failed: {message} (request_id: {request_id})")]
    ApiError {
        message: String,
        status_code: Option<u16>,
        request_id: RequestId,
    },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

impl PolymarketError {
    pub fn api_error(message: impl Into<String>, status_code: Option<u16>) -> Self {
        Self::ApiError {
            message: message.into(),
            status_code,
            request_id: RequestId::new(),
        }
    }

    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    pub fn deserialization_error(message: impl Into<String>) -> Self {
        Self::DeserializationError {
            message: message.into(),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, PolymarketError>;
