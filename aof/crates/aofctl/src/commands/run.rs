use anyhow::{Context, Result};
use aof_core::AgentConfig;
use aof_runtime::Runtime;
use std::fs;
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

    // Execute the agent
    let input_str = input.unwrap_or("default input");
    let result = runtime
        .execute(&agent_name, input_str)
        .await
        .context("Failed to execute agent")?;

    // Output result in requested format
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
