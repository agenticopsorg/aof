//! Integration test: Multi-tool parallel execution
//! This test was created based on the existing parallel_tools_test.rs specification

use aof_core::{
    AgentConfig, AgentContext, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, StopReason, StreamChunk, ToolCall, ToolDefinition, ToolExecutor, ToolInput,
    ToolResult, Usage,
};
use aof_runtime::AgentExecutor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Mock model that requests parallel tool execution
struct ParallelToolMockModel {
    config: ModelConfig,
}

impl ParallelToolMockModel {
    fn new() -> Self {
        Self {
            config: ModelConfig {
                model: "parallel-test-model".to_string(),
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
impl Model for ParallelToolMockModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        // Check if this is the first request (no tool results yet)
        let has_tool_results = request.messages.iter().any(|m| {
            m.role == aof_core::model::MessageRole::Tool
        });

        if !has_tool_results {
            // First call: request multiple tools in parallel
            Ok(ModelResponse {
                content: "I'll execute multiple tools in parallel".to_string(),
                tool_calls: vec![
                    ToolCall {
                        id: "tool_1".to_string(),
                        name: "slow_tool".to_string(),
                        arguments: serde_json::json!({"delay_ms": 100}),
                    },
                    ToolCall {
                        id: "tool_2".to_string(),
                        name: "fast_tool".to_string(),
                        arguments: serde_json::json!({}),
                    },
                    ToolCall {
                        id: "tool_3".to_string(),
                        name: "medium_tool".to_string(),
                        arguments: serde_json::json!({"delay_ms": 50}),
                    },
                    ToolCall {
                        id: "tool_4".to_string(),
                        name: "fast_tool".to_string(),
                        arguments: serde_json::json!({}),
                    },
                    ToolCall {
                        id: "tool_5".to_string(),
                        name: "slow_tool".to_string(),
                        arguments: serde_json::json!({"delay_ms": 150}),
                    },
                ],
                stop_reason: StopReason::ToolUse,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 20,
                },
                metadata: HashMap::new(),
            })
        } else {
            // Second call: return final response after tools executed
            Ok(ModelResponse {
                content: "All tools executed successfully in parallel!".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 200,
                    output_tokens: 40,
                },
                metadata: HashMap::new(),
            })
        }
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
        unimplemented!("Streaming not needed for parallel tool test")
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Mock tool executor that simulates different execution speeds
struct ParallelToolExecutor {
    execution_log: Arc<Mutex<Vec<(String, Instant)>>>,
}

impl ParallelToolExecutor {
    fn new() -> Self {
        Self {
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_execution_log(&self) -> Vec<(String, Instant)> {
        self.execution_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl ToolExecutor for ParallelToolExecutor {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult> {
        let start = Instant::now();

        // Log execution start
        {
            let mut log = self.execution_log.lock().unwrap();
            log.push((name.to_string(), start));
        }

        // Simulate different execution times
        let delay_ms = match name {
            "fast_tool" => 10,
            "medium_tool" => {
                input.get_arg::<u64>("delay_ms").unwrap_or(50)
            }
            "slow_tool" => {
                input.get_arg::<u64>("delay_ms").unwrap_or(100)
            }
            _ => 50,
        };

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(ToolResult::success(serde_json::json!({
            "tool": name,
            "delay_ms": delay_ms,
            "actual_ms": elapsed
        })).with_execution_time(elapsed))
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "fast_tool".to_string(),
                description: "A fast tool".to_string(),
                parameters: serde_json::json!({}),
            },
            ToolDefinition {
                name: "medium_tool".to_string(),
                description: "A medium speed tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "delay_ms": {"type": "number"}
                    }
                }),
            },
            ToolDefinition {
                name: "slow_tool".to_string(),
                description: "A slow tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "delay_ms": {"type": "number"}
                    }
                }),
            },
        ]
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn aof_core::Tool>> {
        None
    }
}

