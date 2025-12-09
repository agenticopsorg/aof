// Agent Commands - Tauri handlers for agent operations

use aof_core::{AgentConfig, AgentContext, ExecutionMetadata, Message, MessageRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::RwLock;
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

    // Create agent runtime entry
    let runtime = AgentRuntime {
        id: agent_id.clone(),
        name: agent_name.clone(),
        config: config.clone(),
        status: AgentStatus::Running,
        output: vec![format!("Starting agent: {}", agent_name)],
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
    let _ = window.emit("agent-started", serde_json::json!({
        "agent_id": agent_id,
        "name": agent_name,
    }));

    // Spawn background task for agent execution
    let state_clone = state.inner().clone();
    let window_clone = window.clone();
    let agent_id_clone = agent_id.clone();
    let input = request.input.clone();

    tokio::spawn(async move {
        execute_agent(agent_id_clone, config, input, state_clone, window_clone).await;
    });

    Ok(AgentRunResponse {
        agent_id,
        name: agent_name,
        status: "running".to_string(),
    })
}

/// Internal agent execution logic
async fn execute_agent(
    agent_id: String,
    config: AgentConfig,
    input: String,
    state: AppState,
    window: tauri::Window,
) {
    // Create agent context
    let mut ctx = AgentContext::new(&input);
    ctx.add_message(MessageRole::User, &input);

    // Simulate agent execution with output streaming
    // In a real implementation, this would use aof-runtime
    let start_time = std::time::Instant::now();

    // Emit output chunks
    for i in 0..5 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let output_line = format!(
            "[{}] Agent '{}' processing step {}/5...",
            chrono::Utc::now().format("%H:%M:%S"),
            config.name,
            i + 1
        );

        // Update state
        {
            let mut agents = state.agents.write().await;
            if let Some(runtime) = agents.get_mut(&agent_id) {
                runtime.output.push(output_line.clone());
            }
        }

        // Emit to frontend
        let _ = window.emit("agent-output", serde_json::json!({
            "agent_id": agent_id,
            "content": output_line,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));
    }

    let execution_time = start_time.elapsed().as_millis() as u64;

    // Create final result
    let result = format!(
        "Agent '{}' completed successfully.\n\nInput: {}\n\nModel: {}\nTemperature: {}\nMax Iterations: {}",
        config.name,
        input,
        config.model,
        config.temperature,
        config.max_iterations
    );

    // Update final state
    {
        let mut agents = state.agents.write().await;
        if let Some(runtime) = agents.get_mut(&agent_id) {
            runtime.status = AgentStatus::Completed;
            runtime.output.push(result.clone());
            runtime.finished_at = Some(chrono::Utc::now());
            runtime.metadata = Some(ExecutionMetadata {
                input_tokens: input.len() / 4, // Rough estimate
                output_tokens: result.len() / 4,
                execution_time_ms: execution_time,
                tool_calls: 0,
                model: Some(config.model.clone()),
            });
        }
    }

    // Emit completion event
    let _ = window.emit("agent-completed", serde_json::json!({
        "agent_id": agent_id,
        "result": result,
        "execution_time_ms": execution_time,
    }));
}

/// Stop a running agent
#[tauri::command]
pub async fn agent_stop(
    agent_id: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    let mut agents = state.agents.write().await;

    if let Some(runtime) = agents.get_mut(&agent_id) {
        if runtime.status == AgentStatus::Running {
            runtime.status = AgentStatus::Stopped;
            runtime.finished_at = Some(chrono::Utc::now());
            runtime.output.push("Agent stopped by user".to_string());

            let _ = window.emit("agent-stopped", serde_json::json!({
                "agent_id": agent_id,
            }));

            Ok(())
        } else {
            Err(format!("Agent {} is not running", agent_id))
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
        })
    } else {
        Err(format!("Agent {} not found", agent_id))
    }
}

/// List all agents
#[tauri::command]
pub async fn agent_list(
    state: State<'_, AppState>,
) -> Result<Vec<AgentStatusResponse>, String> {
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
        })
        .collect();

    Ok(list)
}

/// Clear completed agents from list
#[tauri::command]
pub async fn agent_clear_completed(
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let mut agents = state.agents.write().await;
    let initial_count = agents.len();

    agents.retain(|_, runtime| {
        runtime.status == AgentStatus::Running || runtime.status == AgentStatus::Pending
    });

    Ok(initial_count - agents.len())
}
