//! Integration test: GUI command → Runtime execution

use aof_core::{
    AgentConfig, AgentContext, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, StopReason, StreamChunk, Usage,
};
use aof_runtime::AgentExecutor;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

/// GUI command structure (simulating Tauri commands)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GuiCommand {
    Execute {
        agent_name: String,
        input: String,
        config: Option<AgentConfig>,
    },
    Status {
        agent_name: String,
    },
    ListAgents,
    GetHistory {
        session_id: String,
    },
}

/// GUI response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuiResponse {
    success: bool,
    data: serde_json::Value,
    error: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl GuiResponse {
    fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metadata: HashMap::new(),
        }
    }

    fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(msg.into()),
            metadata: HashMap::new(),
        }
    }

    fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Mock model for GUI testing
struct GuiMockModel {
    config: ModelConfig,
}

impl GuiMockModel {
    fn new() -> Self {
        Self {
            config: ModelConfig {
                model: "gui-test-model".to_string(),
                provider: ModelProvider::Custom,
                api_key: None,
                endpoint: None,
                temperature: 0.7,
                max_tokens: Some(2000),
                timeout_secs: 60,
                headers: HashMap::new(),
                extra: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Model for GuiMockModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        let last_msg = request.messages.last().unwrap();
        let response_text = format!("GUI response to: {}", last_msg.content);

        Ok(ModelResponse {
            content: response_text,
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 75,
                output_tokens: 25,
            },
            metadata: HashMap::new(),
        })
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
        unimplemented!("Streaming not needed for GUI test")
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// GUI command handler (simulates Tauri invoke handler)
async fn handle_gui_command(
    command: GuiCommand,
    executor: &AgentExecutor,
) -> GuiResponse {
    match command {
        GuiCommand::Execute { agent_name, input, config: _ } => {
            let mut context = AgentContext::new(input);

            match executor.execute(&mut context).await {
                Ok(response) => {
                    let mut gui_response = GuiResponse::success(serde_json::json!({
                        "agent": agent_name,
                        "response": response,
                    }));

                    // Add execution metadata
                    gui_response = gui_response
                        .with_metadata("input_tokens", serde_json::json!(context.metadata.input_tokens))
                        .with_metadata("output_tokens", serde_json::json!(context.metadata.output_tokens))
                        .with_metadata("execution_time_ms", serde_json::json!(context.metadata.execution_time_ms))
                        .with_metadata("tool_calls", serde_json::json!(context.metadata.tool_calls));

                    gui_response
                }
                Err(e) => GuiResponse::error(format!("Execution failed: {}", e)),
            }
        }

        GuiCommand::Status { agent_name } => {
            GuiResponse::success(serde_json::json!({
                "agent": agent_name,
                "status": "ready",
                "model": executor.model().config().model,
            }))
        }

        GuiCommand::ListAgents => {
            GuiResponse::success(serde_json::json!({
                "agents": vec![executor.config().name.clone()],
            }))
        }

        GuiCommand::GetHistory { session_id } => {
            GuiResponse::success(serde_json::json!({
                "session_id": session_id,
                "history": vec![] as Vec<String>,
            }))
        }
    }
}

#[tokio::test]
async fn test_gui_execute_command() {
    let config = AgentConfig {
        name: "gui-agent".to_string(),
        system_prompt: Some("You are a GUI assistant.".to_string()),
        model: "gui-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(2000),
        extra: HashMap::new(),
    };

    let model = Box::new(GuiMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let command = GuiCommand::Execute {
        agent_name: "gui-agent".to_string(),
        input: "Hello from GUI!".to_string(),
        config: None,
    };

    let response = handle_gui_command(command, &executor).await;

    assert!(response.success);
    assert!(response.data["response"].as_str().unwrap().contains("Hello from GUI!"));
    assert!(response.metadata.contains_key("input_tokens"));
    assert!(response.metadata.contains_key("execution_time_ms"));
}

#[tokio::test]
async fn test_gui_status_command() {
    let config = AgentConfig {
        name: "status-agent".to_string(),
        system_prompt: None,
        model: "gui-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(GuiMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let command = GuiCommand::Status {
        agent_name: "status-agent".to_string(),
    };

    let response = handle_gui_command(command, &executor).await;

    assert!(response.success);
    assert_eq!(response.data["agent"], "status-agent");
    assert_eq!(response.data["status"], "ready");
    assert_eq!(response.data["model"], "gui-test-model");
}

#[tokio::test]
async fn test_gui_list_agents_command() {
    let config = AgentConfig {
        name: "list-agent".to_string(),
        system_prompt: None,
        model: "gui-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(GuiMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let command = GuiCommand::ListAgents;

    let response = handle_gui_command(command, &executor).await;

    assert!(response.success);
    assert!(response.data["agents"].is_array());
    assert_eq!(response.data["agents"][0], "list-agent");
}

#[tokio::test]
async fn test_gui_get_history_command() {
    let config = AgentConfig {
        name: "history-agent".to_string(),
        system_prompt: None,
        model: "gui-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(GuiMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let command = GuiCommand::GetHistory {
        session_id: "session_123".to_string(),
    };

    let response = handle_gui_command(command, &executor).await;

    assert!(response.success);
    assert_eq!(response.data["session_id"], "session_123");
    assert!(response.data["history"].is_array());
}

#[tokio::test]
async fn test_gui_command_serialization() {
    let execute_cmd = GuiCommand::Execute {
        agent_name: "test".to_string(),
        input: "test input".to_string(),
        config: None,
    };

    let json = serde_json::to_string(&execute_cmd).unwrap();
    let deserialized: GuiCommand = serde_json::from_str(&json).unwrap();

    match deserialized {
        GuiCommand::Execute { agent_name, input, .. } => {
            assert_eq!(agent_name, "test");
            assert_eq!(input, "test input");
        }
        _ => panic!("Wrong command type"),
    }
}

#[tokio::test]
async fn test_gui_response_with_metadata() {
    let response = GuiResponse::success(serde_json::json!({"result": "ok"}))
        .with_metadata("execution_time", serde_json::json!(123))
        .with_metadata("tokens_used", serde_json::json!(50));

    assert!(response.success);
    assert_eq!(response.data["result"], "ok");
    assert_eq!(response.metadata["execution_time"], 123);
    assert_eq!(response.metadata["tokens_used"], 50);
}
