//! Example: Streaming agent execution with real-time updates
//!
//! This example demonstrates how to use the streaming API for real-time
//! agent execution with text deltas, tool events, and progress tracking.
//!
//! Run with: cargo run --example streaming_example

use aof_runtime::{Runtime, StreamEvent};
use tokio::sync::mpsc;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AOF Streaming Example");
    println!("=" .repeat(50));

    // Create runtime and load agent
    let mut runtime = Runtime::new();

    // For this example, we'll need a config file or programmatic config
    // This is just a demonstration of the API
    println!("\n📝 Note: This example requires an agent configuration.");
    println!("   See docs/STREAMING_IMPLEMENTATION.md for setup.\n");

    // Example 1: Basic streaming with real-time output
    println!("Example 1: Basic Streaming");
    println!("-" .repeat(50));

    let (tx, mut rx) = mpsc::channel::<StreamEvent>(100);

    // Spawn task to handle stream events
    let stream_handler = tokio::spawn(async move {
        let mut iteration_count = 0;

        while let Some(event) = rx.recv().await {
            match event {
                StreamEvent::TextDelta { delta, timestamp } => {
                    // Real-time typing effect
                    print!("{}", delta);
                    io::stdout().flush().ok();
                }

                StreamEvent::ToolCallStart { tool_name, tool_id, arguments } => {
                    println!("\n\n🔧 Calling tool: {}", tool_name);
                    println!("   ID: {}", tool_id);
                    println!("   Args: {}", arguments);
                }

                StreamEvent::ToolCallComplete { tool_name, success, execution_time_ms, error, .. } => {
                    if success {
                        println!("   ✅ {} completed in {}ms", tool_name, execution_time_ms);
                    } else {
                        println!("   ❌ {} failed: {:?}", tool_name, error);
                    }
                }

                StreamEvent::Thinking { content } => {
                    println!("\n💭 Thinking: {}", content);
                }

                StreamEvent::IterationStart { iteration, max_iterations } => {
                    iteration_count = iteration;
                    println!("\n\n🔄 Iteration {}/{}", iteration, max_iterations);
                }

                StreamEvent::IterationComplete { iteration, stop_reason } => {
                    println!("\n✓ Iteration {} complete (reason: {:?})", iteration, stop_reason);
                }

                StreamEvent::Done { content, total_iterations, execution_time_ms, input_tokens, output_tokens } => {
                    println!("\n\n🎉 Execution Complete!");
                    println!("=" .repeat(50));
                    println!("Iterations: {}", total_iterations);
                    println!("Time: {}ms", execution_time_ms);
                    println!("Tokens: {} input, {} output", input_tokens, output_tokens);
                    println!("\nFinal content length: {} chars", content.len());
                }

                StreamEvent::Error { message } => {
                    eprintln!("\n❌ Error: {}", message);
                }
            }
        }

        println!("\n\nStream ended after {} iterations", iteration_count);
    });

    // Simulate agent execution (in real usage, you'd call runtime.execute_streaming)
    println!("\n📤 Simulating agent execution...\n");

    // Send example events to demonstrate the API
    tx.send(StreamEvent::IterationStart { iteration: 1, max_iterations: 3 }).await?;

    tx.send(StreamEvent::TextDelta {
        delta: "Hello".to_string(),
        timestamp: Some(chrono::Utc::now().timestamp_millis() as u64)
    }).await?;

    tx.send(StreamEvent::TextDelta {
        delta: " from".to_string(),
        timestamp: Some(chrono::Utc::now().timestamp_millis() as u64)
    }).await?;

    tx.send(StreamEvent::TextDelta {
        delta: " AOF!".to_string(),
        timestamp: Some(chrono::Utc::now().timestamp_millis() as u64)
    }).await?;

    tx.send(StreamEvent::ToolCallStart {
        tool_name: "example_tool".to_string(),
        tool_id: "call_123".to_string(),
        arguments: serde_json::json!({ "param": "value" }),
    }).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    tx.send(StreamEvent::ToolCallComplete {
        tool_name: "example_tool".to_string(),
        tool_id: "call_123".to_string(),
        success: true,
        execution_time_ms: 487,
        error: None,
    }).await?;

    tx.send(StreamEvent::IterationComplete {
        iteration: 1,
        stop_reason: aof_core::StopReason::EndTurn,
    }).await?;

    tx.send(StreamEvent::Done {
        content: "Hello from AOF!".to_string(),
        total_iterations: 1,
        execution_time_ms: 1250,
        input_tokens: 10,
        output_tokens: 15,
    }).await?;

    // Close channel and wait for handler
    drop(tx);
    stream_handler.await?;

    // Example 2: Demonstrate cancellation (commented out for now)
    println!("\n\nExample 2: Cancellation Support");
    println!("-" .repeat(50));
    println!("See docs/STREAMING_IMPLEMENTATION.md for cancellation examples");

    println!("\n\n✨ Examples complete!");

    Ok(())
}
