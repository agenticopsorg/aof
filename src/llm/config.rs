use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default provider if not specified
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// Timeout for requests
    #[serde(default = "default_timeout")]
    pub timeout: Duration,

    /// Enable retry on failure
    #[serde(default = "default_retry")]
    pub retry_enabled: bool,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Initial retry delay
    #[serde(default = "default_retry_delay")]
    pub retry_delay: Duration,

    /// Enable circuit breaker
    #[serde(default = "default_circuit_breaker")]
    pub circuit_breaker_enabled: bool,

    /// Circuit breaker failure threshold
    #[serde(default = "default_failure_threshold")]
    pub circuit_breaker_threshold: usize,

    /// Provider-specific configurations
    #[serde(default)]
    pub providers: ProviderConfigs,
}

/// Provider-specific configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderConfigs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openai: Option<OpenAiConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub anthropic: Option<AnthropicConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google: Option<GoogleConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama: Option<OllamaConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grok: Option<GrokConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure: Option<AzureOpenAiConfig>,
}

/// OpenAI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    /// API key (falls back to OPENAI_API_KEY env var)
    pub api_key: Option<String>,

    /// Base URL (for proxies)
    #[serde(default = "openai_base_url")]
    pub base_url: String,

    /// Organization ID
    pub organization_id: Option<String>,

    /// Default model
    #[serde(default = "openai_default_model")]
    pub default_model: String,

    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Default max tokens
    pub max_tokens: Option<usize>,
}

/// Anthropic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// API key (falls back to ANTHROPIC_API_KEY env var)
    pub api_key: Option<String>,

    /// Base URL
    #[serde(default = "anthropic_base_url")]
    pub base_url: String,

    /// Default model
    #[serde(default = "anthropic_default_model")]
    pub default_model: String,

    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Default max tokens
    #[serde(default = "anthropic_default_max_tokens")]
    pub max_tokens: usize,
}

/// Google configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    /// API key (falls back to GOOGLE_API_KEY env var)
    pub api_key: Option<String>,

    /// Base URL
    #[serde(default = "google_base_url")]
    pub base_url: String,

    /// Default model
    #[serde(default = "google_default_model")]
    pub default_model: String,

    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

/// Ollama configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL
    #[serde(default = "ollama_base_url")]
    pub base_url: String,

    /// Default model
    #[serde(default = "ollama_default_model")]
    pub default_model: String,

    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

/// Docker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Docker host
    #[serde(default = "docker_host")]
    pub host: String,

    /// Container image
    pub image: Option<String>,

    /// Default model
    pub default_model: String,
}

/// Grok configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokConfig {
    /// API key (falls back to XAI_API_KEY env var)
    pub api_key: Option<String>,

    /// Base URL
    #[serde(default = "grok_base_url")]
    pub base_url: String,

    /// Default model
    #[serde(default = "grok_default_model")]
    pub default_model: String,
}

/// Azure OpenAI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAiConfig {
    /// API key (falls back to AZURE_OPENAI_API_KEY env var)
    pub api_key: Option<String>,

    /// Endpoint
    pub endpoint: String,

    /// Deployment name
    pub deployment_name: String,

    /// API version
    #[serde(default = "azure_api_version")]
    pub api_version: String,
}

// Default functions
fn default_provider() -> String {
    "openai".to_string()
}

fn default_timeout() -> Duration {
    Duration::from_secs(60)
}

fn default_retry() -> bool {
    true
}

fn default_max_retries() -> usize {
    3
}

fn default_retry_delay() -> Duration {
    Duration::from_millis(1000)
}

fn default_circuit_breaker() -> bool {
    true
}

fn default_failure_threshold() -> usize {
    5
}

fn default_temperature() -> f32 {
    0.7
}

fn openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn openai_default_model() -> String {
    "gpt-4o".to_string()
}

fn anthropic_base_url() -> String {
    "https://api.anthropic.com/v1".to_string()
}

fn anthropic_default_model() -> String {
    "claude-3-5-sonnet-20241022".to_string()
}

fn anthropic_default_max_tokens() -> usize {
    4096
}

fn google_base_url() -> String {
    "https://generativelanguage.googleapis.com/v1".to_string()
}

fn google_default_model() -> String {
    "gemini-2.0-flash-exp".to_string()
}

fn ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn ollama_default_model() -> String {
    "llama3".to_string()
}

fn docker_host() -> String {
    "http://localhost:8080".to_string()
}

fn grok_base_url() -> String {
    "https://api.x.ai/v1".to_string()
}

fn grok_default_model() -> String {
    "grok-2".to_string()
}

fn azure_api_version() -> String {
    "2024-02-15-preview".to_string()
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            timeout: default_timeout(),
            retry_enabled: default_retry(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            circuit_breaker_enabled: default_circuit_breaker(),
            circuit_breaker_threshold: default_failure_threshold(),
            providers: ProviderConfigs::default(),
        }
    }
}

impl OpenAiConfig {
    pub fn from_env() -> Option<Self> {
        std::env::var("OPENAI_API_KEY").ok().map(|api_key| Self {
            api_key: Some(api_key),
            base_url: openai_base_url(),
            organization_id: std::env::var("OPENAI_ORG_ID").ok(),
            default_model: openai_default_model(),
            temperature: default_temperature(),
            max_tokens: None,
        })
    }
}

impl AnthropicConfig {
    pub fn from_env() -> Option<Self> {
        std::env::var("ANTHROPIC_API_KEY").ok().map(|api_key| Self {
            api_key: Some(api_key),
            base_url: anthropic_base_url(),
            default_model: anthropic_default_model(),
            temperature: default_temperature(),
            max_tokens: anthropic_default_max_tokens(),
        })
    }
}

impl GoogleConfig {
    pub fn from_env() -> Option<Self> {
        std::env::var("GOOGLE_API_KEY").ok().map(|api_key| Self {
            api_key: Some(api_key),
            base_url: google_base_url(),
            default_model: google_default_model(),
            temperature: default_temperature(),
        })
    }
}
