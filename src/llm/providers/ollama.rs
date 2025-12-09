use crate::llm::config::OllamaConfig;
use crate::llm::core::*;
use crate::llm::error::{LlmError, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

/// Ollama provider implementation for local models
pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
    model_info: ModelInfo,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig, model_id: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // Longer timeout for local inference
            .build()
            .map_err(|e| LlmError::ConfigurationError(format!("Failed to build client: {}", e)))?;

        let model = model_id.unwrap_or_else(|| config.default_model.clone());
        let model_info = Self::get_model_info(&model);

        Ok(Self {
            client,
            config,
            model_info,
        })
    }

    fn get_model_info(model_id: &str) -> ModelInfo {
        // Local models - no cost, context window varies
        ModelInfo {
            provider: "ollama".to_string(),
            model_id: model_id.to_string(),
            display_name: model_id.to_string(),
            context_window: 8_192, // Default, can vary
            max_output_tokens: 4_096,
            supports_tools: model_id.contains("llama3") || model_id.contains("mistral"),
            supports_vision: model_id.contains("llava") || model_id.contains("bakllava"),
            supports_structured_output: false,
            cost_per_1k_input_tokens: 0.0,
            cost_per_1k_output_tokens: 0.0,
        }
    }

    fn convert_request(&self, request: &ChatRequest) -> OllamaRequest {
        OllamaRequest {
            model: self.model_info.model_id.clone(),
            messages: request.messages.clone(),
            options: Some(OllamaOptions {
                temperature: request.temperature.or(Some(self.config.temperature)),
                num_predict: request.max_tokens,
                top_p: request.top_p,
                stop: request.stop_sequences.clone(),
            }),
            stream: request.stream.unwrap_or(false),
        }
    }

    fn convert_response(&self, response: OllamaResponse) -> ChatResponse {
        let finish_reason = if response.done {
            FinishReason::Stop
        } else {
            FinishReason::Length
        };

        ChatResponse {
            id: "ollama-response".to_string(),
            model: response.model,
            choices: vec![Choice {
                index: 0,
                message: response.message,
                finish_reason,
            }],
            usage: Usage::new(
                response.prompt_eval_count.unwrap_or(0),
                response.eval_count.unwrap_or(0),
            ),
            created_at: Some(response.created_at.parse().unwrap_or(0)),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.config.base_url);
        let ollama_request = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    LlmError::NetworkError(format!(
                        "Failed to connect to Ollama at {}. Is Ollama running?",
                        self.config.base_url
                    ))
                } else {
                    LlmError::NetworkError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError {
                status: response.status().as_u16(),
                message: error_body,
            });
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        Ok(self.convert_response(ollama_response))
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>> {
        // TODO: Implement streaming for Ollama
        Err(LlmError::UnsupportedFeature(
            "Streaming not yet implemented for Ollama".to_string(),
        ))
    }

    fn supports_tools(&self) -> bool {
        self.model_info.supports_tools
    }

    fn supports_vision(&self) -> bool {
        self.model_info.supports_vision
    }

    fn supports_structured_output(&self) -> bool {
        self.model_info.supports_structured_output
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

// Ollama API types
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    message: Message,
    done: bool,
    created_at: String,
    #[serde(rename = "prompt_eval_count")]
    prompt_eval_count: Option<usize>,
    #[serde(rename = "eval_count")]
    eval_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        let info = OllamaProvider::get_model_info("llama3");
        assert_eq!(info.model_id, "llama3");
        assert_eq!(info.provider, "ollama");
        assert_eq!(info.cost_per_1k_input_tokens, 0.0);
        assert_eq!(info.cost_per_1k_output_tokens, 0.0);
    }

    #[test]
    fn test_vision_model() {
        let info = OllamaProvider::get_model_info("llava");
        assert!(info.supports_vision);
    }
}
