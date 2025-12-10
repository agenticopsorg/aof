//! Integration test for parallel tool execution
//!
//! This test validates that the AOF runtime can execute multiple tools
//! concurrently, improving performance significantly.

use aof_core::{
    AgentConfig, AgentContext, ModelConfig, ModelProvider, ModelRequest, ModelResponse,
    StopReason, ToolCall, ToolDefinition, ToolExecutor, ToolInput, ToolResult, Usage,
};
use aof_runtime::AgentExecutor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Mock model that returns tool calls
struct MockToolModel {
    tool_count: usize,
}

#[async_trait]
impl aof_core::Model for MockToolModel {
    async fn generate(&self, _request: &ModelRequest) -> aof_core::AofResult<ModelResponse> {
        // First call: request tools
        // Second call: end turn
        Ok(ModelResponse {
            content: "Using tools...".to_string(),
            tool_calls: (0..self.tool_count)
                .map(|i| ToolCall {
                    id: format!("call_{}", i),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({ "index": i }),
                })
                .collect(),
            stop_reason: StopReason::ToolUse,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
            metadata: HashMap::new(),
        })
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> aof_core::AofResult<
        std::pin::Pin<Box<dyn futures::Stream<Item = aof_core::AofResult<aof_core::StreamChunk>> + Send>>,
    > {
        unimplemented!()
    }

    fn config(&self) -> &ModelConfig {
        unimplemented!()
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Mock tool executor that simulates async work
struct MockParallelToolExecutor {
    execution_log: Arc<Mutex<Vec<(String, Instant)>>>,
    delay_ms: u64,
}

impl MockParallelToolExecutor {
    fn new(delay_ms: u64) -> Self {
        Self {
            execution_log: Arc::new(Mutex::new(Vec::new())),
            delay_ms,
        }
    }

    fn get_execution_log(&self) -> Vec<(String, Instant)> {
        self.execution_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl ToolExecutor for MockParallelToolExecutor {
    async fn execute_tool(
        &self,
        name: &str,
        input: ToolInput,
    ) -> aof_core::AofResult<ToolResult> {
        let start = Instant::now();

        // Log execution start
        {
            let mut log = self.execution_log.lock().unwrap();
            log.push((format!("start:{}", name), start));
        }

        // Simulate async work
        sleep(Duration::from_millis(self.delay_ms)).await;

        let end = Instant::now();

        // Log execution end
        {
            let mut log = self.execution_log.lock().unwrap();
            log.push((format!("end:{}", name), end));
        }

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "tool": name,
                "input": input.arguments,
                "duration_ms": (end - start).as_millis()
            }),
            error: None,
            execution_time_ms: (end - start).as_millis() as u64,
        })
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool that simulates async work".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": { "type": "number" }
                }
            }),
        }]
    }

    fn get_tool(&self, name: &str) -> Option<ToolDefinition> {
        if name == "test_tool" {
            Some(self.list_tools()[0].clone())
        } else {
            None
        }
    }
}

