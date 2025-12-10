//! Integration test: Platform message → Runtime → Response flow

use aof_core::{
    AgentConfig, AgentContext, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, StopReason, StreamChunk, Usage,
};
use aof_runtime::AgentExecutor;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

/// Platform message format (simulating Slack/Discord/etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlatformMessage {
    platform: String,
    user_id: String,
    channel_id: String,
    text: String,
    timestamp: u64,
}

/// Platform response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlatformResponse {
    platform: String,
    channel_id: String,
    text: String,
    status: String,
}

/// Mock model for platform testing
struct PlatformMockModel {
    config: ModelConfig,
}

impl PlatformMockModel {
    fn new() -> Self {
        Self {
            config: ModelConfig {
                model: "platform-test-model".to_string(),
                provider: ModelProvider::Custom,
                api_key: None,
                endpoint: None,
                temperature: 0.7,
                max_tokens: Some(1000),
                timeout_secs: 60,
                headers: HashMap::new(),
                extra: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Model for PlatformMockModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        // Extract last message
        let last_msg = request.messages.last().unwrap();
        let content = &last_msg.content;

        // Generate contextual response based on platform command
        let response_text = if content.contains("/help") {
            "Available commands: /help, /status, /info".to_string()
        } else if content.contains("/status") {
            "System status: All systems operational".to_string()
        } else if content.contains("/info") {
            "AOF Platform Integration Test Bot v1.0.0".to_string()
        } else {
            format!("I received your message: {}", content)
        };

        Ok(ModelResponse {
            content: response_text,
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 50,
                output_tokens: 20,
            },
            metadata: HashMap::new(),
        })
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
        unimplemented!("Streaming not needed for platform test")
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Simulates a platform handler that processes messages through the runtime
async fn handle_platform_message(
    message: PlatformMessage,
    executor: &AgentExecutor,
) -> AofResult<PlatformResponse> {
    // Create agent context from platform message
    let mut context = AgentContext::new(&message.text);

    // Add platform metadata to context state
    context.set_state("platform", &message.platform)?;
    context.set_state("user_id", &message.user_id)?;
    context.set_state("channel_id", &message.channel_id)?;
    context.set_state("timestamp", message.timestamp)?;

    // Execute agent
    let response_text = executor.execute(&mut context).await?;

    // Create platform response
    Ok(PlatformResponse {
        platform: message.platform.clone(),
        channel_id: message.channel_id.clone(),
        text: response_text,
        status: "success".to_string(),
    })
}

#[tokio::test]
async fn test_platform_message_to_response_flow() {
    let config = AgentConfig {
        name: "platform-bot".to_string(),
        system_prompt: Some("You are a helpful platform bot.".to_string()),
        model: "platform-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(1000),
        extra: HashMap::new(),
    };

    let model = Box::new(PlatformMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    // Simulate Slack message
    let slack_msg = PlatformMessage {
        platform: "slack".to_string(),
        user_id: "U12345".to_string(),
        channel_id: "C67890".to_string(),
        text: "Hello bot!".to_string(),
        timestamp: 1234567890,
    };

    let response = handle_platform_message(slack_msg, &executor).await.unwrap();

    assert_eq!(response.platform, "slack");
    assert_eq!(response.channel_id, "C67890");
    assert!(response.text.contains("I received your message"));
    assert_eq!(response.status, "success");
}

#[tokio::test]
async fn test_platform_command_help() {
    let config = AgentConfig {
        name: "command-bot".to_string(),
        system_prompt: Some("You are a command bot.".to_string()),
        model: "platform-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(1000),
        extra: HashMap::new(),
    };

    let model = Box::new(PlatformMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let msg = PlatformMessage {
        platform: "discord".to_string(),
        user_id: "user123".to_string(),
        channel_id: "channel456".to_string(),
        text: "/help".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    let response = handle_platform_message(msg, &executor).await.unwrap();

    assert!(response.text.contains("Available commands"));
    assert_eq!(response.platform, "discord");
}

#[tokio::test]
async fn test_platform_command_status() {
    let config = AgentConfig {
        name: "status-bot".to_string(),
        system_prompt: Some("You are a status bot.".to_string()),
        model: "platform-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(1000),
        extra: HashMap::new(),
    };

    let model = Box::new(PlatformMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let msg = PlatformMessage {
        platform: "telegram".to_string(),
        user_id: "telegram_user".to_string(),
        channel_id: "telegram_channel".to_string(),
        text: "/status".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    let response = handle_platform_message(msg, &executor).await.unwrap();

    assert!(response.text.contains("System status"));
    assert_eq!(response.platform, "telegram");
}

#[tokio::test]
async fn test_platform_metadata_preservation() {
    let config = AgentConfig {
        name: "metadata-bot".to_string(),
        system_prompt: None,
        model: "platform-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let model = Box::new(PlatformMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    let msg = PlatformMessage {
        platform: "whatsapp".to_string(),
        user_id: "whatsapp_user_789".to_string(),
        channel_id: "whatsapp_chat_abc".to_string(),
        text: "Test message".to_string(),
        timestamp: 9876543210,
    };

    // Create context and add metadata
    let mut context = AgentContext::new(&msg.text);
    context.set_state("platform", &msg.platform).unwrap();
    context.set_state("user_id", &msg.user_id).unwrap();
    context.set_state("channel_id", &msg.channel_id).unwrap();
    context.set_state("timestamp", msg.timestamp).unwrap();

    let _ = executor.execute(&mut context).await.unwrap();

    // Verify metadata is preserved in context
    let platform: Option<String> = context.get_state("platform");
    assert_eq!(platform, Some("whatsapp".to_string()));

    let user_id: Option<String> = context.get_state("user_id");
    assert_eq!(user_id, Some("whatsapp_user_789".to_string()));

    let timestamp: Option<u64> = context.get_state("timestamp");
    assert_eq!(timestamp, Some(9876543210));
}

#[tokio::test]
async fn test_platform_multi_message_flow() {
    let config = AgentConfig {
        name: "multi-message-bot".to_string(),
        system_prompt: Some("You are a conversational bot.".to_string()),
        model: "platform-test".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(1000),
        extra: HashMap::new(),
    };

    let model = Box::new(PlatformMockModel::new());
    let executor = AgentExecutor::new(config, model, None, None);

    // Simulate multiple messages from same user/channel
    let messages = vec![
        PlatformMessage {
            platform: "slack".to_string(),
            user_id: "U123".to_string(),
            channel_id: "C456".to_string(),
            text: "First message".to_string(),
            timestamp: 1000,
        },
        PlatformMessage {
            platform: "slack".to_string(),
            user_id: "U123".to_string(),
            channel_id: "C456".to_string(),
            text: "Second message".to_string(),
            timestamp: 2000,
        },
        PlatformMessage {
            platform: "slack".to_string(),
            user_id: "U123".to_string(),
            channel_id: "C456".to_string(),
            text: "/help".to_string(),
            timestamp: 3000,
        },
    ];

    let mut responses = Vec::new();

    for msg in messages {
        let response = handle_platform_message(msg, &executor).await.unwrap();
        responses.push(response);
    }

    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0].platform, "slack");
    assert_eq!(responses[1].platform, "slack");
    assert!(responses[2].text.contains("Available commands"));
}
