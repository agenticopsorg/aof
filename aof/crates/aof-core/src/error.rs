use thiserror::Error;

/// Main error type for AOF framework
#[derive(Error, Debug)]
pub enum AofError {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Tool execution error: {0}")]
    Tool(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("MCP protocol error: {0}")]
    Mcp(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Type alias for Results using AofError
pub type AofResult<T> = Result<T, AofError>;

impl AofError {
    /// Create an agent error
    pub fn agent(msg: impl Into<String>) -> Self {
        Self::Agent(msg.into())
    }

    /// Create a model error
    pub fn model(msg: impl Into<String>) -> Self {
        Self::Model(msg.into())
    }

    /// Create a tool error
    pub fn tool(msg: impl Into<String>) -> Self {
        Self::Tool(msg.into())
    }

    /// Create a memory error
    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }

    /// Create an MCP error
    pub fn mcp(msg: impl Into<String>) -> Self {
        Self::Mcp(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}
