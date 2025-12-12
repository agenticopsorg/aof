use anyhow::{Context, Result};
use aof_core::AgentConfig;
use aof_runtime::Runtime;
use std::fs;
use std::io::{self, BufRead, IsTerminal};
use tracing::info;
use crate::resources::ResourceType;

/// Execute a resource (agent, workflow, job) with configuration and input
pub async fn execute(
    resource_type: &str,
    name_or_config: &str,
    input: Option<&str>,
    output: &str,
) -> Result<()> {
    // Parse resource type
    let rt = ResourceType::from_str(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: {}", resource_type))?;

    match rt {
        ResourceType::Agent => run_agent(name_or_config, input, output).await,
        ResourceType::Workflow => run_workflow(name_or_config, input, output).await,
        ResourceType::Job => run_job(name_or_config, input, output).await,
        _ => {
            anyhow::bail!("Resource type '{}' cannot be run directly", resource_type)
        }
    }
}

/// Run an agent with configuration
async fn run_agent(config: &str, input: Option<&str>, output: &str) -> Result<()> {
    info!("Loading agent config from: {}", config);

    // Load and parse agent configuration
    let config_content = fs::read_to_string(config)
        .with_context(|| format!("Failed to read config file: {}", config))?;

    let agent_config: AgentConfig = serde_yaml::from_str(&config_content)
        .with_context(|| format!("Failed to parse agent config from: {}", config))?;

    let agent_name = agent_config.name.clone();
    info!("Agent loaded: {}", agent_name);

    // Create runtime and load agent
    let mut runtime = Runtime::new();
    runtime
        .load_agent_from_config(agent_config)
        .await
        .context("Failed to load agent")?;

    // Check if interactive mode should be enabled (when no input provided and stdin is a TTY)
    if input.is_none() && io::stdin().is_terminal() {
        // Interactive REPL mode - only when stdin is a TTY (terminal)
        run_agent_interactive(&runtime, &agent_name, output).await?;
        return Ok(());
    }

    // Single execution mode
    let input_str = input.unwrap_or("default input");
    let result = runtime
        .execute(&agent_name, input_str)
        .await
        .context("Failed to execute agent")?;

    // Output result in requested format
    output_result(&agent_name, &result, output)?;

    Ok(())
}

/// Run agent in interactive REPL mode with beautiful CLI UI
async fn run_agent_interactive(runtime: &Runtime, agent_name: &str, _output: &str) -> Result<()> {
    // Print welcome message with beautiful styling
    println!("\n{}", "=".repeat(60));
    println!("  🤖 Interactive Agent Console - {}", agent_name);
    println!("  Type your query and press Enter. Type 'exit' or 'quit' to exit.");
    println!("{}\n", "=".repeat(60));

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let input_text = line?;

        // Check for exit commands
        if input_text.trim().is_empty() {
            continue;
        }
        if input_text.trim().to_lowercase() == "exit"
            || input_text.trim().to_lowercase() == "quit" {
            println!("\n{} Goodbye!\n", "👋");
            break;
        }

        // Execute the agent with user input
        print!("\n⏳ Processing...");
        io::Write::flush(&mut io::stdout()).ok();

        let result = runtime
            .execute(agent_name, &input_text)
            .await
            .context("Failed to execute agent")?;

        // Output result with beautiful formatting
        println!("\r{}  Agent Response:", "✓".repeat(1));
        println!("{}\n", "-".repeat(60));
        println!("{}\n", result);
        println!("{}", "─".repeat(60));
        print!("\n💬 You: ");
        io::Write::flush(&mut io::stdout()).ok();
    }

    Ok(())
}

/// Format and output agent result
fn output_result(agent_name: &str, result: &str, output: &str) -> Result<()> {
    match output {
        "json" => {
            let json_output = serde_json::json!({
                "success": true,
                "agent": agent_name,
                "result": result
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&serde_json::json!({
                "success": true,
                "agent": agent_name,
                "result": result
            }))?;
            println!("{}", yaml_output);
        }
        "text" | _ => {
            println!("Agent: {}", agent_name);
            println!("Result: {}", result);
        }
    }
    Ok(())
}

/// Run a workflow (placeholder)
async fn run_workflow(name_or_config: &str, input: Option<&str>, output: &str) -> Result<()> {
    println!("Run workflow - Not yet implemented");
    println!("Workflow: {}", name_or_config);
    if let Some(inp) = input {
        println!("Input: {}", inp);
    }
    println!("Output format: {}", output);
    Ok(())
}

/// Run a job (placeholder)
async fn run_job(name_or_config: &str, input: Option<&str>, output: &str) -> Result<()> {
    println!("Run job - Not yet implemented");
    println!("Job: {}", name_or_config);
    if let Some(inp) = input {
        println!("Input: {}", inp);
    }
    println!("Output format: {}", output);
    Ok(())
}
