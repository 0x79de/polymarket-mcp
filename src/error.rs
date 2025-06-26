use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
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
#[allow(dead_code)]
pub enum PolymarketError {
    #[error("API request failed: {message} (request_id: {request_id})")]
    ApiError {
        message: String,
        status_code: Option<u16>,
        request_id: RequestId,
    },
    
    #[error("Rate limit exceeded (request_id: {request_id})")]
    RateLimitError {
        retry_after: Option<u64>,
        request_id: RequestId,
    },
    
    #[error("Network error: {message} (request_id: {request_id})")]
    NetworkError {
        message: String,
        request_id: RequestId,
    },
    
    #[error("Deserialization error: {message} (request_id: {request_id})")]
    DeserializationError {
        message: String,
        request_id: RequestId,
    },
    
    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
    },
    
    #[error("Cache error: {message} (request_id: {request_id})")]
    CacheError {
        message: String,
        request_id: RequestId,
    },
    
    #[error("Market not found: {market_id} (request_id: {request_id})")]
    MarketNotFound {
        market_id: String,
        request_id: RequestId,
    },
    
    #[error("Invalid parameters: {message} (request_id: {request_id})")]
    InvalidParameters {
        message: String,
        request_id: RequestId,
    },
}

#[allow(dead_code)]
impl PolymarketError {
    pub fn api_error(message: impl Into<String>, status_code: Option<u16>) -> Self {
        Self::ApiError {
            message: message.into(),
            status_code,
            request_id: RequestId::new(),
        }
    }
    
    pub fn rate_limit_error(retry_after: Option<u64>) -> Self {
        Self::RateLimitError {
            retry_after,
            request_id: RequestId::new(),
        }
    }
    
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
            request_id: RequestId::new(),
        }
    }
    
    pub fn deserialization_error(message: impl Into<String>) -> Self {
        Self::DeserializationError {
            message: message.into(),
            request_id: RequestId::new(),
        }
    }
    
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }
    
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::CacheError {
            message: message.into(),
            request_id: RequestId::new(),
        }
    }
    
    pub fn market_not_found(market_id: impl Into<String>) -> Self {
        Self::MarketNotFound {
            market_id: market_id.into(),
            request_id: RequestId::new(),
        }
    }
    
    pub fn invalid_parameters(message: impl Into<String>) -> Self {
        Self::InvalidParameters {
            message: message.into(),
            request_id: RequestId::new(),
        }
    }
    
    pub fn request_id(&self) -> Option<&RequestId> {
        match self {
            Self::ApiError { request_id, .. }
            | Self::RateLimitError { request_id, .. }
            | Self::NetworkError { request_id, .. }
            | Self::DeserializationError { request_id, .. }
            | Self::CacheError { request_id, .. }
            | Self::MarketNotFound { request_id, .. }
            | Self::InvalidParameters { request_id, .. } => Some(request_id),
            Self::ConfigError { .. } => None,
        }
    }
    
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError { .. } | Self::RateLimitError { .. }
        )
    }
    
    pub fn log_error(&self) {
        match self {
            Self::ApiError { message, status_code, request_id } => {
                error!(
                    request_id = %request_id,
                    status_code = ?status_code,
                    "API Error: {}", message
                );
            }
            Self::RateLimitError { retry_after, request_id } => {
                error!(
                    request_id = %request_id,
                    retry_after = ?retry_after,
                    "Rate limit exceeded"
                );
            }
            Self::NetworkError { message, request_id } => {
                error!(
                    request_id = %request_id,
                    "Network Error: {}", message
                );
            }
            Self::DeserializationError { message, request_id } => {
                error!(
                    request_id = %request_id,
                    "Deserialization Error: {}", message
                );
            }
            Self::ConfigError { message } => {
                error!("Configuration Error: {}", message);
            }
            Self::CacheError { message, request_id } => {
                error!(
                    request_id = %request_id,
                    "Cache Error: {}", message
                );
            }
            Self::MarketNotFound { market_id, request_id } => {
                error!(
                    request_id = %request_id,
                    market_id = %market_id,
                    "Market not found"
                );
            }
            Self::InvalidParameters { message, request_id } => {
                error!(
                    request_id = %request_id,
                    "Invalid Parameters: {}", message
                );
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, PolymarketError>;

#[allow(dead_code)]
pub trait ErrorContext<T> {
    fn with_request_context(self, request_id: &RequestId) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_request_context(self, _request_id: &RequestId) -> Result<T> {
        self.map_err(|e| {
            PolymarketError::network_error(format!("Request failed: {}", e))
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub api_requests_total: u64,
    pub api_requests_failed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub active_connections: u64,
    pub avg_response_time_ms: f64,
}

#[allow(dead_code)]
impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment_api_requests(&mut self) {
        self.api_requests_total += 1;
    }
    
    pub fn increment_api_failures(&mut self) {
        self.api_requests_failed += 1;
    }
    
    pub fn increment_cache_hits(&mut self) {
        self.cache_hits += 1;
    }
    
    pub fn increment_cache_misses(&mut self) {
        self.cache_misses += 1;
    }
    
    pub fn set_active_connections(&mut self, count: u64) {
        self.active_connections = count;
    }
    
    pub fn update_avg_response_time(&mut self, response_time_ms: f64) {
        if self.api_requests_total > 0 {
            self.avg_response_time_ms = 
                (self.avg_response_time_ms * (self.api_requests_total as f64 - 1.0) + response_time_ms) 
                / self.api_requests_total as f64;
        } else {
            self.avg_response_time_ms = response_time_ms;
        }
    }
    
    pub fn cache_hit_ratio(&self) -> f64 {
        let total_cache_requests = self.cache_hits + self.cache_misses;
        if total_cache_requests > 0 {
            self.cache_hits as f64 / total_cache_requests as f64
        } else {
            0.0
        }
    }
    
    pub fn error_rate(&self) -> f64 {
        if self.api_requests_total > 0 {
            self.api_requests_failed as f64 / self.api_requests_total as f64
        } else {
            0.0
        }
    }
}