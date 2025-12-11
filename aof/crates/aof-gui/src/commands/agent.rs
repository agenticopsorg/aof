// Agent Commands - Tauri handlers for agent operations with aof-runtime integration

use aof_core::{AgentConfig, AgentContext, ExecutionMetadata, MessageRole, ModelConfig, ModelProvider};
use aof_llm::ProviderFactory;
use aof_runtime::{AgentExecutor, Task, TaskStatus};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use uuid::Uuid;

use crate::state::AppState;
use crate::commands::mcp::auto_connect_for_tools;

/// Agent run request
#[derive(Debug, Deserialize)]
pub struct AgentRunRequest {
    pub config_yaml: String,
    pub input: String,
}

/// Agent run response
#[derive(Debug, Serialize)]
pub struct AgentRunResponse {
    pub agent_id: String,
    pub name: String,
    pub status: String,
}

/// Agent status response
#[derive(Debug, Serialize)]
pub struct AgentStatusResponse {
    pub agent_id: String,
    pub name: String,
    pub status: AgentStatus,
    pub output: Vec<String>,       // Status logs (legacy, kept for compatibility)
    pub response: Option<String>,  // The actual LLM response
    pub metadata: Option<ExecutionMetadataResponse>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<String>,
}

/// Execution metadata for frontend
#[derive(Debug, Serialize, Clone)]
pub struct ExecutionMetadataResponse {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub execution_time_ms: u64,
    pub tool_calls: usize,
    pub model: Option<String>,
}

impl From<ExecutionMetadata> for ExecutionMetadataResponse {
    fn from(meta: ExecutionMetadata) -> Self {
        Self {
            input_tokens: meta.input_tokens,
            output_tokens: meta.output_tokens,
            execution_time_ms: meta.execution_time_ms,
            tool_calls: meta.tool_calls,
            model: meta.model,
        }
    }
}

/// Agent execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Stopped,
}

impl From<TaskStatus> for AgentStatus {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => AgentStatus::Pending,
            TaskStatus::Running => AgentStatus::Running,
            TaskStatus::Completed => AgentStatus::Completed,
            TaskStatus::Failed => AgentStatus::Failed,
            TaskStatus::Cancelled => AgentStatus::Stopped,
        }
    }
}

