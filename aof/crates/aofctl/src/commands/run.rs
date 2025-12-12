use anyhow::{Context, Result};
use aof_core::AgentConfig;
use aof_runtime::Runtime;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
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

/// Print ASCII banner for AOF (Agentic Ops Framework)
fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("╔════════════════════════════════════════════╦═══════════╗");
    println!("║                                            ║           ║");
    println!("║  A O F  :  AGENTIC OPS FRAMEWORK           ║ v{}      ║", version);
    println!("║  ═ ═ ═  :  DEVOPS AUTOMATION              ║           ║");
    println!("║                                            ║           ║");
    println!("║       https://aof.sh                       ║           ║");
    println!("║                                            ║           ║");
    println!("║  Commands: help, exit/quit                 ║           ║");
    println!("║                                            ║           ║");
    println!("╚════════════════════════════════════════════╩═══════════╝");
    println!();
}

/// Run agent in interactive REPL mode with professional CLI UI
async fn run_agent_interactive(runtime: &Runtime, agent_name: &str, _output: &str) -> Result<()> {
    // Print welcome banner
    print_banner();
    println!("   Agent: {}", agent_name);
    println!("   {}", "─".repeat(56));
    println!("   [Ready]\n");

    let stdin = io::stdin();
    let reader = stdin.lock();

    // Show initial prompt
    print!("💬 ");
    io::stdout().flush().ok();

    for line in reader.lines() {
        let input_text = line?;
        let trimmed = input_text.trim();

        // Check for exit commands
        if trimmed.is_empty() {
            print!("💬 ");
            io::stdout().flush().ok();
            continue;
        }
        if trimmed.to_lowercase() == "exit" || trimmed.to_lowercase() == "quit" {
            println!("\n-- Exiting Agentic Ops Framework --\n");
            break;
        }
        if trimmed.to_lowercase() == "help" {
            println!("\nAvailable commands:");
            println!("  help     Show this help message");
            println!("  exit     Exit the console");
            println!("  quit     Exit the console\n");
            print!("💬 ");
            io::stdout().flush().ok();
            continue;
        }

        // Suppress logs during interactive execution
        let log_level = std::env::var("RUST_LOG").unwrap_or_default();
        std::env::set_var("RUST_LOG", "error");

        // Show processing indicator
        print!("⏳ ");
        io::stdout().flush().ok();

        // Execute the agent with user input
        match runtime.execute(agent_name, trimmed).await {
            Ok(result) => {
                // Clear processing indicator and show response
                println!("\r✓ Response:");
                println!("─".repeat(56));

                // Format response
                for response_line in result.lines() {
                    println!("{}", response_line);
                }

                println!("─".repeat(56));
                println!();
            }
            Err(e) => {
                // Handle errors gracefully
                println!("\r✗ Error: {}\n", e);
            }
        }

        // Restore log level
        if log_level.is_empty() {
            std::env::remove_var("RUST_LOG");
        } else {
            std::env::set_var("RUST_LOG", log_level);
        }

        // Show next prompt
        print!("💬 ");
        io::stdout().flush().ok();
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
