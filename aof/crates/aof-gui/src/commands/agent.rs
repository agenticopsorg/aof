// Agent Commands - Tauri handlers for agent operations with aof-runtime integration

use aof_core::{AgentConfig, AgentContext, ExecutionMetadata, MessageRole, ModelConfig, ModelProvider};
use aof_llm::ProviderFactory;
use aof_runtime::{AgentExecutor, Task, TaskStatus};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use uuid::Uuid;

use crate::state::AppState;

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
    pub output: Vec<String>,
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
    pub output: Vec<String>,
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

    // Validate API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set. Please configure your API key.".to_string())?;

    // Create agent runtime entry
    let runtime = AgentRuntime {
        id: agent_id.clone(),
        name: agent_name.clone(),
        config: config.clone(),
        status: AgentStatus::Pending,
        output: vec![format!("Initializing agent: {}", agent_name)],
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
    let handle = state.orchestrator.submit_task(task);

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
        provider: ModelProvider::Anthropic,
        api_key: Some(api_key.clone()),
        endpoint: None,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        timeout_secs: 60,
        headers: std::collections::HashMap::new(),
        extra: std::collections::HashMap::new(),
    };

    let model = match ProviderFactory::create(model_config).await {
        Ok(m) => m,
        Err(e) => {
            let error_msg = format!("Failed to create model: {}", e);
            handle_execution_error(&agent_id, &error_msg, &state, &window).await;
            return;
        }
    };

    // Create tool executor if tools are specified in config
    let tool_executor: Option<std::sync::Arc<dyn aof_core::ToolExecutor>> = if !config.tools.is_empty() {
        // Create MCP-based tool executor
        use aof_mcp::McpClientBuilder;
        use aof_core::{ToolDefinition, ToolInput, Tool};
        use async_trait::async_trait;

        match McpClientBuilder::new()
            .stdio(
                "npx",
                vec!["-y".to_string(), "@modelcontextprotocol/server-everything".to_string()],
            )
            .build()
        {
            Ok(mcp_client) => {
                // Create a simple MCP tool executor
                struct McpToolExecutor {
                    client: std::sync::Arc<aof_mcp::McpClient>,
                    tool_names: Vec<String>,
                }

                #[async_trait]
                impl aof_core::ToolExecutor for McpToolExecutor {
                    async fn execute_tool(
                        &self,
                        name: &str,
                        input: ToolInput,
                    ) -> aof_core::AofResult<aof_core::ToolResult> {
                        let start = std::time::Instant::now();
                        let result = self
                            .client
                            .call_tool(name, input.arguments)
                            .await
                            .map_err(|e| aof_core::AofError::tool(format!("MCP tool call failed: {}", e)))?;

                        let execution_time_ms = start.elapsed().as_millis() as u64;

                        Ok(aof_core::ToolResult {
                            success: true,
                            data: result,
                            error: None,
                            execution_time_ms,
                        })
                    }

                    fn list_tools(&self) -> Vec<ToolDefinition> {
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

                    fn get_tool(&self, _name: &str) -> Option<std::sync::Arc<dyn Tool>> {
                        None
                    }
                }

                Some(std::sync::Arc::new(McpToolExecutor {
                    client: std::sync::Arc::new(mcp_client),
                    tool_names: config.tools.clone(),
                }))
            }
            Err(e) => {
                eprintln!("Warning: Failed to create MCP client for tools: {}", e);
                None
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

    // Emit progress update
    let _ = window.emit(
        "agent-output",
        serde_json::json!({
            "agent_id": agent_id,
            "content": "Calling LLM model...",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );

    // Execute agent
    let start_time = std::time::Instant::now();
    let result = executor.execute(&mut ctx).await;

    let execution_time = start_time.elapsed().as_millis() as u64;

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
                    runtime.output.push(format!(
                        "[{}] Agent completed successfully",
                        chrono::Utc::now().format("%H:%M:%S")
                    ));
                    runtime.output.push(output.clone());
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
