use crate::llm::core::{Message, MessageContent};

/// Token counting utilities for different providers
pub trait TokenCounter: Send + Sync {
    /// Count tokens in a string
    fn count_tokens(&self, text: &str) -> usize;

    /// Count tokens in messages
    fn count_message_tokens(&self, messages: &[Message]) -> usize {
        messages
            .iter()
            .map(|msg| self.count_message_token(msg))
            .sum()
    }

    /// Count tokens in a single message
    fn count_message_token(&self, message: &Message) -> usize {
        let mut total = 0;

        // Add overhead for message formatting (role, etc.)
        total += 4;

        match &message.content {
            MessageContent::Text(text) => {
                total += self.count_tokens(text);
            }
            MessageContent::Parts(parts) => {
                for part in parts {
                    match part {
                        crate::llm::core::ContentPart::Text { text } => {
                            total += self.count_tokens(text);
                        }
                        crate::llm::core::ContentPart::ImageUrl { .. } => {
                            // Images typically cost ~85 tokens for low detail, ~255 for high
                            total += 170; // Conservative estimate
                        }
                    }
                }
            }
        }

        if let Some(tool_calls) = &message.tool_calls {
            for tool_call in tool_calls {
                total += self.count_tokens(&tool_call.function.name);
                total += self.count_tokens(&tool_call.function.arguments);
                total += 10; // Overhead for tool call formatting
            }
        }

        total
    }
}

/// Simple token counter based on word/character estimation
pub struct SimpleTokenCounter;

impl TokenCounter for SimpleTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        // Rough estimation: ~4 characters per token
        // More accurate for English text
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();

        // Average of character-based and word-based estimation
        ((char_count / 4) + (word_count * 4 / 3)) / 2
    }
}

/// OpenAI-specific token counter
/// In production, this would use the tiktoken library
pub struct OpenAiTokenCounter {
    model: String,
}

impl OpenAiTokenCounter {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

impl TokenCounter for OpenAiTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        // Placeholder: In production, use tiktoken-rs
        // For now, use character-based estimation
        let char_count = text.chars().count();
        char_count / 4
    }
}

/// Anthropic-specific token counter
pub struct AnthropicTokenCounter;

impl TokenCounter for AnthropicTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        // Anthropic's tokenizer is similar to GPT-4
        // Character-based estimation is reasonable
        let char_count = text.chars().count();
        char_count / 4
    }
}

/// Google-specific token counter
pub struct GoogleTokenCounter;

impl TokenCounter for GoogleTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        // Google's tokenizer varies by model
        // Conservative estimation
        let char_count = text.chars().count();
        char_count / 4
    }
}

/// Get appropriate token counter for a provider
pub fn get_token_counter(provider: &str, model: Option<&str>) -> Box<dyn TokenCounter> {
    match provider {
        "openai" => Box::new(OpenAiTokenCounter::new(model.unwrap_or("gpt-4"))),
        "anthropic" => Box::new(AnthropicTokenCounter),
        "google" => Box::new(GoogleTokenCounter),
        _ => Box::new(SimpleTokenCounter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::core::Message;

    #[test]
    fn test_simple_token_counter() {
        let counter = SimpleTokenCounter;

        let text = "Hello, world!";
        let tokens = counter.count_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < 10); // Should be around 3-4 tokens

        let long_text = "The quick brown fox jumps over the lazy dog.";
        let long_tokens = counter.count_tokens(long_text);
        assert!(long_tokens > tokens);
    }

    #[test]
    fn test_message_token_counting() {
        let counter = SimpleTokenCounter;

        let message = Message::user("Hello, how are you?");
        let tokens = counter.count_message_token(&message);
        assert!(tokens > 4); // Should be > overhead

        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("Hello!"),
            Message::assistant("Hi there!"),
        ];

        let total = counter.count_message_tokens(&messages);
        assert!(total > 12); // Should be > 3 * overhead
    }

    #[test]
    fn test_get_token_counter() {
        let openai_counter = get_token_counter("openai", Some("gpt-4"));
        let tokens = openai_counter.count_tokens("Hello world");
        assert!(tokens > 0);

        let anthropic_counter = get_token_counter("anthropic", None);
        let tokens = anthropic_counter.count_tokens("Hello world");
        assert!(tokens > 0);

        let unknown_counter = get_token_counter("unknown", None);
        let tokens = unknown_counter.count_tokens("Hello world");
        assert!(tokens > 0);
    }
}
