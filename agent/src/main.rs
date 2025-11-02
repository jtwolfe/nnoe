use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber;

mod config;
mod core;
mod etcd;
mod nebula;
mod plugin;
mod services;
mod sled_cache;

use config::AgentConfig;
use core::Orchestrator;

#[derive(Parser)]
#[command(name = "nnoe-agent")]
#[command(about = "NNOE Agent - Distributed DDI orchestration agent", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Configuration file path
    #[arg(short, long, default_value = "/etc/nnoe/agent.toml")]
    config: PathBuf,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the agent
    Run,
    /// Validate configuration
    Validate {
        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.debug {
        "nnoe_agent=debug"
    } else {
        "nnoe_agent=info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command.as_ref() {
        Some(Commands::Version) => {
            println!("nnoe-agent {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(Commands::Validate { config }) => {
            let config_path = config.as_ref().unwrap_or(&cli.config);
            let cfg = AgentConfig::load(config_path)?;
            println!("Configuration valid: {:?}", config_path);
            println!("{:#?}", cfg);
            return Ok(());
        }
        Some(Commands::Run) | None => {
            info!("Starting NNOE Agent v{}", env!("CARGO_PKG_VERSION"));

            // Load configuration
            let cfg = AgentConfig::load(&cli.config)?;
            info!("Configuration loaded from {:?}", cli.config);

            // Create and run orchestrator
            let mut orchestrator = Orchestrator::new(cfg).await?;
            orchestrator.run().await?;
        }
    }

    Ok(())
}
