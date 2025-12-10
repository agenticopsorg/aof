// Memory integration methods for AgentExecutor
// These methods should be added to agent_executor.rs

use aof_core::{AgentContext, AofError, AofResult, MessageRole, MemoryEntry};
use aof_memory::SimpleMemory;
use std::sync::Arc;
use tracing::{debug, warn};

/// Restore conversation history from memory
pub async fn restore_conversation_history(
    agent_name: &str,
    context: &mut AgentContext,
    memory: &Arc<SimpleMemory>,
) -> AofResult<()> {
    let conversation_key = format!("agent:{}:conversation", agent_name);

    if let Some(history) = memory.retrieve::<Vec<aof_core::Message>>(&conversation_key).await? {
        debug!(
            "Restored {} messages from memory for agent: {}",
            history.len(),
            agent_name
        );

        // Prune history if it exceeds context window
        let pruned_history = prune_conversation_history(agent_name, history);
        context.messages = pruned_history;
    } else {
        debug!("No conversation history found for agent: {}", agent_name);
    }

    Ok(())
}

/// Store conversation turn in memory
pub async fn store_conversation_turn(
    agent_name: &str,
    context: &AgentContext,
    memory: &Arc<SimpleMemory>,
    iteration: usize,
) -> AofResult<()> {
    let conversation_key = format!("agent:{}:conversation", agent_name);
    let turn_key = format!("agent:{}:turn:{}", agent_name, iteration);

    // Store full conversation history
    let conversation_value = serde_json::to_value(&context.messages)
        .map_err(|e| AofError::memory(format!("Failed to serialize messages: {}", e)))?;

    memory.store(&conversation_key, conversation_value).await?;

    // Store individual turn with metadata for semantic search
    let turn_value = serde_json::json!({
        "iteration": iteration,
        "message_count": context.messages.len(),
        "input_tokens": context.metadata.input_tokens,
        "output_tokens": context.metadata.output_tokens,
        "tool_calls": context.metadata.tool_calls,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    memory.store(&turn_key, turn_value).await?;

    debug!(
        "Stored conversation turn {} for agent: {}",
        iteration, agent_name
    );

    Ok(())
}

/// Prune conversation history to fit context window
pub fn prune_conversation_history(
    agent_name: &str,
    mut history: Vec<aof_core::Message>,
) -> Vec<aof_core::Message> {
    // Simple pruning strategy: keep most recent messages
    // In production, this should use token counting and preserve important context
    const MAX_MESSAGES: usize = 100;

    if history.len() > MAX_MESSAGES {
        warn!(
            "Pruning conversation history from {} to {} messages for agent: {}",
            history.len(),
            MAX_MESSAGES,
            agent_name
        );

        // Keep system messages and most recent messages
        let system_messages: Vec<_> = history
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .cloned()
            .collect();

        // Take most recent non-system messages
        let recent_messages: Vec<_> = history
            .into_iter()
            .filter(|m| m.role != MessageRole::System)
            .rev()
            .take(MAX_MESSAGES - system_messages.len())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Combine system + recent
        history = system_messages;
        history.extend(recent_messages);
    }

    history
}

/// Cleanup expired memory entries
pub async fn cleanup_expired_memory(
    agent_name: &str,
    memory: &Arc<SimpleMemory>,
) -> AofResult<()> {
    let prefix = format!("agent:{}:", agent_name);

    // List all keys for this agent
    let keys = memory.list_keys().await?;
    let agent_keys: Vec<_> = keys.iter().filter(|k| k.starts_with(&prefix)).collect();

    debug!(
        "Checking {} memory entries for expiry for agent: {}",
        agent_keys.len(),
        agent_name
    );

    // Memory backend handles lazy cleanup on retrieve
    // This method is for manual cleanup if needed
    for key in agent_keys {
        // Retrieving will auto-delete if expired
        let _: Option<serde_json::Value> = memory.retrieve(key).await?;
    }

    Ok(())
}

/// Search memory for relevant context
pub async fn search_memory(
    agent_name: &str,
    query: &str,
    memory: &Arc<SimpleMemory>,
) -> AofResult<Vec<MemoryEntry>> {
    // For now, return all turns for this agent
    // In production, implement semantic search using embeddings
    let prefix = format!("agent:{}:turn:", agent_name);

    let keys = memory.list_keys().await?;
    let mut entries = Vec::new();

    for key in keys {
        if key.starts_with(&prefix) {
            if let Some(value) = memory.retrieve::<serde_json::Value>(&key).await? {
                entries.push(MemoryEntry::new(key, value));
            }
        }
    }

    debug!(
        "Found {} memory entries matching query '{}' for agent: {}",
        entries.len(),
        query,
        agent_name
    );

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_memory::InMemoryBackend;

    #[tokio::test]
    async fn test_memory_integration() {
        let backend = InMemoryBackend::new();
        let memory = Arc::new(SimpleMemory::new(Arc::new(backend)));

        let mut context = AgentContext::new("test input");
        context.add_message(MessageRole::User, "Hello".to_string());
        context.add_message(MessageRole::Assistant, "Hi there!".to_string());

        // Store conversation turn
        store_conversation_turn("test-agent", &context, &memory, 1)
            .await
            .unwrap();

        // Create new context and restore
        let mut new_context = AgentContext::new("test input");
        restore_conversation_history("test-agent", &new_context, &memory)
            .await
            .unwrap();

        // Verify history restored
        assert_eq!(new_context.messages.len(), 2);
    }

    #[tokio::test]
    async fn test_pruning() {
        let mut messages = Vec::new();

        // Create 150 messages (exceeds MAX_MESSAGES = 100)
        messages.push(aof_core::Message {
            role: MessageRole::System,
            content: "System prompt".to_string(),
            tool_calls: None,
        });

        for i in 0..149 {
            messages.push(aof_core::Message {
                role: MessageRole::User,
                content: format!("Message {}", i),
                tool_calls: None,
            });
        }

        let pruned = prune_conversation_history("test-agent", messages);

        // Should be pruned to 100 messages (1 system + 99 recent)
        assert_eq!(pruned.len(), 100);

        // System message should be preserved
        assert_eq!(pruned[0].role, MessageRole::System);
    }
}
