pub mod config;
pub mod core;
pub mod error;
pub mod providers;
pub mod registry;
pub mod tokens;

pub use config::{LlmConfig, ProviderConfigs};
pub use core::{
    ChatChunk, ChatRequest, ChatResponse, Choice, ContentPart, Delta, FinishReason,
    FunctionCall, FunctionDefinition, ImageUrl, LlmProvider, Message, MessageContent,
    ModelInfo, Role, Tool, ToolCall, Usage,
};
pub use error::{LlmError, Result};
pub use providers::{AnthropicProvider, GoogleProvider, OllamaProvider, OpenAiProvider};
pub use registry::{ProviderRegistry, RegistryBuilder};
pub use tokens::{TokenCounter, get_token_counter};

/// Create a provider registry with common providers
pub async fn create_default_registry(config: LlmConfig) -> Result<ProviderRegistry> {
    let mut builder = RegistryBuilder::new(config.clone());

    // Register OpenAI if configured
    if let Some(openai_config) = config.providers.openai {
        if let Ok(provider) = OpenAiProvider::new(openai_config.clone(), None) {
            builder = builder.with_provider(std::sync::Arc::new(provider));
        }
    }

    // Register Anthropic if configured
    if let Some(anthropic_config) = config.providers.anthropic {
        if let Ok(provider) = AnthropicProvider::new(anthropic_config.clone(), None) {
            builder = builder.with_provider(std::sync::Arc::new(provider));
        }
    }

    // Register Google if configured
    if let Some(google_config) = config.providers.google {
        if let Ok(provider) = GoogleProvider::new(google_config.clone(), None) {
            builder = builder.with_provider(std::sync::Arc::new(provider));
        }
    }

    // Register Ollama if configured
    if let Some(ollama_config) = config.providers.ollama {
        if let Ok(provider) = OllamaProvider::new(ollama_config.clone(), None) {
            builder = builder.with_provider(std::sync::Arc::new(provider));
        }
    }

    Ok(builder.build())
}

/// Convenience function to create a chat request
pub fn chat(messages: Vec<Message>) -> ChatRequest {
    ChatRequest::new(messages)
}

/// Convenience function to create a simple text message
pub fn message(role: Role, content: impl Into<String>) -> Message {
    Message {
        role,
        content: MessageContent::Text(content.into()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}
