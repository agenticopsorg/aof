use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod commands;
mod resources;

use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aofctl=info,aof_runtime=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI
    let cli = Cli::parse();

    // Execute command
    cli.execute().await?;

    Ok(())
}
