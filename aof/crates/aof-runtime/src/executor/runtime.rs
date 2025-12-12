//! Runtime - Top-level runtime coordinator
//!
//! The Runtime loads agent configurations, creates models, tools, and memory,
//! and executes agents with proper lifecycle management.

use super::{AgentExecutor, agent_executor::StreamEvent};
use aof_core::{
    AgentConfig, AgentContext, AofError, AofResult, ModelConfig, ModelProvider, Tool,
    ToolDefinition, ToolExecutor, ToolInput,
};
use aof_llm::create_model;
use aof_mcp::McpClientBuilder;
use aof_memory::{InMemoryBackend, SimpleMemory};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

/// Top-level runtime for agent execution
///
/// The Runtime coordinates all aspects of agent execution:
/// - Loading agent configurations
/// - Creating and managing models
/// - Setting up tool executors
/// - Managing memory backends
/// - Executing agents with proper lifecycle management
pub struct Runtime {
    /// Loaded agents
    agents: HashMap<String, Arc<AgentExecutor>>,
}

impl Runtime {
    /// Create a new runtime instance
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Load an agent from YAML configuration file
    ///
    /// # Arguments
    /// * `config_path` - Path to the YAML configuration file
    ///
    /// # Returns
    /// The agent name for later execution
    pub async fn load_agent_from_file(&mut self, config_path: &str) -> AofResult<String> {
        info!("Loading agent from config file: {}", config_path);

        // Read and parse YAML config
        let config_content = tokio::fs::read_to_string(config_path).await.map_err(|e| {
            AofError::config(format!("Failed to read config file {}: {}", config_path, e))
        })?;

        let config: AgentConfig = serde_yaml::from_str(&config_content).map_err(|e| {
            AofError::config(format!("Failed to parse YAML config: {}", e))
        })?;

        self.load_agent_from_config(config).await
    }

    /// Load an agent from configuration struct
    ///
    /// # Arguments
    /// * `config` - Agent configuration
    ///
    /// # Returns
    /// The agent name for later execution
    pub async fn load_agent_from_config(&mut self, config: AgentConfig) -> AofResult<String> {
        let agent_name = config.name.clone();
        info!("Loading agent: {}", agent_name);

        // Create model from config
        let model_config = self.create_model_config(&config)?;
        let model = create_model(model_config).await?;
        debug!("Model created for agent: {}", agent_name);

        // Create tool executor if tools are specified
        let tool_executor: Option<Arc<dyn ToolExecutor>> = if !config.tools.is_empty() {
            Some(self.create_tool_executor(&config.tools).await?)
        } else {
            None
        };

        // Create memory backend
        let memory = self.create_memory(&config)?;
        debug!("Memory backend created for agent: {}", agent_name);

        // Create agent executor
        let executor = AgentExecutor::new(config, model, tool_executor, Some(memory));

        self.agents.insert(agent_name.clone(), Arc::new(executor));
        info!("Agent loaded successfully: {}", agent_name);

        Ok(agent_name)
    }

