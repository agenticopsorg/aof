use crate::llm::config::OpenAiConfig;
use crate::llm::core::*;
use crate::llm::error::{LlmError, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

/// OpenAI provider implementation
pub struct OpenAiProvider {
    client: Client,
    config: OpenAiConfig,
    model_info: ModelInfo,
}

impl OpenAiProvider {
    pub fn new(config: OpenAiConfig, model_id: Option<String>) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| LlmError::ConfigurationError("OpenAI API key not found".to_string()))?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", api_key)
                .parse()
                .map_err(|e| LlmError::ConfigurationError(format!("Invalid API key: {}", e)))?,
        );

        if let Some(org_id) = &config.organization_id {
            headers.insert(
                "OpenAI-Organization",
                org_id
                    .parse()
                    .map_err(|e| LlmError::ConfigurationError(format!("Invalid org ID: {}", e)))?,
            );
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .default_headers(headers)
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
        match model_id {
            "gpt-4o" => ModelInfo {
                provider: "openai".to_string(),
                model_id: model_id.to_string(),
                display_name: "GPT-4o".to_string(),
                context_window: 128_000,
                max_output_tokens: 16_384,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: true,
                cost_per_1k_input_tokens: 0.0025,
                cost_per_1k_output_tokens: 0.01,
            },
            "gpt-4o-mini" => ModelInfo {
                provider: "openai".to_string(),
                model_id: model_id.to_string(),
                display_name: "GPT-4o Mini".to_string(),
                context_window: 128_000,
                max_output_tokens: 16_384,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: true,
                cost_per_1k_input_tokens: 0.00015,
                cost_per_1k_output_tokens: 0.0006,
            },
            "o1" | "o1-preview" => ModelInfo {
                provider: "openai".to_string(),
                model_id: model_id.to_string(),
                display_name: "o1".to_string(),
                context_window: 128_000,
                max_output_tokens: 32_768,
                supports_tools: false,
                supports_vision: true,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.015,
                cost_per_1k_output_tokens: 0.06,
            },
            "o1-mini" => ModelInfo {
                provider: "openai".to_string(),
                model_id: model_id.to_string(),
                display_name: "o1-mini".to_string(),
                context_window: 128_000,
                max_output_tokens: 65_536,
                supports_tools: false,
                supports_vision: true,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.003,
                cost_per_1k_output_tokens: 0.012,
            },
            _ => ModelInfo {
                provider: "openai".to_string(),
                model_id: model_id.to_string(),
                display_name: model_id.to_string(),
                context_window: 8_192,
                max_output_tokens: 4_096,
                supports_tools: false,
                supports_vision: false,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.0,
                cost_per_1k_output_tokens: 0.0,
            },
        }
    }

    fn convert_request(&self, request: &ChatRequest) -> OpenAiRequest {
        OpenAiRequest {
            model: self.model_info.model_id.clone(),
            messages: request.messages.clone(),
            temperature: request.temperature.or(Some(self.config.temperature)),
            max_tokens: request.max_tokens.or(self.config.max_tokens),
            top_p: request.top_p,
            stop: request.stop_sequences.clone(),
            tools: request.tools.clone(),
            stream: request.stream,
        }
    }

    fn convert_response(&self, response: OpenAiResponse) -> ChatResponse {
        ChatResponse {
            id: response.id,
            model: response.model,
            choices: response.choices,
            usage: response.usage,
            created_at: Some(response.created),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let openai_request = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status {
                StatusCode::UNAUTHORIZED => {
                    LlmError::AuthenticationError("Invalid API key".to_string())
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    LlmError::RateLimitError("Rate limit exceeded".to_string())
                }
                _ => LlmError::ApiError {
                    status: status.as_u16(),
                    message: error_body,
                },
            });
        }

        let openai_response: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        Ok(self.convert_response(openai_response))
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let mut openai_request = self.convert_request(&request);
        openai_request.stream = Some(true);

        let response = self
            .client
            .post(&url)
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

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

        let stream = response.bytes_stream();
        let converted_stream = stream.map(|result| {
            result
                .map_err(|e| LlmError::StreamingError(e.to_string()))
                .and_then(|bytes| {
                    let text = String::from_utf8_lossy(&bytes);
                    // Parse SSE format: "data: {json}\n\n"
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                continue;
                            }
                            let chunk: OpenAiStreamChunk = serde_json::from_str(data)
                                .map_err(|e| LlmError::SerializationError(e.to_string()))?;
                            return Ok(ChatChunk {
                                id: chunk.id,
                                model: chunk.model,
                                delta: chunk.choices[0].delta.clone(),
                                finish_reason: chunk.choices[0].finish_reason,
                            });
                        }
                    }
                    Err(LlmError::StreamingError("Invalid SSE format".to_string()))
                })
        });

        Ok(Box::pin(converted_stream))
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
        "openai"
    }
}

// OpenAI API types
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
    created: i64,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    id: String,
    model: String,
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
    finish_reason: Option<FinishReason>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        let info = OpenAiProvider::get_model_info("gpt-4o");
        assert_eq!(info.model_id, "gpt-4o");
        assert_eq!(info.provider, "openai");
        assert!(info.supports_tools);
        assert!(info.supports_vision);
    }

    #[test]
    fn test_convert_request() {
        let config = OpenAiConfig {
            api_key: Some("test-key".to_string()),
            base_url: "https://api.openai.com/v1".to_string(),
            organization_id: None,
            default_model: "gpt-4o".to_string(),
            temperature: 0.7,
            max_tokens: None,
        };

        let provider = OpenAiProvider::new(config, None).unwrap();
        let request = ChatRequest::new(vec![Message::user("Hello")])
            .with_temperature(0.5)
            .with_max_tokens(100);

        let openai_request = provider.convert_request(&request);
        assert_eq!(openai_request.model, "gpt-4o");
        assert_eq!(openai_request.temperature, Some(0.5));
        assert_eq!(openai_request.max_tokens, Some(100));
    }
}