#[tokio::test]
async fn test_parallel_tool_execution_performance() {
    // Test with 5 tools, each taking 100ms
    // Sequential: ~500ms
    // Parallel: ~100ms (with concurrency)

    let tool_count = 5;
    let delay_ms = 100;

    let executor = Arc::new(MockParallelToolExecutor::new(delay_ms));
    let model = Box::new(MockToolModel { tool_count });

    let config = AgentConfig {
        name: "parallel-test".to_string(),
        system_prompt: None,
        model: "test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 2,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let agent = AgentExecutor::new(config, model, Some(executor.clone()), None);

    let mut context = AgentContext::new("Execute tools in parallel");

    let start = Instant::now();
    let result = agent.execute(&mut context).await;
    let total_duration = start.elapsed();

    assert!(result.is_ok(), "Execution should succeed");

    // Verify tool calls were executed
    assert_eq!(context.metadata.tool_calls, tool_count);

    // Check execution log for parallelism
    let log = executor.get_execution_log();

    // Count how many tools were running concurrently
    let mut max_concurrent = 0;
    let mut current_concurrent = 0;

    for (event, _time) in log.iter() {
        if event.starts_with("start:") {
            current_concurrent += 1;
            max_concurrent = max_concurrent.max(current_concurrent);
        } else if event.starts_with("end:") {
            current_concurrent -= 1;
        }
    }

    println!("Total duration: {:?}", total_duration);
    println!("Max concurrent tools: {}", max_concurrent);
    println!("Tool calls: {}", context.metadata.tool_calls);

    // With parallel execution, should complete much faster than sequential
    // Sequential would be: tool_count * delay_ms = 500ms
    // Parallel should be close to: delay_ms = 100ms (plus overhead)
    let expected_sequential_ms = (tool_count as u64 * delay_ms) as u128;
    let expected_parallel_ms = delay_ms as u128;

    // Assert parallel execution is significantly faster
    assert!(
        total_duration.as_millis() < expected_sequential_ms - 100,
        "Parallel execution should be faster than sequential. Got {}ms, expected < {}ms",
        total_duration.as_millis(),
        expected_sequential_ms - 100
    );

    // Verify tools ran concurrently
    assert!(
        max_concurrent > 1,
        "Should have concurrent tool execution, got max_concurrent = {}",
        max_concurrent
    );
}

#[tokio::test]
async fn test_single_tool_no_parallel_overhead() {
    // Single tool should execute directly without parallel machinery

    let executor = Arc::new(MockParallelToolExecutor::new(50));
    let model = Box::new(MockToolModel { tool_count: 1 });

    let config = AgentConfig {
        name: "single-tool-test".to_string(),
        system_prompt: None,
        model: "test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 2,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let agent = AgentExecutor::new(config, model, Some(executor), None);

    let mut context = AgentContext::new("Execute single tool");
    let result = agent.execute(&mut context).await;

    assert!(result.is_ok());
    assert_eq!(context.metadata.tool_calls, 1);
}

#[tokio::test]
async fn test_parallel_tool_with_failures() {
    // Test that partial failures are handled gracefully

    struct FailingToolExecutor;

    #[async_trait]
    impl ToolExecutor for FailingToolExecutor {
        async fn execute_tool(
            &self,
            name: &str,
            input: ToolInput,
        ) -> aof_core::AofResult<ToolResult> {
            let index: usize = input.arguments["index"].as_u64().unwrap() as usize;

            // Fail every other tool
            if index % 2 == 0 {
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({ "index": index }),
                    error: None,
                    execution_time_ms: 0,
                })
            } else {
                Err(aof_core::AofError::tool(format!("Tool {} failed", index)))
            }
        }

        fn list_tools(&self) -> Vec<ToolDefinition> {
            vec![ToolDefinition {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({}),
            }]
        }

        fn get_tool(&self, _name: &str) -> Option<ToolDefinition> {
            Some(self.list_tools()[0].clone())
        }
    }

    let executor = Arc::new(FailingToolExecutor);
    let model = Box::new(MockToolModel { tool_count: 4 });

    let config = AgentConfig {
        name: "failing-tools-test".to_string(),
        system_prompt: None,
        model: "test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 2,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let agent = AgentExecutor::new(config, model, Some(executor), None);

    let mut context = AgentContext::new("Test partial failures");
    let result = agent.execute(&mut context).await;

    // Should still succeed overall
    assert!(result.is_ok());

    // Should have attempted all tools
    assert_eq!(context.metadata.tool_calls, 4);

    // Check tool results - should have both successes and failures
    let success_count = context.tool_results.iter().filter(|r| r.success).count();
    let failure_count = context.tool_results.iter().filter(|r| !r.success).count();

    assert_eq!(success_count, 2, "Should have 2 successful tools");
    assert_eq!(failure_count, 2, "Should have 2 failed tools");
}
