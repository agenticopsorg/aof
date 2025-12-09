//! AgentExecutor - Manages agent execution lifecycle
//!
//! The AgentExecutor handles the core execution loop:
//! 1. Call model with messages + tools
//! 2. If stop_reason == ToolUse, execute tools
//! 3. Add tool results to context
//! 4. Repeat until EndTurn or max iterations

use aof_core::{
    AgentConfig, AgentContext, AofError, AofResult, MessageRole, Model, ModelRequest,
    ModelToolDefinition, RequestMessage, StopReason, ToolCall, ToolExecutor, ToolInput, ToolResult,
};
use aof_memory::SimpleMemory;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Agent executor that manages the execution lifecycle
///
/// This is the core execution engine that orchestrates the interaction
/// between the model, tools, and memory to accomplish agent tasks.
pub struct AgentExecutor {
    /// Agent configuration
    config: AgentConfig,

    /// LLM model
    model: Box<dyn Model>,

    /// Tool executor (optional)
    tool_executor: Option<Arc<dyn ToolExecutor>>,

    /// Memory backend (optional)
    memory: Option<Arc<SimpleMemory>>,
}

impl AgentExecutor {
    /// Create a new agent executor
    pub fn new(
        config: AgentConfig,
        model: Box<dyn Model>,
        tool_executor: Option<Arc<dyn ToolExecutor>>,
        memory: Option<Arc<SimpleMemory>>,
    ) -> Self {
        Self {
            config,
            model,
            tool_executor,
            memory,
        }
    }

    /// Execute the agent with the given context
    ///
    /// This runs the main execution loop:
    /// 1. Build model request from context
    /// 2. Call model.generate()
    /// 3. Handle response (execute tools if needed)
    /// 4. Repeat until done or max iterations
    pub async fn execute(&self, context: &mut AgentContext) -> AofResult<String> {
        info!("Starting agent execution: {}", self.config.name);
        let execution_start = Instant::now();

        // Add user message if not already in history
        if context.messages.is_empty() {
            context.add_message(MessageRole::User, context.input.clone());
        }

        let mut iteration = 0;
        let max_iterations = self.config.max_iterations;

        loop {
            iteration += 1;

            if iteration > max_iterations {
                warn!(
                    "Reached max iterations ({}) for agent: {}",
                    max_iterations, self.config.name
                );
                return Err(AofError::agent(format!(
                    "Exceeded max iterations ({})",
                    max_iterations
                )));
            }

            debug!(
                "Agent iteration {}/{} for: {}",
                iteration, max_iterations, self.config.name
            );

            // Build model request
            let request = self.build_model_request(context)?;

            // Call model
            let response = self
                .model
                .generate(&request)
                .await
                .map_err(|e| AofError::agent(format!("Model generation failed: {}", e)))?;

            // Update usage statistics
            context.metadata.input_tokens += response.usage.input_tokens;
            context.metadata.output_tokens += response.usage.output_tokens;
            context.metadata.model = Some(self.model.config().model.clone());

            debug!(
                "Model response - stop_reason: {:?}, content length: {}, tool_calls: {}",
                response.stop_reason,
                response.content.len(),
                response.tool_calls.len()
            );

            // Add assistant message to history
            let mut assistant_msg = aof_core::Message {
                role: MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: None,
            };

            if !response.tool_calls.is_empty() {
                assistant_msg.tool_calls = Some(response.tool_calls.clone());
            }

            context.messages.push(assistant_msg);

            // Handle stop reason
            match response.stop_reason {
                StopReason::EndTurn => {
                    info!(
                        "Agent execution completed in {} iterations",
                        iteration
                    );
                    context.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    return Ok(response.content);
                }

                StopReason::ToolUse => {
                    // Execute tools
                    debug!("Executing {} tool calls", response.tool_calls.len());
                    let tool_results = self.execute_tools(&response.tool_calls).await?;

                    context.metadata.tool_calls += tool_results.len();

                    // Add tool results to context
                    for (tool_call, result) in response.tool_calls.iter().zip(tool_results.iter()) {
                        // Convert tool result to agent tool result
                        let agent_result = aof_core::AgentToolResult {
                            tool_name: tool_call.name.clone(),
                            result: result.data.clone(),
                            success: result.success,
                            error: result.error.clone(),
                        };
                        context.tool_results.push(agent_result);

                        // Add tool result message to history
                        let tool_msg = aof_core::Message {
                            role: MessageRole::Tool,
                            content: serde_json::to_string(&result.data)
                                .unwrap_or_else(|_| "{}".to_string()),
                            tool_calls: None,
                        };
                        context.messages.push(tool_msg);
                    }

                    // Continue loop for next iteration
                    continue;
                }

                StopReason::MaxTokens => {
                    warn!("Model reached max tokens");
                    context.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    return Ok(response.content);
                }

                StopReason::StopSequence => {
                    info!("Model hit stop sequence");
                    context.metadata.execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    return Ok(response.content);
                }

                StopReason::ContentFilter => {
                    error!("Content filter triggered");
                    return Err(AofError::agent(
                        "Content filter triggered by model".to_string(),
                    ));
                }
            }
        }
    }

