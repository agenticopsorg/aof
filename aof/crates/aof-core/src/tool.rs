use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{AofError, AofResult};

/// Tool trait - executable capabilities for agents
///
/// Tools can be MCP tools, shell commands, HTTP APIs, etc.
/// Implementations should be lightweight and fast.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult>;

    /// Tool configuration
    fn config(&self) -> &ToolConfig;

    /// Validate tool input schema
    fn validate_input(&self, input: &ToolInput) -> AofResult<()> {
        Ok(())
    }

    /// Tool definition for model
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.config().name.clone(),
            description: self.config().description.clone(),
            parameters: self.config().parameters.clone(),
        }
    }
}

/// Tool executor - manages tool execution lifecycle
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult>;

    /// Get all available tools
    fn list_tools(&self) -> Vec<ToolDefinition>;

    /// Get specific tool
    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// JSON Schema for parameters
    pub parameters: serde_json::Value,

    /// Tool type (mcp, shell, http, etc.)
    #[serde(default)]
    pub tool_type: ToolType,

    /// Timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Extra configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_timeout() -> u64 {
    30
}

/// Tool type
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    #[default]
    Mcp,
    Shell,
    Http,
    Custom,
}

/// Tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,

    /// Context from agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

impl ToolInput {
    /// Create new tool input
    pub fn new(arguments: serde_json::Value) -> Self {
        Self {
            arguments,
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(
        arguments: serde_json::Value,
        context: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            arguments,
            context: Some(context),
        }
    }

    /// Get argument value
    pub fn get_arg<T: serde::de::DeserializeOwned>(&self, key: &str) -> AofResult<T> {
        self.arguments
            .get(key)
            .ok_or_else(|| AofError::tool(format!("Missing argument: {}", key)))
            .and_then(|v| serde_json::from_value(v.clone()).map_err(Into::into))
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Success status
    pub success: bool,

    /// Result data (JSON)
    pub data: serde_json::Value,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution time (ms)
    pub execution_time_ms: u64,
}

impl ToolResult {
    /// Create successful result
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            execution_time_ms: 0,
        }
    }

    /// Create error result
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(msg.into()),
            execution_time_ms: 0,
        }
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }
}

/// Tool call (from model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID (for tracking)
    pub id: String,

    /// Tool name
    pub name: String,

    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,
}

/// Tool definition (for model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Reference-counted tool
pub type ToolRef = Arc<dyn Tool>;
