use crate::llm::config::LlmConfig;
use crate::llm::core::{LlmProvider, ModelInfo};
use crate::llm::error::{LlmError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing LLM providers and parsing model strings
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    model_map: HashMap<String, String>, // model_id -> provider
    config: LlmConfig,
}

impl ProviderRegistry {
    /// Create a new provider registry with configuration
    pub fn new(config: LlmConfig) -> Self {
        Self {
            providers: HashMap::new(),
            model_map: HashMap::new(),
            config,
        }
    }

    /// Register a provider
    pub fn register(&mut self, provider: Arc<dyn LlmProvider>) {
        let provider_name = provider.provider_name().to_string();
        let model_id = provider.model_info().model_id.clone();

        self.model_map.insert(model_id, provider_name.clone());
        self.providers.insert(provider_name, provider);
    }

    /// Parse model string in format "provider:model_id" or just "model_id"
    /// Examples:
    /// - "openai:gpt-4o" -> (openai, gpt-4o)
    /// - "anthropic:claude-sonnet-4-5" -> (anthropic, claude-sonnet-4-5)
    /// - "gpt-4o" -> (openai, gpt-4o) // auto-detect
    pub fn parse_model_string(&self, model_string: &str) -> Result<(String, String)> {
        if model_string.is_empty() {
            return Err(LlmError::InvalidModelString(
                "Empty model string".to_string(),
            ));
        }

        // Check if format is "provider:model"
        if let Some((provider, model)) = model_string.split_once(':') {
            return Ok((provider.to_string(), model.to_string()));
        }

        // Auto-detect provider from model name
        let provider = self.detect_provider(model_string)?;
        Ok((provider, model_string.to_string()))
    }

    /// Auto-detect provider from model name
    fn detect_provider(&self, model_id: &str) -> Result<String> {
        // Check model map first
        if let Some(provider) = self.model_map.get(model_id) {
            return Ok(provider.clone());
        }

        // Pattern matching for common models
        if model_id.starts_with("gpt-") || model_id.starts_with("o1") || model_id.starts_with("o3") {
            return Ok("openai".to_string());
        }

        if model_id.starts_with("claude-") {
            return Ok("anthropic".to_string());
        }

        if model_id.starts_with("gemini-") {
            return Ok("google".to_string());
        }

        if model_id.starts_with("grok-") {
            return Ok("grok".to_string());
        }

        // Check if it's registered with Ollama
        if self.providers.contains_key("ollama") {
            return Ok("ollama".to_string());
        }

        // Fallback to default provider
        Ok(self.config.default_provider.clone())
    }

    /// Get provider by name
    pub fn get_provider(&self, provider_name: &str) -> Result<Arc<dyn LlmProvider>> {
        self.providers
            .get(provider_name)
            .cloned()
            .ok_or_else(|| LlmError::ProviderNotFound(provider_name.to_string()))
    }

    /// Get provider for a model string
    pub fn get_provider_for_model(&self, model_string: &str) -> Result<Arc<dyn LlmProvider>> {
        let (provider_name, _model_id) = self.parse_model_string(model_string)?;
        self.get_provider(&provider_name)
    }

    /// List all registered providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// List all available models
    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.providers
            .values()
            .map(|p| p.model_info().clone())
            .collect()
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, provider_name: &str) -> bool {
        self.providers.contains_key(provider_name)
    }
}

/// Builder for ProviderRegistry
pub struct RegistryBuilder {
    config: LlmConfig,
    providers: Vec<Arc<dyn LlmProvider>>,
}

impl RegistryBuilder {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            providers: Vec::new(),
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn LlmProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn build(self) -> ProviderRegistry {
        let mut registry = ProviderRegistry::new(self.config);
        for provider in self.providers {
            registry.register(provider);
        }
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_string() {
        let config = LlmConfig::default();
        let registry = ProviderRegistry::new(config);

        // Test explicit provider:model format
        let (provider, model) = registry.parse_model_string("openai:gpt-4o").unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4o");

        let (provider, model) = registry
            .parse_model_string("anthropic:claude-sonnet-4-5")
            .unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-sonnet-4-5");

        // Test auto-detection
        let (provider, model) = registry.parse_model_string("gpt-4o").unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4o");

        let (provider, model) = registry.parse_model_string("claude-3-opus-20240229").unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-opus-20240229");

        let (provider, model) = registry.parse_model_string("gemini-2.0-flash").unwrap();
        assert_eq!(provider, "google");
        assert_eq!(model, "gemini-2.0-flash");

        let (provider, model) = registry.parse_model_string("grok-2").unwrap();
        assert_eq!(provider, "grok");
        assert_eq!(model, "grok-2");
    }

    #[test]
    fn test_empty_model_string() {
        let config = LlmConfig::default();
        let registry = ProviderRegistry::new(config);

        let result = registry.parse_model_string("");
        assert!(result.is_err());
    }
}
