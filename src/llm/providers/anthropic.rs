use crate::llm::config::AnthropicConfig;
use crate::llm::core::*;
use crate::llm::error::{LlmError, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

/// Anthropic provider implementation
pub struct AnthropicProvider {
    client: Client,
    config: AnthropicConfig,
    model_info: ModelInfo,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig, model_id: Option<String>) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| {
                LlmError::ConfigurationError("Anthropic API key not found".to_string())
            })?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            api_key
                .parse()
                .map_err(|e| LlmError::ConfigurationError(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(
            "anthropic-version",
            "2023-06-01"
                .parse()
                .map_err(|e| LlmError::ConfigurationError(format!("Invalid version: {}", e)))?,
        );

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
            "claude-3-5-sonnet-20241022" | "claude-sonnet-4-5" => ModelInfo {
                provider: "anthropic".to_string(),
                model_id: model_id.to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                context_window: 200_000,
                max_output_tokens: 8_192,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.003,
                cost_per_1k_output_tokens: 0.015,
            },
            "claude-3-opus-20240229" => ModelInfo {
                provider: "anthropic".to_string(),
                model_id: model_id.to_string(),
                display_name: "Claude 3 Opus".to_string(),
                context_window: 200_000,
                max_output_tokens: 4_096,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.015,
                cost_per_1k_output_tokens: 0.075,
            },
            "claude-3-haiku-20240307" => ModelInfo {
                provider: "anthropic".to_string(),
                model_id: model_id.to_string(),
                display_name: "Claude 3 Haiku".to_string(),
                context_window: 200_000,
                max_output_tokens: 4_096,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.00025,
                cost_per_1k_output_tokens: 0.00125,
            },
            _ => ModelInfo {
                provider: "anthropic".to_string(),
                model_id: model_id.to_string(),
                display_name: model_id.to_string(),
                context_window: 200_000,
                max_output_tokens: 4_096,
                supports_tools: false,
                supports_vision: false,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.0,
                cost_per_1k_output_tokens: 0.0,
            },
        }
    }

    fn convert_request(&self, request: &ChatRequest) -> AnthropicRequest {
        // Separate system message from other messages
        let mut system = None;
        let mut messages = Vec::new();

        for msg in &request.messages {
            if msg.role == Role::System {
                if let MessageContent::Text(text) = &msg.content {
                    system = Some(text.clone());
                }
            } else {
                messages.push(msg.clone());
            }
        }

        AnthropicRequest {
            model: self.model_info.model_id.clone(),
            messages,
            system,
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            temperature: request.temperature.or(Some(self.config.temperature)),
            top_p: request.top_p,
            stop_sequences: request.stop_sequences.clone(),
            tools: request.tools.as_ref().map(|tools| {
                tools
                    .iter()
                    .map(|t| AnthropicTool {
                        name: t.function.name.clone(),
                        description: t.function.description.clone(),
                        input_schema: t.function.parameters.clone(),
                    })
                    .collect()
            }),
            stream: request.stream,
        }
    }

    fn convert_response(&self, response: AnthropicResponse) -> ChatResponse {
        let content = response
            .content
            .iter()
            .filter_map(|c| match c {
                AnthropicContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let tool_calls = response
            .content
            .iter()
            .filter_map(|c| match c {
                AnthropicContent::ToolUse { id, name, input } => Some(ToolCall {
                    id: id.clone(),
                    tool_type: "function".to_string(),
                    function: FunctionCall {
                        name: name.clone(),
                        arguments: serde_json::to_string(input).unwrap_or_default(),
                    },
                }),
                _ => None,
            })
            .collect::<Vec<_>>();

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        };

        ChatResponse {
            id: response.id,
            model: response.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: MessageContent::Text(content),
                    name: None,
                    tool_calls: if tool_calls.is_empty() {
                        None
                    } else {
                        Some(tool_calls)
                    },
                    tool_call_id: None,
                },
                finish_reason,
            }],
            usage: Usage::new(response.usage.input_tokens, response.usage.output_tokens),
            created_at: None,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/messages", self.config.base_url);
        let anthropic_request = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .json(&anthropic_request)
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

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        Ok(self.convert_response(anthropic_response))
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>> {
        let url = format!("{}/messages", self.config.base_url);
        let mut anthropic_request = self.convert_request(&request);
        anthropic_request.stream = Some(true);

        let response = self
            .client
            .post(&url)
            .json(&anthropic_request)
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
                    // Parse SSE format
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let event: AnthropicStreamEvent = serde_json::from_str(data)
                                .map_err(|e| LlmError::SerializationError(e.to_string()))?;

                            if let AnthropicStreamEvent::ContentBlockDelta { delta, .. } = event {
                                if let AnthropicDelta::TextDelta { text } = delta {
                                    return Ok(ChatChunk {
                                        id: "stream".to_string(),
                                        model: self.model_info.model_id.clone(),
                                        delta: Delta {
                                            role: None,
                                            content: Some(text),
                                            tool_calls: None,
                                        },
                                        finish_reason: None,
                                    });
                                }
                            }
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
        "anthropic"
    }
}

// Anthropic API types
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContent {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicStreamEvent {
    MessageStart,
    ContentBlockStart,
    ContentBlockDelta { index: usize, delta: AnthropicDelta },
    ContentBlockStop,
    MessageDelta,
    MessageStop,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicDelta {
    TextDelta { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        let info = AnthropicProvider::get_model_info("claude-3-5-sonnet-20241022");
        assert_eq!(info.model_id, "claude-3-5-sonnet-20241022");
        assert_eq!(info.provider, "anthropic");
        assert!(info.supports_tools);
        assert!(info.supports_vision);
    }
}
