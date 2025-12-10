//! Integration test: Streaming response flow

use aof_core::{
    AgentConfig, AgentContext, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, StopReason, StreamChunk, Usage,
};
use aof_runtime::executor::{AgentExecutor, StreamEvent};
use async_trait::async_trait;
use futures::stream;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::sync::mpsc;

/// Mock streaming model
struct StreamingMockModel {
    config: ModelConfig,
}

impl StreamingMockModel {
    fn new() -> Self {
        Self {
            config: ModelConfig {
                model: "streaming-test".to_string(),
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
impl Model for StreamingMockModel {
    async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
        Ok(ModelResponse {
            content: "Non-streaming response".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
            },
            metadata: HashMap::new(),
        })
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
        // Create a stream of chunks
        let chunks = vec![
            Ok(StreamChunk::ContentDelta {
                delta: "Hello ".to_string(),
            }),
            Ok(StreamChunk::ContentDelta {
                delta: "from ".to_string(),
            }),
            Ok(StreamChunk::ContentDelta {
                delta: "streaming!".to_string(),
            }),
            Ok(StreamChunk::Done {
                usage: Usage {
                    input_tokens: 20,
                    output_tokens: 10,
                },
                stop_reason: StopReason::EndTurn,
            }),
        ];

        Ok(Box::pin(stream::iter(chunks)))
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

#[tokio::test]
async fn test_streaming_basic_flow() {
    let config = AgentConfig {
        name: "streaming-agent".to_string(),
        system_prompt: Some("You are a test assistant".to_string()),
        model: "streaming-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(1000),
        extra: HashMap::new(),
    };

    let model = Box::new(StreamingMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn task to collect events
    let event_handle = tokio::spawn(async move {
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        events
    });

    let mut context = AgentContext::new("Test streaming");
    let response = executor.execute_streaming(&mut context, tx).await.unwrap();

    // Wait for event collection
    let events = event_handle.await.unwrap();

    // Verify response
    assert_eq!(response, "Hello from streaming!");

    // Verify events
    let mut has_iteration_start = false;
    let mut has_text_delta = false;
    let mut has_done = false;

    for event in &events {
        match event {
            StreamEvent::IterationStart { .. } => has_iteration_start = true,
            StreamEvent::TextDelta { .. } => has_text_delta = true,
            StreamEvent::Done { .. } => has_done = true,
            _ => {}
        }
    }

    assert!(has_iteration_start, "Should have iteration start event");
    assert!(has_text_delta, "Should have text delta events");
    assert!(has_done, "Should have done event");
}

#[tokio::test]
async fn test_streaming_text_deltas() {
    let config = AgentConfig {
        name: "delta-test-agent".to_string(),
        system_prompt: None,
        model: "streaming-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(StreamingMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn task to collect deltas
    let delta_handle = tokio::spawn(async move {
        let mut deltas = Vec::new();
        while let Some(event) = rx.recv().await {
            if let StreamEvent::TextDelta { delta, .. } = event {
                deltas.push(delta);
            }
        }
        deltas
    });

    let mut context = AgentContext::new("Stream text");
    let _ = executor.execute_streaming(&mut context, tx).await.unwrap();

    let deltas = delta_handle.await.unwrap();

    // Should receive multiple deltas
    assert!(deltas.len() >= 3);
    assert_eq!(deltas.join(""), "Hello from streaming!");
}

#[tokio::test]
async fn test_streaming_done_event() {
    let config = AgentConfig {
        name: "done-test-agent".to_string(),
        system_prompt: None,
        model: "streaming-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(StreamingMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn task to find done event
    let done_handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let StreamEvent::Done {
                content,
                total_iterations,
                execution_time_ms,
                input_tokens,
                output_tokens,
            } = event
            {
                return Some((
                    content,
                    total_iterations,
                    execution_time_ms,
                    input_tokens,
                    output_tokens,
                ));
            }
        }
        None
    });

    let mut context = AgentContext::new("Test done event");
    let _ = executor.execute_streaming(&mut context, tx).await.unwrap();

    let done_event = done_handle.await.unwrap();
    assert!(done_event.is_some());

    let (content, iterations, exec_time, input_tokens, output_tokens) = done_event.unwrap();

    assert_eq!(content, "Hello from streaming!");
    assert_eq!(iterations, 1);
    assert!(exec_time > 0);
    assert_eq!(input_tokens, 20);
    assert_eq!(output_tokens, 10);
}

#[tokio::test]
async fn test_streaming_iteration_events() {
    let config = AgentConfig {
        name: "iteration-test-agent".to_string(),
        system_prompt: None,
        model: "streaming-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(StreamingMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn task to collect iteration events
    let iter_handle = tokio::spawn(async move {
        let mut start_count = 0;
        let mut complete_count = 0;

        while let Some(event) = rx.recv().await {
            match event {
                StreamEvent::IterationStart { .. } => start_count += 1,
                StreamEvent::IterationComplete { .. } => complete_count += 1,
                _ => {}
            }
        }

        (start_count, complete_count)
    });

    let mut context = AgentContext::new("Test iterations");
    let _ = executor.execute_streaming(&mut context, tx).await.unwrap();

    let (start_count, complete_count) = iter_handle.await.unwrap();

    // Should have matching start/complete events
    assert_eq!(start_count, 1);
    assert_eq!(complete_count, 1);
}
