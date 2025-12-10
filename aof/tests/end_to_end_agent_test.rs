//! Integration test: End-to-end agent execution

use aof_core::{
    AgentConfig, AgentContext, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, RequestMessage, StopReason, StreamChunk, ToolCall, ToolDefinition,
    ToolExecutor, ToolInput, ToolResult, Usage,
};
use aof_runtime::AgentExecutor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

/// Complete mock model implementation for testing
struct IntegrationMockModel {
    config: ModelConfig,
}

impl IntegrationMockModel {
    fn new() -> Self {
        Self {
            config: ModelConfig {
                model: "integration-test-model".to_string(),
                provider: ModelProvider::Custom,
                api_key: Some("test-key".to_string()),
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
impl Model for IntegrationMockModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        // Simulate real model behavior based on request
        let last_message = request.messages.last().unwrap();

        // If user asks for tool usage, respond with tool call
        if last_message.content.contains("use tool") || last_message.content.contains("calculate") {
            return Ok(ModelResponse {
                content: "I'll use the calculator tool for this".to_string(),
                tool_calls: vec![ToolCall {
                    id: "calc_1".to_string(),
                    name: "calculator".to_string(),
                    arguments: serde_json::json!({
                        "operation": "add",
                        "a": 5,
                        "b": 3
                    }),
                }],
                stop_reason: StopReason::ToolUse,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 20,
                },
                metadata: HashMap::new(),
            });
        }

        // Otherwise, return normal completion
        Ok(ModelResponse {
            content: "I understand your request and have processed it successfully.".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 150,
                output_tokens: 30,
            },
            metadata: HashMap::new(),
        })
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
        unimplemented!("Streaming not needed for this test")
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Mock tool executor with calculator
struct IntegrationToolExecutor;

#[async_trait]
impl ToolExecutor for IntegrationToolExecutor {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult> {
        match name {
            "calculator" => {
                let operation: String = input.get_arg("operation")?;
                let a: i32 = input.get_arg("a")?;
                let b: i32 = input.get_arg("b")?;

                let result = match operation.as_str() {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0 {
                            return Ok(ToolResult::error("Division by zero"));
                        }
                        a / b
                    }
                    _ => return Ok(ToolResult::error("Unknown operation")),
                };

                Ok(ToolResult::success(serde_json::json!({
                    "result": result
                })).with_execution_time(10))
            }
            _ => Ok(ToolResult::error(format!("Unknown tool: {}", name))),
        }
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "calculator".to_string(),
            description: "Perform basic arithmetic operations".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"]
                    },
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["operation", "a", "b"]
            }),
        }]
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn aof_core::Tool>> {
        None
    }
}

#[tokio::test]
async fn test_end_to_end_simple_query() {
    let config = AgentConfig {
        name: "integration-test-agent".to_string(),
        system_prompt: Some("You are a helpful AI assistant for testing.".to_string()),
        model: "test-model".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(2000),
        extra: HashMap::new(),
    };

    let model = Box::new(IntegrationMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let mut context = AgentContext::new("Hello, how are you?");
    let response = executor.execute(&mut context).await.unwrap();

    assert!(!response.is_empty());
    assert!(context.metadata.input_tokens > 0);
    assert!(context.metadata.output_tokens > 0);
    assert_eq!(context.messages.len(), 2); // User + Assistant
}

#[tokio::test]
async fn test_end_to_end_with_tool_usage() {
    let config = AgentConfig {
        name: "tool-test-agent".to_string(),
        system_prompt: Some("You are a calculator assistant.".to_string()),
        model: "test-model".to_string(),
        tools: vec!["calculator".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(2000),
        extra: HashMap::new(),
    };

    let model = Box::new(IntegrationMockModel::new());
    let tool_executor = Arc::new(IntegrationToolExecutor);
    let executor = AgentExecutor::new(config, model, Some(tool_executor), None);

    let mut context = AgentContext::new("Please calculate 5 + 3");
    let response = executor.execute(&mut context).await.unwrap();

    assert!(!response.is_empty());
    assert_eq!(context.metadata.tool_calls, 1);
    assert_eq!(context.tool_results.len(), 1);
    assert!(context.tool_results[0].success);
    assert_eq!(context.tool_results[0].result, serde_json::json!({"result": 8}));
}

#[tokio::test]
async fn test_end_to_end_multi_turn_conversation() {
    let config = AgentConfig {
        name: "conversation-agent".to_string(),
        system_prompt: Some("You are a helpful assistant.".to_string()),
        model: "test-model".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(2000),
        extra: HashMap::new(),
    };

    let model = Box::new(IntegrationMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    // First turn
    let mut context = AgentContext::new("Hello!");
    let response1 = executor.execute(&mut context).await.unwrap();
    assert!(!response1.is_empty());

    // Second turn (context carries over)
    context.input = "Tell me more".to_string();
    context.add_message(aof_core::MessageRole::User, "Tell me more");

    let response2 = executor.execute(&mut context).await.unwrap();
    assert!(!response2.is_empty());

    // Should have multiple messages in history
    assert!(context.messages.len() >= 4); // User1 + Assistant1 + User2 + Assistant2
}

#[tokio::test]
async fn test_end_to_end_token_tracking() {
    let config = AgentConfig {
        name: "token-tracking-agent".to_string(),
        system_prompt: Some("You are a test assistant.".to_string()),
        model: "test-model".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(2000),
        extra: HashMap::new(),
    };

    let model = Box::new(IntegrationMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let mut context = AgentContext::new("Test input");

    // Execute
    let _ = executor.execute(&mut context).await.unwrap();

    // Verify token tracking
    assert!(context.metadata.input_tokens > 0);
    assert!(context.metadata.output_tokens > 0);
    assert!(context.metadata.execution_time_ms > 0);
    assert!(context.metadata.model.is_some());
}

#[tokio::test]
async fn test_end_to_end_context_state() {
    let mut context = AgentContext::new("Test");

    // Set various state values
    context.set_state("user_id", "12345").unwrap();
    context.set_state("session_id", "session_abc").unwrap();
    context.set_state("count", 42i32).unwrap();

    // Retrieve and verify
    let user_id: Option<String> = context.get_state("user_id");
    assert_eq!(user_id, Some("12345".to_string()));

    let session_id: Option<String> = context.get_state("session_id");
    assert_eq!(session_id, Some("session_abc".to_string()));

    let count: Option<i32> = context.get_state("count");
    assert_eq!(count, Some(42));

    // Non-existent key
    let missing: Option<String> = context.get_state("missing");
    assert!(missing.is_none());
}
