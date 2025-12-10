//! Integration tests for memory persistence in agent execution

use aof_core::{AgentConfig, AgentContext, MessageRole};
use aof_memory::{InMemoryBackend, SimpleMemory};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_conversation_persistence_across_sessions() {
    // Setup
    let backend = InMemoryBackend::new();
    let memory = Arc::new(SimpleMemory::new(Arc::new(backend)));

    // Simulate first session
    let mut context1 = AgentContext::new("What is 2+2?");
    context1.add_message(MessageRole::User, "What is 2+2?".to_string());
    context1.add_message(MessageRole::Assistant, "2+2 equals 4.".to_string());

    // Store conversation
    let conversation_key = "agent:test-agent:conversation";
    let conversation_value = serde_json::to_value(&context1.messages).unwrap();
    memory.store(conversation_key, conversation_value).await.unwrap();

    // Simulate second session - restore history
    let mut context2 = AgentContext::new("What about 3+3?");

    if let Some(history) = memory
        .retrieve::<Vec<aof_core::Message>>(conversation_key)
        .await
        .unwrap()
    {
        context2.messages = history;
    }

    context2.add_message(MessageRole::User, "What about 3+3?".to_string());

    // Verify history restored
    assert_eq!(context2.messages.len(), 3); // 2 from first session + 1 new
    assert_eq!(context2.messages[0].content, "What is 2+2?");
    assert_eq!(context2.messages[1].content, "2+2 equals 4.");
    assert_eq!(context2.messages[2].content, "What about 3+3?");
}

#[tokio::test]
async fn test_turn_metadata_tracking() {
    let backend = InMemoryBackend::new();
    let memory = Arc::new(SimpleMemory::new(Arc::new(backend)));

    // Store turn metadata
    let turn_key = "agent:test-agent:turn:1";
    let turn_value = serde_json::json!({
        "iteration": 1,
        "message_count": 2,
        "input_tokens": 100,
        "output_tokens": 50,
        "tool_calls": 0,
        "timestamp": 1702345678
    });

    memory.store(turn_key, turn_value.clone()).await.unwrap();

    // Retrieve and verify
    let retrieved: serde_json::Value = memory.retrieve(turn_key).await.unwrap().unwrap();
    assert_eq!(retrieved["iteration"], 1);
    assert_eq!(retrieved["message_count"], 2);
    assert_eq!(retrieved["input_tokens"], 100);
}

#[tokio::test]
async fn test_memory_pruning() {
    const MAX_MESSAGES: usize = 100;

    // Create history exceeding limit
    let mut messages = Vec::new();

    // Add system message
    messages.push(aof_core::Message {
        role: MessageRole::System,
        content: "You are a helpful assistant".to_string(),
        tool_calls: None,
    });

    // Add 150 user messages
    for i in 0..150 {
        messages.push(aof_core::Message {
            role: MessageRole::User,
            content: format!("Message {}", i),
            tool_calls: None,
        });
    }

    // Prune (simulating the prune_conversation_history function)
    let system_messages: Vec<_> = messages
        .iter()
        .filter(|m| m.role == MessageRole::System)
        .cloned()
        .collect();

    let recent_messages: Vec<_> = messages
        .into_iter()
        .filter(|m| m.role != MessageRole::System)
        .rev()
        .take(MAX_MESSAGES - system_messages.len())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let mut pruned = system_messages;
    pruned.extend(recent_messages);

    // Verify
    assert_eq!(pruned.len(), 100);
    assert_eq!(pruned[0].role, MessageRole::System);
    assert_eq!(pruned[0].content, "You are a helpful assistant");
    // Last message should be the most recent user message
    assert!(pruned.last().unwrap().content.contains("Message"));
}

