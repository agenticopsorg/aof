use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::{AofError, AofResult};

/// Model provider trait - abstraction over LLM providers
///
/// Implementations should minimize allocations and use zero-copy where possible.
#[async_trait]
pub trait Model: Send + Sync {
    /// Generate completion (non-streaming)
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse>;

    /// Generate completion (streaming)
    async fn generate_stream(
        &self,
        request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>>;

    /// Model configuration
    fn config(&self) -> &ModelConfig;

    /// Provider type
    fn provider(&self) -> ModelProvider;

    /// Count tokens in text (approximate)
    fn count_tokens(&self, text: &str) -> usize {
        // Default: rough approximation (4 chars per token)
        text.len() / 4
    }
}

/// Model provider enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    Anthropic,
    OpenAI,
    Bedrock,
    Azure,
    Ollama,
    Custom,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model identifier (e.g., "claude-3-5-sonnet-20241022")
    pub model: String,

    /// Provider
    pub provider: ModelProvider,

    /// API key (optional, can use env var)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API endpoint (for custom providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Temperature (0.0-1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Max tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Custom headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Extra provider-specific config
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout() -> u64 {
    60
}

/// Model request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    /// Messages in conversation
    pub messages: Vec<RequestMessage>,

    /// System prompt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Tools available
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<ToolDefinition>,

    /// Temperature override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Max tokens override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Stream response
    #[serde(default)]
    pub stream: bool,

    /// Extra parameters
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Message in request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::ToolCall>>,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool definition for model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Model response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// Generated text
    pub content: String,

    /// Tool calls requested by model
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_calls: Vec<crate::ToolCall>,

    /// Stop reason
    pub stop_reason: StopReason,

    /// Usage statistics
    pub usage: Usage,

    /// Provider-specific metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Stop reason
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    ContentFilter,
}

/// Token usage statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    ContentDelta { delta: String },
    ToolCall { tool_call: crate::ToolCall },
    Done { usage: Usage, stop_reason: StopReason },
}

/// Reference-counted model
pub type ModelRef = Arc<dyn Model>;
