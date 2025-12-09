use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Invalid model string: {0}")]
    InvalidModelString(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Feature not supported: {0}")]
    UnsupportedFeature(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Circuit breaker open for provider: {0}")]
    CircuitBreakerOpen(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl LlmError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::NetworkError(_)
                | LlmError::TimeoutError(_)
                | LlmError::RateLimitError(_)
                | LlmError::ApiError { status, .. } if *status >= 500
        )
    }

    pub fn is_rate_limit(&self) -> bool {
        matches!(self, LlmError::RateLimitError(_))
    }

    pub fn is_authentication(&self) -> bool {
        matches!(self, LlmError::AuthenticationError(_))
    }
}

pub type Result<T> = std::result::Result<T, LlmError>;
