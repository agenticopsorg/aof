use aof_core::{AofError, AofResult, Model, ModelConfig, ModelProvider};

pub mod anthropic;
pub mod openai;
#[cfg(feature = "bedrock")]
pub mod bedrock;
#[cfg(feature = "ollama")]
pub mod ollama;

/// LLM provider trait
pub trait LlmProvider {
    fn create(config: ModelConfig) -> AofResult<Box<dyn Model>>;
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    pub async fn create(config: ModelConfig) -> AofResult<Box<dyn Model>> {
        match config.provider {
            ModelProvider::Anthropic => anthropic::AnthropicProvider::create(config),
            ModelProvider::OpenAI => openai::OpenAIProvider::create(config),
            #[cfg(feature = "bedrock")]
            ModelProvider::Bedrock => bedrock::BedrockProvider::create(config).await,
            #[cfg(not(feature = "bedrock"))]
            ModelProvider::Bedrock => Err(AofError::config("Bedrock provider not enabled")),
            #[cfg(feature = "ollama")]
            ModelProvider::Ollama => ollama::OllamaProvider::create(config),
            #[cfg(not(feature = "ollama"))]
            ModelProvider::Ollama => Err(AofError::config("Ollama provider not enabled")),
            ModelProvider::Azure => Err(AofError::config("Azure provider not yet implemented")),
            ModelProvider::Custom => Err(AofError::config("Custom provider requires manual implementation")),
        }
    }
}