    /// Execute an agent with the given input
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `input` - User input/query
    ///
    /// # Returns
    /// The agent's final response
    pub async fn execute(&self, agent_name: &str, input: &str) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        let mut context = AgentContext::new(input);
        executor.execute(&mut context).await
    }

    /// Execute an agent with a pre-built context
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `context` - Pre-configured agent context
    ///
    /// # Returns
    /// The agent's final response
    pub async fn execute_with_context(
        &self,
        agent_name: &str,
        context: &mut AgentContext,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        executor.execute(context).await
    }

    /// Execute an agent with streaming support for real-time updates
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `input` - User input/query
    /// * `stream_tx` - Channel sender for streaming events
    ///
    /// # Returns
    /// The agent's final response
    ///
    /// # Example
    /// ```no_run
    /// use tokio::sync::mpsc;
    /// # use aof_runtime::Runtime;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runtime = Runtime::new();
    /// runtime.load_agent_from_file("config.yaml").await?;
    ///
    /// let (tx, mut rx) = mpsc::channel(100);
    ///
    /// // Spawn task to handle stream events
    /// tokio::spawn(async move {
    ///     while let Some(event) = rx.recv().await {
    ///         println!("Event: {:?}", event);
    ///     }
    /// });
    ///
    /// let result = runtime.execute_streaming("my-agent", "Hello", tx).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_streaming(
        &self,
        agent_name: &str,
        input: &str,
        stream_tx: mpsc::Sender<StreamEvent>,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        let mut context = AgentContext::new(input);
        executor.execute_streaming(&mut context, stream_tx).await
    }

    /// Execute an agent with streaming and a pre-built context
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `context` - Pre-configured agent context
    /// * `stream_tx` - Channel sender for streaming events
    ///
    /// # Returns
    /// The agent's final response
    pub async fn execute_streaming_with_context(
        &self,
        agent_name: &str,
        context: &mut AgentContext,
        stream_tx: mpsc::Sender<StreamEvent>,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        executor.execute_streaming(context, stream_tx).await
    }

    /// Execute an agent with streaming and cancellation support
    ///
    /// # Arguments
    /// * `agent_name` - Name of the loaded agent
    /// * `input` - User input/query
    /// * `stream_tx` - Channel sender for streaming events
    /// * `cancel_rx` - Channel receiver for cancellation signal
    ///
    /// # Returns
    /// The agent's final response or cancellation error
    ///
    /// # Example
    /// ```no_run
    /// use tokio::sync::{mpsc, oneshot};
    /// # use aof_runtime::Runtime;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runtime = Runtime::new();
    /// runtime.load_agent_from_file("config.yaml").await?;
    ///
    /// let (stream_tx, mut stream_rx) = mpsc::channel(100);
    /// let (cancel_tx, cancel_rx) = oneshot::channel();
    ///
    /// // Spawn task to handle cancellation
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    ///     let _ = cancel_tx.send(());
    /// });
    ///
    /// let result = runtime.execute_streaming_cancellable(
    ///     "my-agent",
    ///     "Long running task",
    ///     stream_tx,
    ///     cancel_rx
    /// ).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_streaming_cancellable(
        &self,
        agent_name: &str,
        input: &str,
        stream_tx: mpsc::Sender<StreamEvent>,
        mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> AofResult<String> {
        let executor = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AofError::agent(format!("Agent not found: {}", agent_name)))?;

        let mut context = AgentContext::new(input);

        tokio::select! {
            result = executor.execute_streaming(&mut context, stream_tx.clone()) => {
                result
            }
            _ = &mut cancel_rx => {
                let _ = stream_tx.send(StreamEvent::Error {
                    message: "Execution cancelled by user".to_string(),
                }).await;
                Err(AofError::agent("Execution cancelled".to_string()))
            }
        }
    }

    /// List all loaded agents
    pub fn list_agents(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Get agent executor by name
    pub fn get_agent(&self, name: &str) -> Option<Arc<AgentExecutor>> {
        self.agents.get(name).cloned()
    }

    // Helper: Create model config from agent config
    fn create_model_config(&self, config: &AgentConfig) -> AofResult<ModelConfig> {
        // Parse model string (format: "provider:model" or just "model")
        let (provider, model) = if config.model.contains(':') {
            let parts: Vec<&str> = config.model.splitn(2, ':').collect();
            let provider = match parts[0].to_lowercase().as_str() {
                "anthropic" => ModelProvider::Anthropic,
                "openai" => ModelProvider::OpenAI,
                "google" => ModelProvider::Google,
                "bedrock" => ModelProvider::Bedrock,
                "azure" => ModelProvider::Azure,
                "ollama" => ModelProvider::Ollama,
                "groq" => ModelProvider::Groq,
                _ => ModelProvider::Custom,
            };
            (provider, parts[1].to_string())
        } else {
            // Default to Anthropic if no provider specified
            (ModelProvider::Anthropic, config.model.clone())
        };

        Ok(ModelConfig {
            model,
            provider,
            api_key: None, // Will use environment variables
            endpoint: None,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        })
    }

    // Helper: Create tool executor from tool list
    async fn create_tool_executor(
        &self,
        tool_names: &[String],
    ) -> AofResult<Arc<dyn ToolExecutor>> {
        info!("Creating tool executor with {} tools", tool_names.len());

        let mcp_client = McpClientBuilder::new()
            .stdio(
                "npx",
                vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-everything".to_string(),
                ],
            )
            .build()
            .map_err(|e| AofError::tool(format!("Failed to create MCP client: {}", e)))?;

        // CRITICAL: Initialize the MCP client with server-specific options
        // The server-everything package requires 'roots' for filesystem access
        let init_options = serde_json::json!({
            "roots": ["/tmp", "/"],
            "maxDepth": 10
        });

        mcp_client.initialize_with_options(Some(init_options))
            .await
            .map_err(|e| AofError::tool(format!("Failed to initialize MCP client: {}", e)))?;

        info!("MCP client initialized successfully with server options");

        Ok(Arc::new(McpToolExecutor {
            client: Arc::new(mcp_client),
            tool_names: tool_names.to_vec(),
        }))
    }

    // Helper: Create memory backend
    fn create_memory(&self, _config: &AgentConfig) -> AofResult<Arc<SimpleMemory>> {
        let backend = InMemoryBackend::new();
        Ok(Arc::new(SimpleMemory::new(Arc::new(backend))))
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP-based tool executor implementation
struct McpToolExecutor {
    client: Arc<aof_mcp::McpClient>,
    tool_names: Vec<String>,
}

#[async_trait]
impl ToolExecutor for McpToolExecutor {
    async fn execute_tool(
        &self,
        name: &str,
        input: ToolInput,
    ) -> AofResult<aof_core::ToolResult> {
        debug!("Executing MCP tool: {}", name);
        let start = std::time::Instant::now();

        // Call MCP tool
        let result = self
            .client
            .call_tool(name, input.arguments)
            .await
            .map_err(|e| AofError::tool(format!("MCP tool call failed: {}", e)))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(aof_core::ToolResult {
            success: true,
            data: result,
            error: None,
            execution_time_ms,
        })
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        // In a real implementation, this would query MCP for tool definitions
        // For now, return basic definitions
        self.tool_names
            .iter()
            .map(|name| ToolDefinition {
                name: name.clone(),
                description: format!("MCP tool: {}", name),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                }),
            })
            .collect()
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn Tool>> {
        // MCP tools are dynamically resolved, not stored as objects
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert_eq!(runtime.list_agents().len(), 0);
    }

    #[test]
    fn test_model_config_parsing() {
        let runtime = Runtime::new();

        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "anthropic:claude-3-5-sonnet-20241022".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        let model_config = runtime.create_model_config(&config).unwrap();
        assert_eq!(model_config.provider, ModelProvider::Anthropic);
        assert_eq!(model_config.model, "claude-3-5-sonnet-20241022");
        assert_eq!(model_config.temperature, 0.7);
    }

    #[test]
    fn test_model_config_default_provider() {
        let runtime = Runtime::new();

        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "gpt-4".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        let model_config = runtime.create_model_config(&config).unwrap();
        assert_eq!(model_config.provider, ModelProvider::Anthropic);
        assert_eq!(model_config.model, "gpt-4");
    }
}