#[tokio::test]
async fn test_multi_tool_parallel_execution() {
    let config = AgentConfig {
        name: "parallel-tool-agent".to_string(),
        system_prompt: Some("You are a parallel execution test agent.".to_string()),
        model: "parallel-test".to_string(),
        tools: vec!["fast_tool".to_string(), "medium_tool".to_string(), "slow_tool".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(2000),
        extra: HashMap::new(),
    };

    let model = Box::new(ParallelToolMockModel::new());
    let tool_executor = Arc::new(ParallelToolExecutor::new());
    let executor = AgentExecutor::new(config, model, Some(tool_executor.clone()), None);

    let start_time = Instant::now();
    let mut context = AgentContext::new("Execute tools in parallel");
    let response = executor.execute(&mut context).await.unwrap();
    let total_time = start_time.elapsed();

    assert_eq!(response, "All tools executed successfully in parallel!");
    assert_eq!(context.metadata.tool_calls, 5);
    assert_eq!(context.tool_results.len(), 5);

    // Verify all tools succeeded
    for result in &context.tool_results {
        assert!(result.success);
    }

    // Parallel execution should be faster than sequential
    // Sequential would be: 100 + 10 + 50 + 10 + 150 = 320ms minimum
    // Parallel should complete in ~150ms (slowest tool)
    // Add some buffer for test reliability
    assert!(
        total_time.as_millis() < 250,
        "Parallel execution took {}ms, expected < 250ms",
        total_time.as_millis()
    );
}

#[tokio::test]
async fn test_tool_execution_ordering() {
    let config = AgentConfig {
        name: "order-test-agent".to_string(),
        system_prompt: None,
        model: "parallel-test".to_string(),
        tools: vec!["fast_tool".to_string(), "medium_tool".to_string(), "slow_tool".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(ParallelToolMockModel::new());
    let tool_executor = Arc::new(ParallelToolExecutor::new());
    let executor = AgentExecutor::new(config, model, Some(tool_executor.clone()), None);

    let mut context = AgentContext::new("Test ordering");
    let _ = executor.execute(&mut context).await.unwrap();

    let execution_log = tool_executor.get_execution_log();

    // All tools should start execution within a short time window (parallel)
    if execution_log.len() >= 2 {
        let first_start = execution_log[0].1;
        let last_start = execution_log[execution_log.len() - 1].1;
        let span = last_start.duration_since(first_start);

        // All tools should start within 50ms of each other (parallel execution)
        assert!(
            span.as_millis() < 100,
            "Tools should start in parallel, but span was {}ms",
            span.as_millis()
        );
    }
}

#[tokio::test]
async fn test_parallel_tool_result_preservation() {
    let config = AgentConfig {
        name: "result-test-agent".to_string(),
        system_prompt: None,
        model: "parallel-test".to_string(),
        tools: vec!["fast_tool".to_string(), "slow_tool".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(ParallelToolMockModel::new());
    let tool_executor = Arc::new(ParallelToolExecutor::new());
    let executor = AgentExecutor::new(config, model, Some(tool_executor), None);

    let mut context = AgentContext::new("Test result preservation");
    let _ = executor.execute(&mut context).await.unwrap();

    // Verify results are in correct order (matching tool call order)
    assert_eq!(context.tool_results.len(), 5);

    // Results should be in the order tools were called
    let tool_names: Vec<String> = context
        .tool_results
        .iter()
        .map(|r| r.tool_name.clone())
        .collect();

    assert_eq!(tool_names[0], "slow_tool");
    assert_eq!(tool_names[1], "fast_tool");
    assert_eq!(tool_names[2], "medium_tool");
    assert_eq!(tool_names[3], "fast_tool");
    assert_eq!(tool_names[4], "slow_tool");
}

#[tokio::test]
async fn test_parallel_execution_with_many_tools() {
    // Test with maximum parallel tools (10)
    let config = AgentConfig {
        name: "many-tools-agent".to_string(),
        system_prompt: None,
        model: "parallel-test".to_string(),
        tools: vec!["fast_tool".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    // Create a custom model that requests exactly 10 tools
    struct ManyToolsModel {
        config: ModelConfig,
    }

    #[async_trait]
    impl Model for ManyToolsModel {
        async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
            let has_tool_results = request.messages.iter().any(|m| {
                m.role == aof_core::model::MessageRole::Tool
            });

            if !has_tool_results {
                // Request 10 tools in parallel
                let tool_calls: Vec<ToolCall> = (0..10)
                    .map(|i| ToolCall {
                        id: format!("tool_{}", i),
                        name: "fast_tool".to_string(),
                        arguments: serde_json::json!({}),
                    })
                    .collect();

                Ok(ModelResponse {
                    content: "Executing 10 tools".to_string(),
                    tool_calls,
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                    metadata: HashMap::new(),
                })
            } else {
                Ok(ModelResponse {
                    content: "All 10 tools completed".to_string(),
                    tool_calls: vec![],
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                    metadata: HashMap::new(),
                })
            }
        }

        async fn generate_stream(
            &self,
            _request: &ModelRequest,
        ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
            unimplemented!()
        }

        fn config(&self) -> &ModelConfig {
            &self.config
        }

        fn provider(&self) -> ModelProvider {
            ModelProvider::Custom
        }
    }

    let model = Box::new(ManyToolsModel {
        config: ModelConfig {
            model: "many-tools-test".to_string(),
            provider: ModelProvider::Custom,
            api_key: None,
            endpoint: None,
            temperature: 0.7,
            max_tokens: None,
            timeout_secs: 60,
            headers: HashMap::new(),
            extra: HashMap::new(),
        },
    });

    let tool_executor = Arc::new(ParallelToolExecutor::new());
    let executor = AgentExecutor::new(config, model, Some(tool_executor), None);

    let mut context = AgentContext::new("Execute 10 tools");
    let response = executor.execute(&mut context).await.unwrap();

    assert_eq!(response, "All 10 tools completed");
    assert_eq!(context.metadata.tool_calls, 10);
    assert_eq!(context.tool_results.len(), 10);

    // Verify all succeeded
    for result in &context.tool_results {
        assert!(result.success);
    }
}