    /// Build a model request from the current context
    fn build_model_request(&self, context: &AgentContext) -> AofResult<ModelRequest> {
        // Convert context messages to request messages
        let messages: Vec<RequestMessage> = context
            .messages
            .iter()
            .map(|m| RequestMessage {
                role: match m.role {
                    MessageRole::User => aof_core::model::MessageRole::User,
                    MessageRole::Assistant => aof_core::model::MessageRole::Assistant,
                    MessageRole::System => aof_core::model::MessageRole::System,
                    MessageRole::Tool => aof_core::model::MessageRole::Tool,
                },
                content: m.content.clone(),
                tool_calls: m.tool_calls.clone(),
            })
            .collect();

        // Get tool definitions if available
        let tools: Vec<ModelToolDefinition> = if let Some(executor) = &self.tool_executor {
            executor
                .list_tools()
                .into_iter()
                .map(|t| ModelToolDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(ModelRequest {
            messages,
            system: self.config.system_prompt.clone(),
            tools,
            temperature: Some(self.config.temperature),
            max_tokens: self.config.max_tokens,
            stream: false,
            extra: std::collections::HashMap::new(),
        })
    }

    /// Execute a list of tool calls
    async fn execute_tools(&self, tool_calls: &[ToolCall]) -> AofResult<Vec<ToolResult>> {
        let executor = self
            .tool_executor
            .as_ref()
            .ok_or_else(|| AofError::tool("No tool executor available".to_string()))?;

        let mut results = Vec::new();

        for tool_call in tool_calls {
            debug!("Executing tool: {}", tool_call.name);

            let input = ToolInput::new(tool_call.arguments.clone());

            let result = executor
                .execute_tool(&tool_call.name, input)
                .await
                .unwrap_or_else(|e| {
                    error!("Tool execution failed for {}: {}", tool_call.name, e);
                    ToolResult {
                        success: false,
                        data: serde_json::Value::Null,
                        error: Some(e.to_string()),
                        execution_time_ms: 0,
                    }
                });

            results.push(result);
        }

        Ok(results)
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get model reference
    pub fn model(&self) -> &dyn Model {
        self.model.as_ref()
    }

    /// Get tool executor reference
    pub fn tool_executor(&self) -> Option<&Arc<dyn ToolExecutor>> {
        self.tool_executor.as_ref()
    }

    /// Get memory reference
    pub fn memory(&self) -> Option<&Arc<SimpleMemory>> {
        self.memory.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::{ModelConfig, ModelProvider, ModelResponse, StopReason, Usage};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock model for testing
    struct MockModel {
        responses: Vec<ModelResponse>,
        current: std::sync::Mutex<usize>,
        config: ModelConfig,
    }

    impl MockModel {
        fn new(responses: Vec<ModelResponse>) -> Self {
            Self {
                responses,
                current: std::sync::Mutex::new(0),
                config: ModelConfig {
                    model: "mock-model".to_string(),
                    provider: ModelProvider::Custom,
                    api_key: None,
                    endpoint: None,
                    temperature: 0.7,
                    max_tokens: None,
                    timeout_secs: 60,
                    headers: HashMap::new(),
                    extra: HashMap::new(),
                },
            }
        }
    }

    #[async_trait]
    impl Model for MockModel {
        async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
            let mut current = self.current.lock().unwrap();
            let idx = *current;
            *current += 1;

            if idx < self.responses.len() {
                Ok(self.responses[idx].clone())
            } else {
                Ok(ModelResponse {
                    content: "Done".to_string(),
                    tool_calls: vec![],
                    stop_reason: StopReason::EndTurn,
                    usage: Usage {
                        input_tokens: 100,
                        output_tokens: 50,
                    },
                    metadata: HashMap::new(),
                })
            }
        }

        async fn generate_stream(
            &self,
            _request: &ModelRequest,
        ) -> AofResult<std::pin::Pin<Box<dyn futures::Stream<Item = AofResult<aof_core::StreamChunk>> + Send>>>
        {
            unimplemented!()
        }

        fn config(&self) -> &ModelConfig {
            &self.config
        }

        fn provider(&self) -> ModelProvider {
            ModelProvider::Custom
        }
    }

    #[tokio::test]
    async fn test_agent_executor_simple() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: Some("You are a helpful assistant".to_string()),
            model: "test-model".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: Some(1000),
            extra: HashMap::new(),
        };

        let model = Box::new(MockModel::new(vec![ModelResponse {
            content: "Hello! How can I help?".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
            metadata: HashMap::new(),
        }]));

        let executor = AgentExecutor::new(config, model, None, None);

        let mut context = AgentContext::new("Hello");
        let response = executor.execute(&mut context).await.unwrap();

        assert_eq!(response, "Hello! How can I help?");
        assert_eq!(context.metadata.input_tokens, 100);
        assert_eq!(context.metadata.output_tokens, 50);
    }

    #[tokio::test]
    async fn test_agent_executor_max_iterations() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "test-model".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 2,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        // Mock model that keeps requesting tools
        let model = Box::new(MockModel::new(vec![
            ModelResponse {
                content: "Calling tool...".to_string(),
                tool_calls: vec![ToolCall {
                    id: "1".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }],
                stop_reason: StopReason::ToolUse,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                metadata: HashMap::new(),
            },
            ModelResponse {
                content: "Calling tool again...".to_string(),
                tool_calls: vec![ToolCall {
                    id: "2".to_string(),
                    name: "test_tool".to_string(),
                    arguments: serde_json::json!({}),
                }],
                stop_reason: StopReason::ToolUse,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                metadata: HashMap::new(),
            },
            ModelResponse {
                content: "Should not reach here".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 50,
                },
                metadata: HashMap::new(),
            },
        ]));

        let executor = AgentExecutor::new(config, model, None, None);

        let mut context = AgentContext::new("Test");
        let result = executor.execute(&mut context).await;

        // Should fail due to max iterations, but we have no tool executor
        // so it will fail on tool execution first
        assert!(result.is_err());
    }
}