/// Stored agent runtime information
#[derive(Debug, Clone)]
pub struct AgentRuntime {
    pub id: String,
    pub name: String,
    pub config: AgentConfig,
    pub status: AgentStatus,
    pub output: Vec<String>,      // Legacy: status logs
    pub response: Option<String>, // The actual LLM response
    pub metadata: Option<ExecutionMetadata>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// Run an agent from YAML configuration
#[tauri::command]
pub async fn agent_run(
    request: AgentRunRequest,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<AgentRunResponse, String> {
    // Parse the YAML configuration
    let config: AgentConfig = serde_yaml::from_str(&request.config_yaml)
        .map_err(|e| format!("Failed to parse agent config: {}", e))?;

    let agent_id = Uuid::new_v4().to_string();
    let agent_name = config.name.clone();

    // Auto-connect to MCP servers if tools are configured
    if !config.tools.is_empty() {
        tracing::info!("Agent {} requires tools: {:?}, checking auto-connect...", agent_name, config.tools);

        // Emit event to frontend
        let _ = window.emit(
            "agent-output",
            serde_json::json!({
                "agent_id": agent_id,
                "content": format!("Checking MCP connections for tools: {:?}", config.tools),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        );

        match auto_connect_for_tools(&config.tools, state.inner(), &window).await {
            Ok(connected) => {
                if !connected.is_empty() {
                    tracing::info!("Auto-connected {} MCP servers for tools", connected.len());
                    let _ = window.emit(
                        "agent-output",
                        serde_json::json!({
                            "agent_id": agent_id,
                            "content": format!("Auto-connected {} MCP server(s) for tools", connected.len()),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }),
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Failed to auto-connect MCP servers: {}", e);
                let _ = window.emit(
                    "agent-output",
                    serde_json::json!({
                        "agent_id": agent_id,
                        "content": format!("Warning: Could not auto-connect MCP servers: {}", e),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }),
                );
            }
        }
    }

    // Determine provider from model name and get appropriate API key
    let (provider, api_key_var) = if config.model.starts_with("gemini") || config.model.starts_with("google/") {
        (ModelProvider::Google, "GOOGLE_API_KEY")
    } else if config.model.starts_with("claude") || config.model.starts_with("anthropic") {
        (ModelProvider::Anthropic, "ANTHROPIC_API_KEY")
    } else if config.model.starts_with("gpt") || config.model.starts_with("openai") || config.model.starts_with("o1") || config.model.starts_with("o3") {
        (ModelProvider::OpenAI, "OPENAI_API_KEY")
    } else if config.model.starts_with("llama") && !config.model.contains("groq") {
        (ModelProvider::Ollama, "OLLAMA_HOST")
    } else if config.model.starts_with("mistral") || config.model.starts_with("codellama") || config.model.starts_with("phi") {
        (ModelProvider::Ollama, "OLLAMA_HOST")
    } else if config.model.contains("groq") || config.model.contains("mixtral") {
        (ModelProvider::Groq, "GROQ_API_KEY")
    } else {
        // Default to trying Google for unknown models
        (ModelProvider::Google, "GOOGLE_API_KEY")
    };

    // Get API key - first try database, then fall back to environment variable
    let provider_name = match provider {
        ModelProvider::Google => "google",
        ModelProvider::Anthropic => "anthropic",
        ModelProvider::OpenAI => "openai",
        ModelProvider::Groq => "groq",
        ModelProvider::Ollama => "ollama",
        _ => "unknown",
    };

    let api_key = if provider == ModelProvider::Ollama {
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string())
    } else {
        // Try to get from database first
        let db_key = if let Ok(db) = state.get_db().await {
            tracing::info!("Checking database for {} API key...", provider_name);
            match sqlx::query_scalar::<_, String>("SELECT api_key FROM provider_api_keys WHERE provider = ?")
                .bind(provider_name)
                .fetch_optional(&db)
                .await
            {
                Ok(Some(key)) => {
                    tracing::info!("✓ Found API key in database for {} (length: {})", provider_name, key.len());
                    Some(key)
                }
                Ok(None) => {
                    tracing::warn!("No API key found in database for {}", provider_name);
                    None
                }
                Err(e) => {
                    tracing::error!("Database query failed for {} API key: {}", provider_name, e);
                    None
                }
            }
        } else {
            tracing::warn!("Could not get database connection");
            None
        };

        // Fall back to environment variable if not in database
        let final_key = if let Some(key) = db_key {
            tracing::info!("Using API key from DATABASE for {}", provider_name);
            Some(key)
        } else if let Ok(env_key) = std::env::var(api_key_var) {
            tracing::info!("Using API key from ENVIRONMENT ({}) for {} (length: {})", api_key_var, provider_name, env_key.len());
            Some(env_key)
        } else {
            tracing::error!("No API key found for {} in database or environment ({})", provider_name, api_key_var);
            None
        };

        final_key.ok_or_else(|| format!("No API key found for {} provider. Please configure your API key in Settings or set the {} environment variable.", provider_name, api_key_var))?
    };

    // Create agent runtime entry
    let runtime = AgentRuntime {
        id: agent_id.clone(),
        name: agent_name.clone(),
        config: config.clone(),
        status: AgentStatus::Pending,
        output: vec![],  // Status logs (not shown in main output)
        response: None,  // The actual LLM response
        metadata: None,
        started_at: Some(chrono::Utc::now()),
        finished_at: None,
        error: None,
    };

    // Store in state
    {
        let mut agents = state.agents.write().await;
        agents.insert(agent_id.clone(), runtime);
    }

    // Emit event to frontend
    let _ = window.emit(
        "agent-started",
        serde_json::json!({
            "agent_id": agent_id,
            "name": agent_name,
        }),
    );

    // Create task for orchestrator
    let task = Task::new(
        agent_id.clone(),
        format!("Execute agent: {}", agent_name),
        agent_name.clone(),
        request.input.clone(),
    );

    // Submit task to orchestrator
    let _handle = state.orchestrator.submit_task(task);

    // Spawn background task for agent execution
    let state_clone = state.inner().clone();
    let window_clone = window.clone();
    let agent_id_clone = agent_id.clone();
    let input = request.input.clone();

    tokio::spawn(async move {
        execute_agent_with_runtime(
            agent_id_clone,
            config,
            input,
            api_key,
            provider,
            state_clone,
            window_clone,
        )
        .await;
    });

    Ok(AgentRunResponse {
        agent_id,
        name: agent_name,
        status: "running".to_string(),
    })
}

/// Internal agent execution logic using aof-runtime
async fn execute_agent_with_runtime(
    agent_id: String,
    config: AgentConfig,
    input: String,
    api_key: String,
    provider: ModelProvider,
    state: AppState,
    window: tauri::Window,
) {
    // Update status to running
    {
        let mut agents = state.agents.write().await;
        if let Some(runtime) = agents.get_mut(&agent_id) {
            runtime.status = AgentStatus::Running;
            runtime.output.push(format!(
                "[{}] Starting agent execution...",
                chrono::Utc::now().format("%H:%M:%S")
            ));
        }
    }

    // Emit status update
    let _ = window.emit(
        "agent-output",
        serde_json::json!({
            "agent_id": agent_id,
            "content": format!("Starting execution with model: {}", config.model),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );

    // Create LLM model
    let model_config = ModelConfig {
        model: config.model.clone(),
        provider,
        api_key: Some(api_key.clone()),
        endpoint: None,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        timeout_secs: 60,
        headers: std::collections::HashMap::new(),
        extra: std::collections::HashMap::new(),
    };

    let model = match ProviderFactory::create(model_config).await {
        Ok(m) => {
            tracing::info!("Successfully created model provider for {}", config.model);
            let _ = window.emit(
                "agent-output",
                serde_json::json!({
                    "agent_id": agent_id,
                    "content": format!("Model provider created successfully for {}", config.model),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            );
            m
        }
        Err(e) => {
            let error_msg = format!("Failed to create model: {}", e);
            tracing::error!("{}", error_msg);
            handle_execution_error(&agent_id, &error_msg, &state, &window).await;
            return;
        }
    };

    // Create tool executor using MCP connections from app state
    let tool_executor: Option<std::sync::Arc<dyn aof_core::ToolExecutor>> = if !config.tools.is_empty() {
        use aof_core::{ToolDefinition, ToolInput, Tool};
        use async_trait::async_trait;

        // Get all available tools from connected MCP servers
        let mcp_connections = state.mcp_connections.read().await;
        tracing::warn!("[TOOL_SETUP] MCP connections count: {}", mcp_connections.len());

        if mcp_connections.is_empty() {
            tracing::warn!("[TOOL_SETUP] No MCP servers connected! Tools will not be available.");
            // Emit warning to frontend
            let _ = window.emit(
                "agent-output",
                serde_json::json!({
                    "agent_id": agent_id,
                    "content": "⚠️ No MCP servers connected. Tools will not be available. Connect MCP servers in the MCP Tools tab.",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            );
            None
        } else {
            // Collect all tool definitions from connected MCP servers
            let mut all_tools: Vec<ToolDefinition> = Vec::new();
            let mut tool_to_connection: std::collections::HashMap<String, String> = std::collections::HashMap::new();

            for (conn_id, connection) in mcp_connections.iter() {
                for tool in &connection.tools {
                    // Only include tools that are in the config
                    if config.tools.contains(&tool.name) || config.tools.is_empty() {
                        all_tools.push(tool.clone());
                        tool_to_connection.insert(tool.name.clone(), conn_id.clone());
                    }
                }
            }

            if all_tools.is_empty() {
                let available = mcp_connections.values().flat_map(|c| c.tools.iter().map(|t| &t.name)).collect::<Vec<_>>();
                tracing::warn!("[TOOL_SETUP] Requested tools {:?} not found. Available: {:?}", config.tools, available);
                let _ = window.emit(
                    "agent-output",
                    serde_json::json!({
                        "agent_id": agent_id,
                        "content": format!("⚠️ Requested tools {:?} not found in connected MCP servers. Available tools: {:?}",
                            config.tools,
                            available
                        ),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }),
                );
                None
            } else {
                // Log available tools
                let _ = window.emit(
                    "agent-output",
                    serde_json::json!({
                        "agent_id": agent_id,
                        "content": format!("Found {} MCP tools available: {:?}",
                            all_tools.len(),
                            all_tools.iter().map(|t| &t.name).collect::<Vec<_>>()
                        ),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }),
                );

                // Create multi-connection tool executor
                struct McpMultiToolExecutor {
                    connections: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, crate::commands::mcp::McpConnection>>>,
                    tool_to_connection: std::collections::HashMap<String, String>,
                    all_tools: Vec<ToolDefinition>,
                }

                #[async_trait]
                impl aof_core::ToolExecutor for McpMultiToolExecutor {
                    async fn execute_tool(
                        &self,
                        name: &str,
                        input: ToolInput,
                    ) -> aof_core::AofResult<aof_core::ToolResult> {
                        tracing::warn!("[MCP_EXECUTOR] execute_tool called: name={}", name);
                        let start = std::time::Instant::now();

                        // Find which connection has this tool
                        let conn_id = self.tool_to_connection.get(name)
                            .ok_or_else(|| {
                                tracing::error!("[MCP_EXECUTOR] Tool '{}' not found in tool_to_connection map", name);
                                aof_core::AofError::tool(format!("Tool '{}' not found in any connected MCP server", name))
                            })?;

                        tracing::warn!("[MCP_EXECUTOR] Found tool in connection: {}", conn_id);

                        let connections = self.connections.read().await;
                        let connection = connections.get(conn_id)
                            .ok_or_else(|| {
                                tracing::error!("[MCP_EXECUTOR] Connection '{}' no longer available", conn_id);
                                aof_core::AofError::tool(format!("MCP connection '{}' no longer available", conn_id))
                            })?;

                        tracing::warn!("[MCP_EXECUTOR] Calling MCP tool: {} with args: {:?}", name, input.arguments);
                        let result = connection.client
                            .call_tool(name, input.arguments)
                            .await
                            .map_err(|e| {
                                tracing::error!("[MCP_EXECUTOR] MCP tool call failed: {}", e);
                                aof_core::AofError::tool(format!("MCP tool call failed: {}", e))
                            })?;

                        let execution_time_ms = start.elapsed().as_millis() as u64;
                        tracing::warn!("[MCP_EXECUTOR] Tool completed in {}ms", execution_time_ms);

                        Ok(aof_core::ToolResult {
                            success: true,
                            data: result,
                            error: None,
                            execution_time_ms,
                        })
                    }

                    fn list_tools(&self) -> Vec<ToolDefinition> {
                        tracing::warn!("[MCP_EXECUTOR] list_tools called, returning {} tools", self.all_tools.len());
                        for tool in &self.all_tools {
                            tracing::warn!("[MCP_EXECUTOR] Available tool: {}", tool.name);
                        }
                        self.all_tools.clone()
                    }

                    fn get_tool(&self, _name: &str) -> Option<std::sync::Arc<dyn Tool>> {
                        None
                    }
                }

                Some(std::sync::Arc::new(McpMultiToolExecutor {
                    connections: state.mcp_connections.clone(),
                    tool_to_connection,
                    all_tools,
                }))
            }
        }
    } else {
        None
    };

    // Create memory backend
    let memory = {
        use aof_memory::{InMemoryBackend, SimpleMemory};
        let backend = InMemoryBackend::new();
        Some(std::sync::Arc::new(SimpleMemory::new(std::sync::Arc::new(backend))))
    };

    // Create agent executor
    let executor = AgentExecutor::new(
        config.clone(),
        model,
        tool_executor,
        memory,
    );

    // Create agent context
    let mut ctx = AgentContext::new(&input);
    ctx.add_message(MessageRole::User, &input);

    // Emit progress update with config info for debugging
    let _ = window.emit(
        "agent-output",
        serde_json::json!({
            "agent_id": agent_id,
            "content": format!("Calling LLM model: {} (provider: {:?})", config.model, provider),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );

    tracing::info!("Agent {} executing with model {} ({:?})", agent_id, config.model, provider);
    tracing::info!("System prompt: {:?}", config.system_prompt.as_ref().map(|s| s.chars().take(100).collect::<String>()));
    tracing::info!("Tools: {:?}", config.tools);

    // Execute agent
    let start_time = std::time::Instant::now();
    tracing::info!("Starting executor.execute()...");

    let _ = window.emit(
        "agent-output",
        serde_json::json!({
            "agent_id": agent_id,
            "content": "Sending request to LLM API...",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );

    let result = executor.execute(&mut ctx).await;

    let elapsed = start_time.elapsed().as_millis();

    // Detailed logging based on result
    match &result {
        Ok(output) => {
            tracing::info!("executor.execute() SUCCESS in {}ms, output length: {}", elapsed, output.len());
        }
        Err(e) => {
            tracing::error!("executor.execute() FAILED in {}ms: {:?}", elapsed, e);
            // Also emit the error to the frontend immediately
            let _ = window.emit(
                "agent-output",
                serde_json::json!({
                    "agent_id": agent_id,
                    "content": format!("ERROR: {}", e),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            );
        }
    }

    let _ = window.emit(
        "agent-output",
        serde_json::json!({
            "agent_id": agent_id,
            "content": format!("LLM API call completed in {}ms", elapsed),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );

    let execution_time = elapsed as u64;

    // Handle execution result
    match result {
        Ok(output) => {
            // Stream output chunks to frontend
            let output_lines: Vec<&str> = output.lines().collect();
            for (i, line) in output_lines.iter().enumerate() {
                if !line.trim().is_empty() {
                    let _ = window.emit(
                        "agent-output",
                        serde_json::json!({
                            "agent_id": agent_id,
                            "content": line,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }),
                    );
                }

                // Add small delay for natural streaming effect
                if i < output_lines.len() - 1 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }

            // Update final state
            {
                let mut agents = state.agents.write().await;
                if let Some(runtime) = agents.get_mut(&agent_id) {
                    runtime.status = AgentStatus::Completed;
                    runtime.response = Some(output.clone()); // Store actual response separately
                    runtime.finished_at = Some(chrono::Utc::now());
                    runtime.metadata = Some(ctx.metadata.clone());
                }
            }

            // Emit completion event
            let _ = window.emit(
                "agent-completed",
                serde_json::json!({
                    "agent_id": agent_id,
                    "result": output,
                    "execution_time_ms": execution_time,
                    "metadata": ExecutionMetadataResponse::from(ctx.metadata),
                }),
            );
        }
        Err(e) => {
            let error_msg = format!("Execution failed: {}", e);
            handle_execution_error(&agent_id, &error_msg, &state, &window).await;
        }
    }
}

/// Handle execution errors with proper state updates and events
async fn handle_execution_error(
    agent_id: &str,
    error_msg: &str,
    state: &AppState,
    window: &tauri::Window,
) {
    // Update state
    {
        let mut agents = state.agents.write().await;
        if let Some(runtime) = agents.get_mut(agent_id) {
            runtime.status = AgentStatus::Failed;
            runtime.finished_at = Some(chrono::Utc::now());
            runtime.error = Some(error_msg.to_string());
            runtime.output.push(format!(
                "[{}] Error: {}",
                chrono::Utc::now().format("%H:%M:%S"),
                error_msg
            ));
        }
    }

    // Emit error event
    let _ = window.emit(
        "agent-error",
        serde_json::json!({
            "agent_id": agent_id,
            "error": error_msg,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );
}

/// Stop a running agent
#[tauri::command]
pub async fn agent_stop(
    agent_id: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    // Try to cancel the task in orchestrator
    let _ = state.orchestrator.cancel_task(&agent_id).await;

    // Update agent state
    let mut agents = state.agents.write().await;

    if let Some(runtime) = agents.get_mut(&agent_id) {
        if runtime.status == AgentStatus::Running || runtime.status == AgentStatus::Pending {
            runtime.status = AgentStatus::Stopped;
            runtime.finished_at = Some(chrono::Utc::now());
            runtime.output.push(format!(
                "[{}] Agent stopped by user",
                chrono::Utc::now().format("%H:%M:%S")
            ));

            let _ = window.emit(
                "agent-stopped",
                serde_json::json!({
                    "agent_id": agent_id,
                }),
            );

            Ok(())
        } else {
            Err(format!(
                "Agent {} is not running (status: {:?})",
                agent_id, runtime.status
            ))
        }
    } else {
        Err(format!("Agent {} not found", agent_id))
    }
}

/// Get agent status
#[tauri::command]
pub async fn agent_status(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<AgentStatusResponse, String> {
    let agents = state.agents.read().await;

    if let Some(runtime) = agents.get(&agent_id) {
        Ok(AgentStatusResponse {
            agent_id: runtime.id.clone(),
            name: runtime.name.clone(),
            status: runtime.status.clone(),
            output: runtime.output.clone(),
            response: runtime.response.clone(),
            metadata: runtime.metadata.clone().map(|m| m.into()),
            started_at: runtime.started_at.map(|t| t.to_rfc3339()),
            finished_at: runtime.finished_at.map(|t| t.to_rfc3339()),
            error: runtime.error.clone(),
        })
    } else {
        Err(format!("Agent {} not found", agent_id))
    }
}

/// List all agents
#[tauri::command]
pub async fn agent_list(state: State<'_, AppState>) -> Result<Vec<AgentStatusResponse>, String> {
    let agents = state.agents.read().await;

    let list: Vec<AgentStatusResponse> = agents
        .values()
        .map(|runtime| AgentStatusResponse {
            agent_id: runtime.id.clone(),
            name: runtime.name.clone(),
            status: runtime.status.clone(),
            output: runtime.output.clone(),
            response: runtime.response.clone(),
            metadata: runtime.metadata.clone().map(|m| m.into()),
            started_at: runtime.started_at.map(|t| t.to_rfc3339()),
            finished_at: runtime.finished_at.map(|t| t.to_rfc3339()),
            error: runtime.error.clone(),
        })
        .collect();

    Ok(list)
}

/// Clear completed agents from list
#[tauri::command]
pub async fn agent_clear_completed(state: State<'_, AppState>) -> Result<usize, String> {
    let mut agents = state.agents.write().await;
    let initial_count = agents.len();

    agents.retain(|_, runtime| {
        runtime.status == AgentStatus::Running || runtime.status == AgentStatus::Pending
    });

    // Also cleanup finished tasks in orchestrator
    state.orchestrator.cleanup_finished_tasks().await;

    Ok(initial_count - agents.len())
}

/// Get orchestrator statistics
#[tauri::command]
pub async fn agent_orchestrator_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let stats = state.orchestrator.stats().await;

    Ok(serde_json::json!({
        "pending": stats.pending,
        "running": stats.running,
        "completed": stats.completed,
        "failed": stats.failed,
        "cancelled": stats.cancelled,
        "max_concurrent": stats.max_concurrent,
        "available_permits": stats.available_permits,
    }))
}
