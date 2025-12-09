use crate::llm::config::GoogleConfig;
use crate::llm::core::*;
use crate::llm::error::{LlmError, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

/// Google provider implementation
pub struct GoogleProvider {
    client: Client,
    config: GoogleConfig,
    model_info: ModelInfo,
}

impl GoogleProvider {
    pub fn new(config: GoogleConfig, model_id: Option<String>) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
            .ok_or_else(|| LlmError::ConfigurationError("Google API key not found".to_string()))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
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
            "gemini-2.0-flash-exp" | "gemini-2.0-flash" => ModelInfo {
                provider: "google".to_string(),
                model_id: model_id.to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
                context_window: 1_000_000,
                max_output_tokens: 8_192,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: true,
                cost_per_1k_input_tokens: 0.0,
                cost_per_1k_output_tokens: 0.0,
            },
            "gemini-1.5-pro" => ModelInfo {
                provider: "google".to_string(),
                model_id: model_id.to_string(),
                display_name: "Gemini 1.5 Pro".to_string(),
                context_window: 2_000_000,
                max_output_tokens: 8_192,
                supports_tools: true,
                supports_vision: true,
                supports_structured_output: true,
                cost_per_1k_input_tokens: 0.00125,
                cost_per_1k_output_tokens: 0.005,
            },
            _ => ModelInfo {
                provider: "google".to_string(),
                model_id: model_id.to_string(),
                display_name: model_id.to_string(),
                context_window: 32_000,
                max_output_tokens: 8_192,
                supports_tools: false,
                supports_vision: false,
                supports_structured_output: false,
                cost_per_1k_input_tokens: 0.0,
                cost_per_1k_output_tokens: 0.0,
            },
        }
    }

    fn convert_request(&self, request: &ChatRequest) -> GoogleRequest {
        let contents = request
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|msg| GoogleContent {
                role: match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "model".to_string(),
                    _ => "user".to_string(),
                },
                parts: match &msg.content {
                    MessageContent::Text(text) => vec![GooglePart::Text { text: text.clone() }],
                    MessageContent::Parts(parts) => parts
                        .iter()
                        .map(|p| match p {
                            ContentPart::Text { text } => GooglePart::Text { text: text.clone() },
                            ContentPart::ImageUrl { image_url } => GooglePart::InlineData {
                                inline_data: GoogleInlineData {
                                    mime_type: "image/jpeg".to_string(),
                                    data: image_url.url.clone(),
                                },
                            },
                        })
                        .collect(),
                },
            })
            .collect();

        let system_instruction = request
            .messages
            .iter()
            .find(|m| m.role == Role::System)
            .and_then(|msg| match &msg.content {
                MessageContent::Text(text) => Some(GoogleContent {
                    role: "user".to_string(),
                    parts: vec![GooglePart::Text { text: text.clone() }],
                }),
                _ => None,
            });

        GoogleRequest {
            contents,
            system_instruction,
            generation_config: Some(GoogleGenerationConfig {
                temperature: request.temperature.or(Some(self.config.temperature)),
                max_output_tokens: request.max_tokens,
                top_p: request.top_p,
                stop_sequences: request.stop_sequences.clone(),
            }),
            tools: request.tools.as_ref().map(|tools| {
                vec![GoogleToolSet {
                    function_declarations: tools
                        .iter()
                        .map(|t| GoogleFunctionDeclaration {
                            name: t.function.name.clone(),
                            description: t.function.description.clone(),
                            parameters: t.function.parameters.clone(),
                        })
                        .collect(),
                }]
            }),
        }
    }

    fn convert_response(&self, response: GoogleResponse) -> ChatResponse {
        let candidate = &response.candidates[0];
        let content_text = candidate
            .content
            .parts
            .iter()
            .filter_map(|p| match p {
                GooglePart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::Length,
            Some("SAFETY") => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        ChatResponse {
            id: "google-response".to_string(),
            model: self.model_info.model_id.clone(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: MessageContent::Text(content_text),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason,
            }],
            usage: Usage::new(
                response.usage_metadata.prompt_token_count,
                response.usage_metadata.candidates_token_count,
            ),
            created_at: None,
        }
    }

    fn get_api_key(&self) -> Result<String> {
        self.config
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
            .ok_or_else(|| LlmError::ConfigurationError("Google API key not found".to_string()))
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.base_url, self.model_info.model_id, api_key
        );

        let google_request = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .json(&google_request)
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

        let google_response: GoogleResponse = response
            .json()
            .await
            .map_err(|e| LlmError::SerializationError(e.to_string()))?;

        Ok(self.convert_response(google_response))
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>> {
        // TODO: Implement streaming for Google
        Err(LlmError::UnsupportedFeature(
            "Streaming not yet implemented for Google".to_string(),
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
        "google"
    }
}

// Google API types
#[derive(Debug, Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GoogleGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GoogleToolSet>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GooglePart {
    Text { text: String },
    InlineData { inline_data: GoogleInlineData },
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleInlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct GoogleGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct GoogleToolSet {
    function_declarations: Vec<GoogleFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct GoogleFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GoogleResponse {
    candidates: Vec<GoogleCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: GoogleUsageMetadata,
}

#[derive(Debug, Deserialize)]
struct GoogleCandidate {
    content: GoogleContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: usize,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        let info = GoogleProvider::get_model_info("gemini-2.0-flash-exp");
        assert_eq!(info.model_id, "gemini-2.0-flash-exp");
        assert_eq!(info.provider, "google");
        assert!(info.supports_tools);
        assert!(info.supports_vision);
    }
}