#[tokio::test]
async fn test_ttl_expiry() {
    use tokio::time::{sleep, Duration};

    let backend = InMemoryBackend::new();
    let memory = Arc::new(SimpleMemory::new(Arc::new(backend.clone())));

    // Store entry with 1 second TTL
    let entry = aof_core::MemoryEntry::new("ttl-test", serde_json::json!({"data": "test"}))
        .with_ttl(1);

    backend.store("ttl-test", entry).await.unwrap();

    // Should retrieve immediately
    let retrieved1 = backend.retrieve("ttl-test").await.unwrap();
    assert!(retrieved1.is_some());

    // Wait for expiry
    sleep(Duration::from_millis(1100)).await;

    // Should be expired and removed
    let retrieved2 = backend.retrieve("ttl-test").await.unwrap();
    assert!(retrieved2.is_none());

    // Verify it's actually removed
    assert_eq!(backend.len(), 0);
}

#[tokio::test]
async fn test_search_by_prefix() {
    let backend = InMemoryBackend::new();
    let memory = Arc::new(SimpleMemory::new(Arc::new(backend)));

    // Store multiple agent turns
    for i in 1..=5 {
        let turn_key = format!("agent:test-agent:turn:{}", i);
        let turn_value = serde_json::json!({
            "iteration": i,
            "message_count": 2 * i,
            "timestamp": 1702345678 + i as u64
        });
        memory.store(&turn_key, turn_value).await.unwrap();
    }

    // Store some other agent data
    memory
        .store("agent:other-agent:turn:1", serde_json::json!({"iteration": 1}))
        .await
        .unwrap();

    // Search for test-agent turns
    let keys = memory.list_keys().await.unwrap();
    let test_agent_turns: Vec<_> = keys
        .iter()
        .filter(|k| k.starts_with("agent:test-agent:turn:"))
        .collect();

    assert_eq!(test_agent_turns.len(), 5);
}

#[tokio::test]
async fn test_concurrent_memory_access() {
    let backend = Arc::new(InMemoryBackend::new());
    let memory = Arc::new(SimpleMemory::new(backend.clone()));

    let mut handles = vec![];

    // Spawn 10 concurrent tasks storing conversation turns
    for i in 0..10 {
        let memory_clone = Arc::clone(&memory);
        let handle = tokio::spawn(async move {
            let turn_key = format!("agent:concurrent-test:turn:{}", i);
            let turn_value = serde_json::json!({
                "iteration": i,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });

            memory_clone.store(&turn_key, turn_value).await.unwrap();

            // Verify immediate retrieval
            let retrieved: serde_json::Value = memory_clone
                .retrieve(&turn_key)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(retrieved["iteration"], i);
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all 10 entries exist
    assert_eq!(backend.len(), 10);
}

#[tokio::test]
async fn test_cleanup_expired_entries() {
    use tokio::time::{sleep, Duration};

    let backend = InMemoryBackend::new();
    let memory = Arc::new(SimpleMemory::new(Arc::new(backend.clone())));

    // Store 5 entries: 2 with short TTL, 3 without
    let entry1 = aof_core::MemoryEntry::new("agent:test:entry1", serde_json::json!(1)).with_ttl(1);
    let entry2 = aof_core::MemoryEntry::new("agent:test:entry2", serde_json::json!(2)).with_ttl(1);
    let entry3 = aof_core::MemoryEntry::new("agent:test:entry3", serde_json::json!(3));
    let entry4 = aof_core::MemoryEntry::new("agent:test:entry4", serde_json::json!(4));
    let entry5 = aof_core::MemoryEntry::new("agent:test:entry5", serde_json::json!(5));

    backend.store("agent:test:entry1", entry1).await.unwrap();
    backend.store("agent:test:entry2", entry2).await.unwrap();
    backend.store("agent:test:entry3", entry3).await.unwrap();
    backend.store("agent:test:entry4", entry4).await.unwrap();
    backend.store("agent:test:entry5", entry5).await.unwrap();

    assert_eq!(backend.len(), 5);

    // Wait for TTL expiry
    sleep(Duration::from_millis(1100)).await;

    // Trigger lazy cleanup by retrieving all entries
    let keys = memory.list_keys().await.unwrap();
    for key in &keys {
        let _: Option<serde_json::Value> = memory.retrieve(key).await.unwrap();
    }

    // Only 3 entries should remain (the ones without TTL)
    assert_eq!(backend.len(), 3);
}
