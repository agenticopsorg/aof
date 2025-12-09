use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{AofError, AofResult, Memory, Model, Tool};

/// Core agent trait - the foundation of AOF
///
/// Agents orchestrate models, tools, and memory to accomplish tasks.
/// Implementations should be zero-cost wrappers where possible.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Execute the agent with given input
    async fn execute(&self, ctx: &mut AgentContext) -> AofResult<String>;

    /// Agent metadata
    fn metadata(&self) -> &AgentMetadata;

    /// Initialize agent (setup resources, validate config)
    async fn init(&mut self) -> AofResult<()> {
        Ok(())
    }

    /// Cleanup agent resources
    async fn cleanup(&mut self) -> AofResult<()> {
        Ok(())
    }

    /// Validate agent configuration
    fn validate(&self) -> AofResult<()> {
        Ok(())
    }
}

/// Agent execution context - passed through the execution chain
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// User input/query
    pub input: String,

    /// Conversation history
    pub messages: Vec<Message>,

    /// Session state/variables
    pub state: HashMap<String, serde_json::Value>,

    /// Tool execution results
    pub tool_results: Vec<ToolResult>,

    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Message in conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
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

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub result: serde_json::Value,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Execution metadata
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetadata {
    /// Tokens used (input)
    pub input_tokens: usize,
    /// Tokens used (output)
    pub output_tokens: usize,
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Number of tool calls
    pub tool_calls: usize,
    /// Model used
    pub model: Option<String>,
}

impl AgentContext {
    /// Create new context with input
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            messages: Vec::new(),
            state: HashMap::new(),
            tool_results: Vec::new(),
            metadata: ExecutionMetadata::default(),
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, role: MessageRole, content: impl Into<String>) {
        self.messages.push(Message {
            role,
            content: content.into(),
            tool_calls: None,
        });
    }

    /// Get state value
    pub fn get_state<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.state
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set state value
    pub fn set_state<T: Serialize>(&mut self, key: impl Into<String>, value: T) -> AofResult<()> {
        let json_value = serde_json::to_value(value)?;
        self.state.insert(key.into(), json_value);
        Ok(())
    }
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent name
    pub name: String,

    /// Agent description
    pub description: String,

    /// Agent version
    pub version: String,

    /// Supported capabilities
    pub capabilities: Vec<String>,

    /// Custom metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,

    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Model to use
    pub model: String,

    /// Tools available to agent
    #[serde(default)]
    pub tools: Vec<String>,

    /// Memory backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,

    /// Max iterations
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Temperature (0.0-1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Max tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Custom configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_max_iterations() -> usize {
    10
}

fn default_temperature() -> f32 {
    0.7
}

/// Reference-counted agent
pub type AgentRef = Arc<dyn Agent>;
