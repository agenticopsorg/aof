use clap::{Parser, Subcommand};

use crate::commands;

/// AOF CLI - kubectl-style agent orchestration
#[derive(Parser, Debug)]
#[command(name = "aofctl")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run an agent workflow
    Run {
        /// Agent configuration file (YAML)
        #[arg(short, long)]
        config: String,

        /// Input/query for the agent
        #[arg(short, long)]
        input: String,

        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        output: String,
    },

    /// Get agent/workflow status
    Get {
        /// Resource type (agent, workflow, tool)
        resource: String,

        /// Resource name (optional)
        name: Option<String>,
    },

    /// Apply configuration
    Apply {
        /// Configuration file (YAML)
        #[arg(short, long)]
        file: String,
    },

    /// Delete resource
    Delete {
        /// Resource type
        resource: String,

        /// Resource name
        name: String,
    },

    /// List MCP tools
    Tools {
        /// MCP server command
        #[arg(long)]
        server: String,

        /// Server arguments
        #[arg(long)]
        args: Vec<String>,
    },

    /// Validate agent configuration
    Validate {
        /// Configuration file
        #[arg(short, long)]
        file: String,
    },

    /// Show version information
    Version,
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Run {
                config,
                input,
                output,
            } => commands::run::execute(&config, &input, &output).await,
            Commands::Get { resource, name } => {
                commands::get::execute(&resource, name.as_deref()).await
            }
            Commands::Apply { file } => commands::apply::execute(&file).await,
            Commands::Delete { resource, name } => commands::delete::execute(&resource, &name).await,
            Commands::Tools { server, args } => commands::tools::execute(&server, &args).await,
            Commands::Validate { file } => commands::validate::execute(&file).await,
            Commands::Version => commands::version::execute().await,
        }
    }
}
